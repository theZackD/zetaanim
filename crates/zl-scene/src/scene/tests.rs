use super::*;
use crate::{
    BinaryOp, BuiltInFn, Callee, Construction, Expr, LineStyle, ObjectStyle, Origin, Property,
    Value,
};
use glam::DVec3;
use kurbo::{ParamCurve, PathEl, Shape};
use num_complex::Complex;
use std::f64::consts::{FRAC_PI_2, FRAC_PI_3};
use zl_math::Easing;
use zl_primitives::{Color, Dash, Stroke, Style};

fn point(name: &str, x: f64, y: f64) -> Object {
    Object {
        name: name.to_string(),
        origin: Origin::Mouse,
        construction: Construction::Point {
            position: Expr::Constant(Value::Vector(DVec3::new(x, y, 0.0))),
        },
        style: ObjectStyle::default(),
    }
}

#[test]
fn add_returns_distinct_ids() {
    let mut scene = Scene::new();
    let a = scene.add(point("A", 0.0, 0.0));
    let b = scene.add(point("B", 1.0, 0.0));
    assert_ne!(a, b);
}

#[test]
fn get_returns_added_object() {
    let mut scene = Scene::new();
    let id = scene.add(point("A", 2.0, 3.0));
    assert_eq!(scene.get(id), Some(&point("A", 2.0, 3.0)));
}

#[test]
fn get_unknown_id_returns_none() {
    let mut scene = Scene::new();
    let id = scene.add(point("A", 0.0, 0.0));
    assert_eq!(scene.get(ObjectId(id.0 + 100)), None);
}

#[test]
fn remove_returns_object_and_forgets_it() {
    let mut scene = Scene::new();
    let id = scene.add(point("A", 2.0, 3.0));
    assert_eq!(scene.remove(id), Some(point("A", 2.0, 3.0)));
    assert_eq!(scene.get(id), None);
}

#[test]
fn remove_twice_returns_none() {
    let mut scene = Scene::new();
    let id = scene.add(point("A", 0.0, 0.0));
    scene.remove(id);
    assert_eq!(scene.remove(id), None);
}

#[test]
fn removed_id_is_never_reused() {
    let mut scene = Scene::new();
    let a = scene.add(point("A", 0.0, 0.0));
    scene.remove(a);
    let b = scene.add(point("B", 1.0, 0.0));
    assert_ne!(a, b);
    assert_eq!(scene.get(a), None);
    assert_eq!(scene.get(b).map(|o| o.name.as_str()), Some("B"));
}

#[test]
fn removing_one_object_keeps_the_others() {
    let mut scene = Scene::new();
    let a = scene.add(point("A", 0.0, 0.0));
    let b = scene.add(point("B", 1.0, 0.0));
    scene.remove(a);
    assert_eq!(scene.get(b), Some(&point("B", 1.0, 0.0)));
}

#[test]
fn new_and_default_are_equal() {
    assert_eq!(Scene::new(), Scene::default());
}

#[test]
fn new_scene_has_the_default_canvas_and_camera() {
    let scene = Scene::new();
    assert_eq!(scene.canvas, Canvas::default());
    assert_eq!(scene.camera, Camera::default());
}

fn reference(id: u64, property: Property) -> Expr {
    Expr::Reference {
        object: ObjectId(id),
        property,
    }
}

fn depending_on(name: &str, deps: &[u64]) -> Object {
    let mut position = Expr::Constant(Value::Vector(DVec3::ZERO));
    for dep in deps {
        position = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(position),
            right: Box::new(reference(*dep, Property::Position)),
        };
    }
    Object {
        name: name.to_string(),
        origin: Origin::Mouse,
        construction: Construction::Point { position },
        style: ObjectStyle::default(),
    }
}

fn ids(ids: &[u64]) -> Vec<ObjectId> {
    ids.iter().map(|id| ObjectId(*id)).collect()
}

fn id_set(ids: &[u64]) -> BTreeSet<ObjectId> {
    ids.iter().map(|id| ObjectId(*id)).collect()
}

fn checked_order(scene: &Scene) -> (Vec<ObjectId>, BTreeSet<ObjectId>) {
    let (order, blocked) = scene.evaluation_order();
    let mut seen = BTreeSet::new();
    for id in &order {
        assert!(seen.insert(*id), "{id:?} appears twice in the order");
        assert!(!blocked.contains(id), "{id:?} is both ordered and blocked");
        for dep in scene.objects[id].construction.dependencies() {
            if scene.objects.contains_key(&dep) {
                assert!(
                    seen.contains(&dep),
                    "{id:?} comes before its dependency {dep:?}"
                );
            }
        }
    }
    for id in &blocked {
        assert!(
            scene.objects[id]
                .construction
                .dependencies()
                .iter()
                .any(|dep| blocked.contains(dep)),
            "{id:?} is blocked without depending on a blocked object"
        );
    }
    let mut all: BTreeSet<ObjectId> = seen;
    all.extend(blocked.iter().copied());
    let expected: BTreeSet<ObjectId> = scene.objects.keys().copied().collect();
    assert_eq!(
        all, expected,
        "order and blocked must cover the scene exactly"
    );
    (order, blocked)
}

#[test]
fn empty_scene_has_an_empty_order() {
    assert_eq!(checked_order(&Scene::new()), (ids(&[]), id_set(&[])));
}

#[test]
fn independent_objects_come_in_increasing_id_order() {
    let mut scene = Scene::new();
    scene.add(point("A", 0.0, 0.0));
    scene.add(point("B", 1.0, 0.0));
    scene.add(point("C", 2.0, 0.0));
    assert_eq!(checked_order(&scene), (ids(&[0, 1, 2]), id_set(&[])));
}

#[test]
fn dependency_on_a_later_object_is_ordered_first() {
    let mut scene = Scene::new();
    scene.add(depending_on("M", &[1]));
    scene.add(point("A", 0.0, 0.0));
    assert_eq!(checked_order(&scene), (ids(&[1, 0]), id_set(&[])));
}

#[test]
fn chain_created_backwards_is_fully_reversed() {
    let mut scene = Scene::new();
    scene.add(depending_on("D", &[1]));
    scene.add(depending_on("C", &[2]));
    scene.add(depending_on("B", &[3]));
    scene.add(point("A", 0.0, 0.0));
    assert_eq!(checked_order(&scene), (ids(&[3, 2, 1, 0]), id_set(&[])));
}

#[test]
fn shared_dependency_is_ordered_once() {
    let mut scene = Scene::new();
    scene.add(depending_on("D", &[2, 3]));
    scene.add(point("A", 0.0, 0.0));
    scene.add(depending_on("C", &[1]));
    scene.add(depending_on("B", &[1]));
    assert_eq!(checked_order(&scene), (ids(&[1, 2, 3, 0]), id_set(&[])));
}

#[test]
fn smallest_ready_id_goes_first_even_when_it_became_ready_later() {
    let mut scene = Scene::new();
    scene.add(point("A", 0.0, 0.0));
    scene.add(depending_on("M", &[0]));
    scene.add(point("B", 1.0, 0.0));
    assert_eq!(checked_order(&scene), (ids(&[0, 1, 2]), id_set(&[])));
}

#[test]
fn circle_created_before_its_center_and_point_is_ordered_after_them() {
    let mut scene = Scene::new();
    let radius = Expr::Call {
        callee: Callee::BuiltIn(BuiltInFn::Norm),
        args: vec![Expr::Binary {
            op: BinaryOp::Sub,
            left: Box::new(reference(2, Property::Position)),
            right: Box::new(reference(1, Property::Position)),
        }],
    };
    scene.add(Object {
        name: "c".to_string(),
        origin: Origin::Mouse,
        construction: Construction::Circle {
            center: reference(1, Property::Position),
            radius,
        },
        style: ObjectStyle::default(),
    });
    scene.add(point("A", 0.0, 0.0));
    scene.add(point("B", 3.0, 4.0));
    assert_eq!(checked_order(&scene), (ids(&[1, 2, 0]), id_set(&[])));
}

#[test]
fn self_reference_is_blocked_alone() {
    let mut scene = Scene::new();
    scene.add(point("A", 0.0, 0.0));
    scene.add(depending_on("P", &[1]));
    scene.add(point("B", 1.0, 0.0));
    assert_eq!(checked_order(&scene), (ids(&[0, 2]), id_set(&[1])));
}

#[test]
fn two_object_cycle_is_blocked() {
    let mut scene = Scene::new();
    scene.add(depending_on("P", &[1]));
    scene.add(depending_on("Q", &[0]));
    scene.add(point("A", 0.0, 0.0));
    assert_eq!(checked_order(&scene), (ids(&[2]), id_set(&[0, 1])));
}

#[test]
fn three_object_cycle_is_blocked() {
    let mut scene = Scene::new();
    scene.add(depending_on("P", &[1]));
    scene.add(depending_on("Q", &[2]));
    scene.add(depending_on("R", &[0]));
    assert_eq!(checked_order(&scene), (ids(&[]), id_set(&[0, 1, 2])));
}

#[test]
fn objects_depending_on_a_cycle_are_blocked_transitively() {
    let mut scene = Scene::new();
    scene.add(depending_on("P", &[1]));
    scene.add(depending_on("Q", &[0]));
    scene.add(depending_on("R", &[1]));
    scene.add(depending_on("S", &[2]));
    scene.add(depending_on("T", &[3, 5]));
    scene.add(point("A", 0.0, 0.0));
    assert_eq!(checked_order(&scene), (ids(&[5]), id_set(&[0, 1, 2, 3, 4])));
}

#[test]
fn objects_beside_a_cycle_are_still_ordered() {
    let mut scene = Scene::new();
    scene.add(depending_on("M", &[3, 4]));
    scene.add(depending_on("P", &[2]));
    scene.add(depending_on("Q", &[1]));
    scene.add(point("A", 0.0, 0.0));
    scene.add(point("B", 1.0, 0.0));
    assert_eq!(checked_order(&scene), (ids(&[3, 4, 0]), id_set(&[1, 2])));
}

#[test]
fn dependency_on_a_missing_object_is_ignored() {
    let mut scene = Scene::new();
    scene.add(depending_on("M", &[1, 42]));
    scene.add(point("A", 0.0, 0.0));
    assert_eq!(checked_order(&scene), (ids(&[1, 0]), id_set(&[])));
}

#[test]
fn dependency_on_a_removed_object_is_ignored() {
    let mut scene = Scene::new();
    let a = scene.add(point("A", 0.0, 0.0));
    scene.add(depending_on("M", &[0]));
    scene.remove(a);
    assert_eq!(checked_order(&scene), (ids(&[1]), id_set(&[])));
}

#[test]
fn long_chain_created_backwards_does_not_overflow() {
    let length = 100_000;
    let mut scene = Scene::new();
    for i in 0..length - 1 {
        scene.add(depending_on("link", &[i + 1]));
    }
    scene.add(point("end", 0.0, 0.0));
    let (order, blocked) = scene.evaluation_order();
    let expected: Vec<ObjectId> = (0..length).rev().map(ObjectId).collect();
    assert_eq!(order, expected);
    assert!(blocked.is_empty());
}

const EPS: f64 = 1e-12;

fn object(name: &str, construction: Construction) -> Object {
    Object {
        name: name.to_string(),
        origin: Origin::Mouse,
        construction,
        style: ObjectStyle::default(),
    }
}

fn point_at(name: &str, position: Expr) -> Object {
    object(name, Construction::Point { position })
}

fn number_of(name: &str, value: Expr) -> Object {
    object(name, Construction::Number { value })
}

fn real(x: f64) -> Expr {
    Expr::Constant(Value::Real(x))
}

fn complex(re: f64, im: f64) -> Expr {
    Expr::Constant(Value::Complex(Complex::new(re, im)))
}

fn vector(x: f64, y: f64, z: f64) -> Expr {
    Expr::Constant(Value::Vector(DVec3::new(x, y, z)))
}

fn vector_of(x: Expr, y: Expr, z: Expr) -> Expr {
    Expr::Vector {
        x: Box::new(x),
        y: Box::new(y),
        z: Box::new(z),
    }
}

fn binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
    Expr::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn call(f: BuiltInFn, args: Vec<Expr>) -> Expr {
    Expr::Call {
        callee: Callee::BuiltIn(f),
        args,
    }
}

fn position(id: u64) -> Expr {
    reference(id, Property::Position)
}

fn value(id: u64) -> Expr {
    reference(id, Property::Value)
}

fn midpoint(a: u64, b: u64) -> Expr {
    binary(
        BinaryOp::Div,
        binary(BinaryOp::Add, position(a), position(b)),
        real(2.0),
    )
}

fn defined(evaluated: &EvaluatedScene) -> Vec<ObjectId> {
    evaluated.values.keys().copied().collect()
}

fn point_value(evaluated: &EvaluatedScene, id: u64) -> DVec3 {
    match evaluated.values.get(&ObjectId(id)) {
        Some(ObjectValue::Point(p)) => *p,
        other => panic!("object {id} should be a point, got {other:?}"),
    }
}

fn real_value(evaluated: &EvaluatedScene, id: u64) -> f64 {
    match evaluated.values.get(&ObjectId(id)) {
        Some(ObjectValue::Number(Value::Real(x))) => *x,
        other => panic!("object {id} should be a real number, got {other:?}"),
    }
}

fn assert_close(actual: DVec3, expected: DVec3) {
    assert!((actual - expected).length() < EPS, "{actual} != {expected}");
}

#[test]
fn empty_scene_evaluates_to_nothing() {
    let evaluated = Scene::new().evaluate(0.0);
    assert!(evaluated.values.is_empty());
    assert!(evaluated.blocked.is_empty());
}

#[test]
fn number_defined_from_a_newer_number() {
    let mut scene = Scene::new();
    scene.add(number_of("a", binary(BinaryOp::Add, value(1), real(1.0))));
    scene.add(number_of("b", real(2.0)));
    let evaluated = scene.evaluate(0.0);
    assert_eq!(
        evaluated.values,
        BTreeMap::from([
            (ObjectId(0), ObjectValue::Number(Value::Real(3.0))),
            (ObjectId(1), ObjectValue::Number(Value::Real(2.0))),
        ])
    );
    assert!(evaluated.blocked.is_empty());
}

#[test]
fn triangle_midpoints_created_before_their_vertices() {
    let mut scene = Scene::new();
    scene.add(point_at("M_AB", midpoint(3, 4)));
    scene.add(point_at("M_BC", midpoint(4, 5)));
    scene.add(point_at("M_CA", midpoint(5, 3)));
    scene.add(point("A", 0.0, 0.0));
    scene.add(point("B", 4.0, 0.0));
    scene.add(point("C", 0.0, 6.0));
    scene.add(object(
        "midline",
        Construction::Segment {
            start: position(0),
            end: position(1),
        },
    ));
    let evaluated = scene.evaluate(0.0);
    assert!(evaluated.blocked.is_empty());
    assert_eq!(defined(&evaluated), ids(&[0, 1, 2, 3, 4, 5, 6]));
    assert_eq!(point_value(&evaluated, 0), DVec3::new(2.0, 0.0, 0.0));
    assert_eq!(point_value(&evaluated, 1), DVec3::new(2.0, 3.0, 0.0));
    assert_eq!(point_value(&evaluated, 2), DVec3::new(0.0, 3.0, 0.0));
    assert_eq!(
        evaluated.values[&ObjectId(6)],
        ObjectValue::Segment {
            start: DVec3::new(2.0, 0.0, 0.0),
            end: DVec3::new(2.0, 3.0, 0.0),
        }
    );
}

#[test]
fn circle_centered_at_a_through_b_and_a_point_on_it() {
    let mut scene = Scene::new();
    let radius = call(
        BuiltInFn::Norm,
        vec![binary(BinaryOp::Sub, position(2), position(1))],
    );
    scene.add(object(
        "c",
        Construction::Circle {
            center: position(1),
            radius,
        },
    ));
    scene.add(point("A", 1.0, 1.0));
    scene.add(point("B", 4.0, 5.0));
    scene.add(number_of("theta", real(FRAC_PI_2)));
    let direction = vector_of(
        call(BuiltInFn::Cos, vec![value(3)]),
        call(BuiltInFn::Sin, vec![value(3)]),
        real(0.0),
    );
    scene.add(point_at(
        "P",
        binary(
            BinaryOp::Add,
            reference(0, Property::Center),
            binary(BinaryOp::Mul, reference(0, Property::Radius), direction),
        ),
    ));
    let evaluated = scene.evaluate(0.0);
    assert!(evaluated.blocked.is_empty());
    assert_eq!(
        evaluated.values[&ObjectId(0)],
        ObjectValue::Circle {
            center: DVec3::new(1.0, 1.0, 0.0),
            radius: 5.0,
        }
    );
    assert_close(point_value(&evaluated, 4), DVec3::new(1.0, 6.0, 0.0));
}

#[test]
fn point_of_z_squared() {
    let mut scene = Scene::new();
    scene.add(number_of("z", complex(1.0, 2.0)));
    scene.add(point_at(
        "P",
        call(
            BuiltInFn::ToPoint,
            vec![binary(BinaryOp::Pow, value(0), real(2.0))],
        ),
    ));
    let evaluated = scene.evaluate(0.0);
    assert!(evaluated.blocked.is_empty());
    assert_close(point_value(&evaluated, 1), DVec3::new(-3.0, 4.0, 0.0));
}

#[test]
fn point_on_the_unit_circle_and_its_abscissa() {
    let mut scene = Scene::new();
    scene.add(number_of(
        "x_P",
        call(BuiltInFn::Dot, vec![position(2), vector(1.0, 0.0, 0.0)]),
    ));
    scene.add(number_of("theta", real(FRAC_PI_3)));
    scene.add(point_at(
        "P",
        vector_of(
            call(BuiltInFn::Cos, vec![value(1)]),
            call(BuiltInFn::Sin, vec![value(1)]),
            real(0.0),
        ),
    ));
    let evaluated = scene.evaluate(0.0);
    assert!(evaluated.blocked.is_empty());
    assert_close(
        point_value(&evaluated, 2),
        DVec3::new(0.5, 3.0_f64.sqrt() / 2.0, 0.0),
    );
    assert!((real_value(&evaluated, 0) - 0.5).abs() < EPS);
}

#[test]
fn undefined_objects_propagate_without_being_blocked() {
    let mut scene = Scene::new();
    scene.add(number_of("theta", vector(1.0, 0.0, 0.0)));
    scene.add(point_at(
        "P",
        vector_of(call(BuiltInFn::Cos, vec![value(0)]), real(0.0), real(0.0)),
    ));
    scene.add(number_of(
        "x_P",
        call(BuiltInFn::Dot, vec![position(1), vector(1.0, 0.0, 0.0)]),
    ));
    scene.add(object(
        "vanished",
        Construction::Circle {
            center: vector(0.0, 0.0, 0.0),
            radius: real(-1.0),
        },
    ));
    scene.add(point_at("O", reference(3, Property::Center)));
    scene.add(point("A", 1.0, 2.0));
    let evaluated = scene.evaluate(0.0);
    assert!(evaluated.blocked.is_empty());
    assert_eq!(defined(&evaluated), ids(&[5]));
}

#[test]
fn cycle_blocks_its_members_and_dependents_but_not_the_rest() {
    let mut scene = Scene::new();
    scene.add(point_at(
        "P",
        binary(BinaryOp::Add, position(1), vector(0.1, 0.0, 0.0)),
    ));
    scene.add(point_at("Q", position(0)));
    scene.add(point_at("R", position(0)));
    scene.add(point_at("S", position(42)));
    scene.add(point("A", 0.0, 0.0));
    scene.add(point_at(
        "B",
        binary(BinaryOp::Add, position(4), vector(1.0, 0.0, 0.0)),
    ));
    scene.add(point_at(
        "T",
        binary(BinaryOp::Add, position(6), vector(0.1, 0.0, 0.0)),
    ));
    let evaluated = scene.evaluate(0.0);
    assert_eq!(evaluated.blocked, id_set(&[0, 1, 2, 6]));
    assert_eq!(defined(&evaluated), ids(&[4, 5]));
    assert_eq!(point_value(&evaluated, 5), DVec3::new(1.0, 0.0, 0.0));
}

#[test]
fn removing_an_object_makes_its_dependents_undefined_not_blocked() {
    let mut scene = Scene::new();
    let a = scene.add(point("A", 1.0, 0.0));
    scene.add(point_at(
        "M",
        binary(BinaryOp::Add, position(0), vector(1.0, 0.0, 0.0)),
    ));
    assert_eq!(defined(&scene.evaluate(0.0)), ids(&[0, 1]));
    scene.remove(a);
    let evaluated = scene.evaluate(0.0);
    assert!(evaluated.values.is_empty());
    assert!(evaluated.blocked.is_empty());
}

#[test]
fn evaluation_remembers_its_time() {
    let mut scene = Scene::new();
    scene.add(point("A", 0.0, 0.0));
    assert_eq!(scene.evaluate(2.5).time, 2.5);
    assert_eq!(scene.evaluate(0.0).time, 0.0);
}

#[test]
fn graph_waits_for_the_objects_it_uses() {
    let x = || Expr::Variable("x".to_string());
    let mut scene = Scene::new();
    scene.add(object(
        "f",
        Construction::Graph {
            variable: "x".to_string(),
            expression: binary(BinaryOp::Mul, value(1), x()),
            from: real(0.0),
            to: value(2),
        },
    ));
    scene.add(number_of("a", real(3.0)));
    scene.add(number_of("b", real(5.0)));
    scene.add(object(
        "g",
        Construction::Graph {
            variable: "x".to_string(),
            expression: binary(BinaryOp::Mul, value(4), x()),
            from: real(0.0),
            to: real(1.0),
        },
    ));
    scene.add(number_of("undefined", vector(1.0, 0.0, 0.0)));
    let evaluated = scene.evaluate(0.0);
    assert!(evaluated.blocked.is_empty());
    assert_eq!(defined(&evaluated), ids(&[0, 1, 2]));
    assert_eq!(
        evaluated.values[&ObjectId(0)],
        ObjectValue::Curve { from: 0.0, to: 5.0 }
    );
}

#[test]
fn evaluating_twice_gives_the_same_result() {
    let mut scene = Scene::new();
    scene.add(point_at("M", midpoint(1, 2)));
    scene.add(point("A", 0.0, 0.0));
    scene.add(point("B", 2.0, 2.0));
    scene.add(point_at("P", position(3)));
    assert_eq!(scene.evaluate(0.0), scene.evaluate(0.0));
}

const FOREGROUND: Color = Color {
    r: 30,
    g: 30,
    b: 30,
    a: 255,
};
const RED: Color = Color {
    r: 255,
    g: 0,
    b: 0,
    a: 255,
};

fn styled(mut object: Object, style: ObjectStyle) -> Object {
    object.style = style;
    object
}

fn circle_at(x: f64, y: f64, radius: f64) -> Object {
    object(
        "c",
        Construction::Circle {
            center: vector(x, y, 0.0),
            radius: real(radius),
        },
    )
}

fn segment_between(start: (f64, f64), end: (f64, f64)) -> Object {
    object(
        "s",
        Construction::Segment {
            start: vector(start.0, start.1, 0.0),
            end: vector(end.0, end.1, 0.0),
        },
    )
}

fn drawn(scene: &Scene) -> Vec<Primitive> {
    scene.primitives(&scene.evaluate(0.0))
}

fn bounds_and_style(primitive: &Primitive) -> (kurbo::Rect, Style) {
    match primitive {
        Primitive::Path { path, style } => (path.bounding_box(), *style),
    }
}

fn assert_disc(primitive: &Primitive, center: (f64, f64), radius: f64) {
    let (bounds, _) = bounds_and_style(primitive);
    let found = (bounds.center().x, bounds.center().y, bounds.width() / 2.0);
    let tolerance = 0.1;
    assert!(
        (found.0 - center.0).abs() <= tolerance
            && (found.1 - center.1).abs() <= tolerance
            && (found.2 - radius).abs() <= tolerance
            && (bounds.height() / 2.0 - radius).abs() <= tolerance,
        "disc at ({}, {}) of radius {} instead of {center:?} and {radius}",
        found.0,
        found.1,
        found.2
    );
}

#[test]
fn point_becomes_a_filled_disc_of_fixed_size() {
    let mut scene = Scene::new();
    scene.add(point("A", 1.0, 1.0));
    let primitives = drawn(&scene);
    assert_eq!(primitives.len(), 1);
    assert_disc(&primitives[0], (1095.0, 405.0), 8.0);
    assert_eq!(
        bounds_and_style(&primitives[0]).1,
        Style {
            stroke: None,
            fill: Some(FOREGROUND),
        }
    );
}

#[test]
fn segment_becomes_a_stroke_through_the_camera() {
    let mut scene = Scene::new();
    scene.add(segment_between((0.0, 0.0), (1.0, 1.0)));
    let primitives = drawn(&scene);
    assert_eq!(primitives.len(), 1);
    let Primitive::Path { path, style } = &primitives[0];
    assert_eq!(
        path.elements(),
        &[
            PathEl::MoveTo(kurbo::Point::new(960.0, 540.0)),
            PathEl::LineTo(kurbo::Point::new(1095.0, 405.0)),
        ]
    );
    assert_eq!(
        *style,
        Style {
            stroke: Some(Stroke {
                color: FOREGROUND,
                width: 4.0,
                dash: None,
            }),
            fill: None,
        }
    );
}

#[test]
fn zoom_scales_a_circle_but_not_a_point() {
    let mut scene = Scene::new();
    scene.add(circle_at(0.0, 0.0, 1.0));
    scene.add(point("A", 0.0, 0.0));
    let primitives = drawn(&scene);
    assert_disc(&primitives[0], (960.0, 540.0), 135.0);
    assert_disc(&primitives[1], (960.0, 540.0), 8.0);
    scene.camera.extent = 4.0;
    let zoomed = drawn(&scene);
    assert_disc(&zoomed[0], (960.0, 540.0), 270.0);
    assert_disc(&zoomed[1], (960.0, 540.0), 8.0);
}

#[test]
fn missing_color_follows_the_canvas_foreground() {
    let mut scene = Scene::new();
    scene.add(point("A", 0.0, 0.0));
    scene.add(styled(
        point("B", 1.0, 0.0),
        ObjectStyle {
            color: Some(RED),
            ..ObjectStyle::default()
        },
    ));
    let fills = |scene: &Scene| -> Vec<Option<Color>> {
        drawn(scene)
            .iter()
            .map(|primitive| bounds_and_style(primitive).1.fill)
            .collect()
    };
    assert_eq!(fills(&scene), vec![Some(FOREGROUND), Some(RED)]);
    let white = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    scene.canvas.foreground = white;
    assert_eq!(fills(&scene), vec![Some(white), Some(RED)]);
}

#[test]
fn filled_circle_keeps_its_fill_and_its_stroke() {
    let mut scene = Scene::new();
    scene.add(styled(
        circle_at(0.0, 0.0, 1.0),
        ObjectStyle {
            fill: Some(RED),
            ..ObjectStyle::default()
        },
    ));
    let primitives = drawn(&scene);
    let (_, style) = bounds_and_style(&primitives[0]);
    assert_eq!(style.fill, Some(RED));
    assert_eq!(style.stroke.map(|stroke| stroke.color), Some(FOREGROUND));
}

#[test]
fn dash_lengths_follow_the_width() {
    let cases = [
        (LineStyle::Solid, None),
        (LineStyle::Dashed, Some(Dash { on: 8.0, off: 6.0 })),
        (LineStyle::Dotted, Some(Dash { on: 2.0, off: 4.0 })),
    ];
    for (line, expected) in cases {
        let mut scene = Scene::new();
        scene.add(styled(
            segment_between((0.0, 0.0), (1.0, 0.0)),
            ObjectStyle {
                width: 2.0,
                line,
                ..ObjectStyle::default()
            },
        ));
        let (_, style) = bounds_and_style(&drawn(&scene)[0]);
        assert_eq!(
            style.stroke.and_then(|stroke| stroke.dash),
            expected,
            "{line:?}"
        );
    }
}

#[test]
fn filled_objects_come_first_and_points_last() {
    let mut scene = Scene::new();
    scene.add(point("A", 0.0, 0.0));
    scene.add(segment_between((0.0, 0.0), (1.0, 0.0)));
    scene.add(styled(
        circle_at(0.0, 0.0, 1.0),
        ObjectStyle {
            fill: Some(RED),
            ..ObjectStyle::default()
        },
    ));
    scene.add(circle_at(0.0, 0.0, 2.0));
    scene.add(point("B", 1.0, 0.0));
    let evaluated = scene.evaluate(0.0);
    let alone = |id: u64| {
        let mut ctx = EvalContext {
            objects: evaluated.values.clone(),
            variables: Vec::new(),
            time: evaluated.time,
        };
        crate::draw::object_primitive(
            &scene.objects[&ObjectId(id)],
            &evaluated.values[&ObjectId(id)],
            &scene.camera,
            &scene.canvas,
            &mut ctx,
        )
        .expect("drawable object")
    };
    assert_eq!(
        scene.primitives(&evaluated),
        vec![alone(2), alone(1), alone(3), alone(0), alone(4)]
    );
}

#[test]
fn numbers_blocked_and_undefined_objects_are_not_drawn() {
    let mut scene = Scene::new();
    scene.add(number_of("a", real(2.0)));
    scene.add(point_at("P", position(2)));
    scene.add(point_at("Q", position(1)));
    scene.add(circle_at(0.0, 0.0, -1.0));
    scene.add(point("A", 1.0, 1.0));
    let evaluated = scene.evaluate(0.0);
    assert_eq!(evaluated.blocked, id_set(&[1, 2]));
    let primitives = scene.primitives(&evaluated);
    assert_eq!(primitives.len(), 1);
    assert_disc(&primitives[0], (1095.0, 405.0), 8.0);
}

#[test]
fn invalid_camera_draws_nothing_but_keeps_the_values() {
    let mut scene = Scene::new();
    scene.add(point("A", 1.0, 1.0));
    scene.add(circle_at(0.0, 0.0, 1.0));
    for extent in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        scene.camera.extent = extent;
        let evaluated = scene.evaluate(0.0);
        assert_eq!(defined(&evaluated), ids(&[0, 1]), "extent = {extent}");
        assert!(scene.primitives(&evaluated).is_empty(), "extent = {extent}");
    }
}

#[test]
fn point_with_an_invalid_radius_keeps_its_value_and_its_dependents() {
    for radius in [-8.0, f64::NAN, 0.0, -1e300, 1e300] {
        let mut scene = Scene::new();
        scene.add(styled(
            point("A", 0.0, 0.0),
            ObjectStyle {
                point_radius: radius,
                ..ObjectStyle::default()
            },
        ));
        scene.add(point("B", 1.0, 0.0));
        scene.add(object(
            "AB",
            Construction::Segment {
                start: position(0),
                end: position(1),
            },
        ));
        let evaluated = scene.evaluate(0.0);
        assert_eq!(defined(&evaluated), ids(&[0, 1, 2]), "radius = {radius}");
        let primitives = scene.primitives(&evaluated);
        assert_eq!(primitives.len(), 2, "radius = {radius}");
        assert!(bounds_and_style(&primitives[0]).1.stroke.is_some());
        assert_disc(&primitives[1], (1095.0, 540.0), 8.0);
    }
}

#[test]
fn invalid_width_removes_the_stroke_but_keeps_the_fill() {
    for width in [0.0, -1.0, f64::NAN, f64::INFINITY, 1e300] {
        let invalid = ObjectStyle {
            width,
            ..ObjectStyle::default()
        };
        let mut scene = Scene::new();
        scene.add(styled(
            circle_at(0.0, 0.0, 1.0),
            ObjectStyle {
                fill: Some(RED),
                ..invalid
            },
        ));
        scene.add(styled(circle_at(0.0, 0.0, 2.0), invalid));
        scene.add(styled(segment_between((0.0, 0.0), (1.0, 0.0)), invalid));
        let primitives = drawn(&scene);
        assert_eq!(primitives.len(), 1, "width = {width}");
        assert_eq!(
            bounds_and_style(&primitives[0]).1,
            Style {
                stroke: None,
                fill: Some(RED),
            },
            "width = {width}"
        );
    }
}

#[test]
fn circle_beyond_the_limit_is_skipped_without_hanging() {
    let mut scene = Scene::new();
    scene.add(circle_at(0.0, 0.0, 1e300));
    scene.add(circle_at(0.0, 0.0, 1e6));
    scene.add(circle_at(0.0, 0.0, 7e4));
    scene.add(circle_at(0.0, 0.0, 0.0));
    let evaluated = scene.evaluate(0.0);
    assert_eq!(defined(&evaluated), ids(&[0, 1, 2, 3]));
    let primitives = scene.primitives(&evaluated);
    assert_eq!(primitives.len(), 1);
    assert_disc(&primitives[0], (960.0, 540.0), 7e4 * 135.0);
}

#[test]
fn fill_is_ignored_for_points_and_segments() {
    let filled_red = ObjectStyle {
        fill: Some(RED),
        ..ObjectStyle::default()
    };
    let mut scene = Scene::new();
    scene.add(styled(segment_between((0.0, 0.0), (1.0, 0.0)), filled_red));
    scene.add(styled(point("A", 0.0, 0.0), filled_red));
    let primitives = drawn(&scene);
    assert_eq!(primitives.len(), 2);
    assert_eq!(bounds_and_style(&primitives[0]).1.fill, None);
    assert_eq!(bounds_and_style(&primitives[1]).1.fill, Some(FOREGROUND));
}

#[test]
fn large_circle_stays_within_the_drawing_tolerance() {
    let mut scene = Scene::new();
    scene.add(circle_at(0.0, 0.0, 30.0));
    let primitives = drawn(&scene);
    let Primitive::Path { path, .. } = &primitives[0];
    let center = kurbo::Point::new(960.0, 540.0);
    let radius = 30.0 * 135.0;
    let mut checked = 0;
    for segment in path.segments() {
        for step in 0..=20 {
            let t = f64::from(step) / 20.0;
            let distance = segment.eval(t).distance(center);
            assert!(
                (distance - radius).abs() <= 0.1,
                "point at {distance} from the center, radius {radius}"
            );
            checked += 1;
        }
    }
    assert!(checked > 4 * 21, "{checked} samples");
}

fn graph_object(name: &str, expression: Expr, from: f64, to: f64) -> Object {
    object(
        name,
        Construction::Graph {
            variable: "x".to_string(),
            expression,
            from: real(from),
            to: real(to),
        },
    )
}

fn variable_x() -> Expr {
    Expr::Variable("x".to_string())
}

fn move_count(primitive: &Primitive) -> usize {
    let Primitive::Path { path, .. } = primitive;
    path.elements()
        .iter()
        .filter(|element| matches!(element, PathEl::MoveTo(_)))
        .count()
}

#[test]
fn graph_becomes_a_stroked_path_drawn_before_the_points() {
    let mut scene = Scene::new();
    scene.add(point("A", 0.0, 0.0));
    scene.add(styled(
        graph_object("f", variable_x(), -1.0, 1.0),
        ObjectStyle {
            line: LineStyle::Dashed,
            fill: Some(RED),
            ..ObjectStyle::default()
        },
    ));
    let primitives = drawn(&scene);
    assert_eq!(primitives.len(), 2);
    let (bounds, style) = bounds_and_style(&primitives[0]);
    assert_eq!(
        style,
        Style {
            stroke: Some(Stroke {
                color: FOREGROUND,
                width: 4.0,
                dash: Some(Dash {
                    on: 16.0,
                    off: 12.0
                }),
            }),
            fill: None,
        }
    );
    assert_eq!(
        (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
        (825.0, 405.0, 1095.0, 675.0)
    );
    assert_disc(&primitives[1], (960.0, 540.0), 8.0);
}

#[test]
fn graph_with_holes_lifts_the_pen_between_pieces() {
    let mut scene = Scene::new();
    let inverse = binary(BinaryOp::Div, real(1.0), variable_x());
    scene.add(graph_object("1/x", inverse, -2.0, 2.0));
    let primitives = drawn(&scene);
    assert_eq!(primitives.len(), 1);
    assert_eq!(move_count(&primitives[0]), 2);
}

#[test]
fn graph_reads_a_number_of_the_scene() {
    let mut scene = Scene::new();
    scene.add(number_of("a", real(0.5)));
    scene.add(graph_object(
        "a x",
        binary(BinaryOp::Mul, value(0), variable_x()),
        -2.0,
        2.0,
    ));
    let primitives = drawn(&scene);
    let (bounds, _) = bounds_and_style(&primitives[0]);
    assert_eq!(
        (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
        (690.0, 405.0, 1230.0, 675.0)
    );
}

#[test]
fn parametric_circle_follows_the_camera_scale() {
    let mut scene = Scene::new();
    let t = || Expr::Variable("t".to_string());
    scene.add(object(
        "circle",
        Construction::Curve {
            variable: "t".to_string(),
            position: vector_of(
                call(BuiltInFn::Cos, vec![t()]),
                call(BuiltInFn::Sin, vec![t()]),
                real(0.0),
            ),
            from: real(0.0),
            to: real(6.0),
        },
    ));
    let primitives = drawn(&scene);
    let Primitive::Path { path, .. } = &primitives[0];
    let center = kurbo::Point::new(960.0, 540.0);
    for element in path.elements() {
        if let PathEl::MoveTo(p) | PathEl::LineTo(p) = element {
            assert!((p.distance(center) - 135.0).abs() < 1e-9, "{p:?}");
        }
    }
}

#[test]
fn graph_just_outside_the_canvas_is_kept_within_the_reach_of_miter_joins() {
    let ripple = binary(
        BinaryOp::Mul,
        real(0.003),
        call(
            BuiltInFn::Sin,
            vec![binary(BinaryOp::Mul, real(200.0), variable_x())],
        ),
    );
    let wave = binary(BinaryOp::Add, real(4.0 + 5.0 / 135.0), ripple);
    let drawn_with = |width: f64| {
        let mut scene = Scene::new();
        scene.add(styled(
            graph_object("wave", wave.clone(), -1.0, 1.0),
            ObjectStyle {
                width,
                ..ObjectStyle::default()
            },
        ));
        drawn(&scene)
    };
    assert!(drawn_with(1.0).is_empty());
    let kept = drawn_with(4.0);
    assert_eq!(kept.len(), 1);
    let Primitive::Path { path, .. } = &kept[0];
    assert!(path.elements().len() > 130, "{}", path.elements().len());
    let (bounds, _) = bounds_and_style(&kept[0]);
    assert!(bounds.y0 >= -8.0 && bounds.y1 <= 0.0, "{bounds:?}");
}

fn assert_within_the_visible_area(primitive: &Primitive, margin: f64) {
    let (bounds, _) = bounds_and_style(primitive);
    assert!(
        bounds.x0 >= -margin
            && bounds.y0 >= -margin
            && bounds.x1 <= 1920.0 + margin
            && bounds.y1 <= 1080.0 + margin,
        "{bounds:?}"
    );
}

#[test]
fn graph_reaching_far_off_the_canvas_is_cut_to_it() {
    let mut scene = Scene::new();
    let inverse = binary(BinaryOp::Div, real(1.0), variable_x());
    scene.add(graph_object("1/x", inverse, -2.0, 2.0));
    scene.add(graph_object(
        "exp",
        call(BuiltInFn::Exp, vec![variable_x()]),
        0.0,
        1000.0,
    ));
    let primitives = drawn(&scene);
    assert_eq!(primitives.len(), 2);
    assert_eq!(move_count(&primitives[0]), 2);
    for primitive in &primitives {
        assert_within_the_visible_area(primitive, 8.0);
    }
}

#[test]
fn segment_between_far_points_is_cut_to_the_canvas() {
    let mut scene = Scene::new();
    scene.add(segment_between((-1e6, -2e6), (1e6, 2e6)));
    let primitives = drawn(&scene);
    assert_eq!(primitives.len(), 1);
    let (bounds, _) = bounds_and_style(&primitives[0]);
    let expected = [686.0, -8.0, 1234.0, 1088.0];
    let actual = [bounds.x0, bounds.y0, bounds.x1, bounds.y1];
    for (a, e) in actual.iter().zip(expected) {
        assert!((a - e).abs() < 1e-6, "{actual:?}");
    }
}

#[test]
fn segment_off_the_canvas_is_not_drawn() {
    let mut scene = Scene::new();
    scene.add(segment_between((10.0, 10.0), (20.0, 10.0)));
    scene.add(segment_between((-10.0, -10.0), (10.0, -10.0)));
    assert!(drawn(&scene).is_empty());
}

#[test]
fn graph_without_anything_to_draw_produces_no_primitive() {
    let mut scene = Scene::new();
    scene.add(graph_object(
        "never defined",
        call(BuiltInFn::Sqrt, vec![real(-1.0)]),
        0.0,
        1.0,
    ));
    scene.add(graph_object("reversed", variable_x(), 1.0, 0.0));
    scene.add(graph_object(
        "missing object",
        binary(BinaryOp::Mul, value(42), variable_x()),
        0.0,
        1.0,
    ));
    scene.add(styled(
        graph_object("no stroke", variable_x(), 0.0, 1.0),
        ObjectStyle {
            width: f64::NAN,
            ..ObjectStyle::default()
        },
    ));
    assert!(drawn(&scene).is_empty());
}

#[test]
fn graph_is_not_drawn_with_an_invalid_camera() {
    let mut scene = Scene::new();
    scene.add(graph_object("f", variable_x(), -1.0, 1.0));
    scene.camera.extent = 0.0;
    let evaluated = scene.evaluate(0.0);
    assert_eq!(defined(&evaluated), ids(&[0]));
    assert!(scene.primitives(&evaluated).is_empty());
}

#[test]
fn add_step_appends_to_the_timeline() {
    let mut scene = Scene::new();
    assert!(scene.timeline.steps.is_empty());
    let wait = Step {
        animations: Vec::new(),
        pause: 1.0,
    };
    let longer_wait = Step {
        animations: Vec::new(),
        pause: 2.0,
    };
    scene.add_step(wait.clone());
    scene.add_step(longer_wait.clone());
    assert_eq!(scene.timeline.steps, vec![wait, longer_wait]);
}

fn animate(object: u64, property: Property, to: Value, duration: f64) -> Animation {
    Animation {
        object: ObjectId(object),
        property,
        to,
        duration,
        delay: 0.0,
        easing: Easing::Linear,
    }
}

fn play(scene: &mut Scene, animations: Vec<Animation>, pause: f64) {
    scene.add_step(Step { animations, pause });
}

fn at(x: f64, y: f64) -> Value {
    Value::Vector(DVec3::new(x, y, 0.0))
}

#[test]
fn animated_parameter_drives_the_objects_that_depend_on_it() {
    let mut scene = Scene::new();
    scene.add(number_of("alpha", real(0.0)));
    scene.add(point_at(
        "P",
        vector_of(
            call(BuiltInFn::Cos, vec![value(0)]),
            call(BuiltInFn::Sin, vec![value(0)]),
            real(0.0),
        ),
    ));
    scene.add(point("Q", 5.0, 5.0));
    let turn = 2.0 * std::f64::consts::PI;
    play(
        &mut scene,
        vec![animate(0, Property::Value, Value::Real(turn), 4.0)],
        0.0,
    );
    for (time, alpha) in [
        (0.0, 0.0),
        (1.0, FRAC_PI_2),
        (2.0, turn / 2.0),
        (4.0, turn),
        (10.0, turn),
    ] {
        let evaluated = scene.evaluate(time);
        assert_eq!(real_value(&evaluated, 0), alpha, "alpha at {time}");
        assert_close(
            point_value(&evaluated, 1),
            DVec3::new(alpha.cos(), alpha.sin(), 0.0),
        );
        assert_eq!(point_value(&evaluated, 2), DVec3::new(5.0, 5.0, 0.0));
    }
}

#[test]
fn consecutive_animations_chain_from_where_the_previous_one_stopped() {
    let mut scene = Scene::new();
    scene.add(number_of("a", real(0.0)));
    play(
        &mut scene,
        vec![animate(0, Property::Value, Value::Real(10.0), 2.0)],
        0.0,
    );
    play(
        &mut scene,
        vec![animate(0, Property::Value, Value::Real(0.0), 2.0)],
        0.0,
    );
    for (time, a) in [(1.0, 5.0), (2.0, 10.0), (3.0, 5.0), (4.0, 0.0), (9.0, 0.0)] {
        assert_eq!(real_value(&scene.evaluate(time), 0), a, "a at {time}");
    }
}

#[test]
fn later_animation_wins_when_two_overlap_on_the_same_target() {
    let mut scene = Scene::new();
    scene.add(number_of("a", real(0.0)));
    let later = Animation {
        delay: 1.0,
        ..animate(0, Property::Value, Value::Real(0.0), 2.0)
    };
    let earlier = animate(0, Property::Value, Value::Real(10.0), 4.0);
    play(&mut scene, vec![later, earlier], 0.0);
    for (time, a) in [(0.5, 1.25), (2.0, 2.5), (3.0, 0.0), (4.0, 0.0)] {
        assert_eq!(real_value(&scene.evaluate(time), 0), a, "a at {time}");
    }
}

#[test]
fn animated_midpoint_follows_its_points_until_it_ends() {
    let mut scene = Scene::new();
    scene.add(point("A", 0.0, 0.0));
    scene.add(point("B", 2.0, 0.0));
    scene.add(point_at("M", midpoint(0, 1)));
    play(
        &mut scene,
        vec![
            animate(0, Property::Position, at(-2.0, 0.0), 2.0),
            animate(2, Property::Position, at(0.0, 3.0), 2.0),
        ],
        0.0,
    );
    play(
        &mut scene,
        vec![animate(0, Property::Position, at(4.0, 0.0), 2.0)],
        0.0,
    );
    let halfway = scene.evaluate(1.0);
    assert_close(point_value(&halfway, 0), DVec3::new(-1.0, 0.0, 0.0));
    assert_close(point_value(&halfway, 2), DVec3::new(0.25, 1.5, 0.0));
    assert_eq!(
        point_value(&scene.evaluate(2.0), 2),
        DVec3::new(0.0, 3.0, 0.0)
    );
    let after = scene.evaluate(4.0);
    assert_eq!(point_value(&after, 0), DVec3::new(4.0, 0.0, 0.0));
    assert_eq!(point_value(&after, 2), DVec3::new(0.0, 3.0, 0.0));
}

#[test]
fn round_trip_gives_a_midpoint_back_to_its_points() {
    let mut scene = Scene::new();
    scene.add(point("A", 0.0, 0.0));
    scene.add(point("B", 2.0, 0.0));
    scene.add(point_at("M", midpoint(0, 1)));
    let round_trip = Animation {
        easing: Easing::ThereAndBack,
        ..animate(2, Property::Position, at(0.0, 3.0), 2.0)
    };
    play(&mut scene, vec![round_trip], 0.0);
    play(
        &mut scene,
        vec![animate(0, Property::Position, at(4.0, 0.0), 2.0)],
        0.0,
    );
    assert_eq!(
        point_value(&scene.evaluate(1.0), 2),
        DVec3::new(0.0, 3.0, 0.0)
    );
    assert_eq!(
        point_value(&scene.evaluate(2.0), 2),
        DVec3::new(1.0, 0.0, 0.0)
    );
    assert_eq!(
        point_value(&scene.evaluate(4.0), 2),
        DVec3::new(3.0, 0.0, 0.0)
    );
}

#[test]
fn animating_a_missing_property_makes_the_object_undefined_from_its_start() {
    let mut scene = Scene::new();
    scene.add(point("P", 1.0, 1.0));
    let radius_of_a_point = Animation {
        delay: 1.0,
        ..animate(0, Property::Radius, Value::Real(2.0), 1.0)
    };
    play(&mut scene, vec![radius_of_a_point], 0.0);
    assert_eq!(defined(&scene.evaluate(0.5)), ids(&[0]));
    for time in [1.0, 1.5, 5.0] {
        assert!(defined(&scene.evaluate(time)).is_empty(), "time {time}");
    }
}

#[test]
fn animating_a_graph_or_a_curve_makes_it_undefined() {
    let mut scene = Scene::new();
    scene.add(object(
        "f",
        Construction::Graph {
            variable: "x".to_string(),
            expression: Expr::Variable("x".to_string()),
            from: real(-1.0),
            to: real(1.0),
        },
    ));
    scene.add(object(
        "c",
        Construction::Curve {
            variable: "t".to_string(),
            position: vector_of(Expr::Variable("t".to_string()), real(0.0), real(0.0)),
            from: real(-1.0),
            to: real(1.0),
        },
    ));
    assert_eq!(defined(&scene.evaluate(0.0)), ids(&[0, 1]));
    play(
        &mut scene,
        vec![
            animate(0, Property::Value, Value::Real(2.0), 1.0),
            animate(1, Property::Position, at(1.0, 1.0), 1.0),
        ],
        0.0,
    );
    assert!(defined(&scene.evaluate(0.5)).is_empty());
}

#[test]
fn animating_to_a_value_of_the_wrong_kind_makes_the_object_undefined() {
    let mut scene = Scene::new();
    scene.add(point("P", 1.0, 1.0));
    scene.add(number_of("a", real(2.0)));
    scene.add(point("Q", 3.0, 3.0));
    play(
        &mut scene,
        vec![
            animate(0, Property::Position, Value::Real(3.0), 1.0),
            animate(1, Property::Value, at(1.0, 0.0), 1.0),
        ],
        0.0,
    );
    for time in [0.0, 0.5, 1.0, 5.0] {
        assert_eq!(defined(&scene.evaluate(time)), ids(&[2]), "time {time}");
    }
}

#[test]
fn radius_overshooting_below_zero_is_undefined_only_during_the_overshoot() {
    let mut scene = Scene::new();
    scene.add(circle_at(0.0, 0.0, 1.0));
    let shrink = Animation {
        easing: Easing::OutBack,
        ..animate(0, Property::Radius, Value::Real(0.0), 1.0)
    };
    play(&mut scene, vec![shrink], 0.0);
    assert_eq!(defined(&scene.evaluate(0.2)), ids(&[0]));
    assert!(defined(&scene.evaluate(0.8)).is_empty());
    assert_eq!(
        scene.evaluate(1.0).values.get(&ObjectId(0)),
        Some(&ObjectValue::Circle {
            center: DVec3::ZERO,
            radius: 0.0
        })
    );
}

#[test]
fn animation_of_a_removed_object_is_ignored() {
    let mut scene = Scene::new();
    scene.add(point("P", 1.0, 1.0));
    scene.add(point("Q", 2.0, 2.0));
    play(
        &mut scene,
        vec![animate(0, Property::Position, at(5.0, 5.0), 1.0)],
        0.0,
    );
    scene.remove(ObjectId(0));
    scene.add(point("R", 3.0, 3.0));
    let evaluated = scene.evaluate(1.0);
    assert_eq!(defined(&evaluated), ids(&[1, 2]));
    assert_eq!(point_value(&evaluated, 1), DVec3::new(2.0, 2.0, 0.0));
    assert_eq!(point_value(&evaluated, 2), DVec3::new(3.0, 3.0, 0.0));
}

#[test]
fn evaluation_jumps_anywhere_and_leaves_the_scene_unchanged() {
    let mut scene = Scene::new();
    scene.add(number_of("a", real(0.0)));
    play(
        &mut scene,
        vec![animate(0, Property::Value, Value::Real(10.0), 2.0)],
        1.0,
    );
    play(
        &mut scene,
        vec![animate(0, Property::Value, Value::Real(0.0), 2.0)],
        0.0,
    );
    let before = scene.clone();
    let direct = scene.evaluate(3.0);
    scene.evaluate(7.5);
    assert_eq!(scene.evaluate(3.0), direct);
    assert_eq!(scene.evaluate(1.0), before.evaluate(1.0));
    assert_eq!(scene, before);
    assert_eq!(real_value(&direct, 0), 10.0);
    assert_eq!(real_value(&scene.evaluate(4.0), 0), 5.0);
}
