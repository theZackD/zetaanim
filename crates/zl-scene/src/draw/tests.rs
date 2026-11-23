use super::*;
use crate::{BinaryOp, BuiltInFn, Callee, Expr, ObjectId, Property};

#[test]
fn every_layer_is_below_the_layer_count() {
    let plain = ObjectStyle::default();
    let filled = ObjectStyle {
        fill: Some(Color {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        }),
        ..ObjectStyle::default()
    };
    let circle = ObjectValue::Circle {
        center: DVec3::ZERO,
        radius: 1.0,
    };
    let cases = [
        (ObjectValue::Number(crate::Value::Real(1.0)), plain, None),
        (circle.clone(), filled, Some(0)),
        (
            ObjectValue::Segment {
                start: DVec3::ZERO,
                end: DVec3::X,
            },
            plain,
            Some(1),
        ),
        (circle, plain, Some(1)),
        (ObjectValue::Point(DVec3::ZERO), plain, Some(2)),
        (ObjectValue::Curve { from: 0.0, to: 1.0 }, plain, Some(1)),
    ];
    for (value, style, expected) in cases {
        let layer = draw_layer(&value, &style);
        assert_eq!(layer, expected, "{value:?}");
        assert!(layer.is_none_or(|layer| layer < LAYER_COUNT), "{value:?}");
    }
}

fn x() -> Expr {
    Expr::Variable("x".to_string())
}

fn constant(x: f64) -> Expr {
    Expr::Constant(Value::Real(x))
}

fn binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
    Expr::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn builtin(f: BuiltInFn, args: Vec<Expr>) -> Expr {
    Expr::Call {
        callee: Callee::BuiltIn(f),
        args,
    }
}

fn graph_of(expression: Expr) -> Construction {
    Construction::Graph {
        variable: "x".to_string(),
        expression,
        from: constant(0.0),
        to: constant(1.0),
    }
}

fn unit_circle() -> Construction {
    let t = || Expr::Variable("t".to_string());
    Construction::Curve {
        variable: "t".to_string(),
        position: Expr::Vector {
            x: Box::new(builtin(BuiltInFn::Cos, vec![t()])),
            y: Box::new(builtin(BuiltInFn::Sin, vec![t()])),
            z: Box::new(constant(0.0)),
        },
        from: constant(0.0),
        to: constant(1.0),
    }
}

fn squared(e: Expr) -> Expr {
    binary(BinaryOp::Mul, e.clone(), e)
}

#[test]
fn graph_sample_is_x_and_f_of_x() {
    let mut ctx = EvalContext::default();
    assert_eq!(
        sample_point(&graph_of(squared(x())), &mut ctx, 3.0),
        Some(DVec3::new(3.0, 9.0, 0.0))
    );
}

#[test]
fn curve_sample_is_its_position() {
    let mut ctx = EvalContext::default();
    let sample = sample_point(&unit_circle(), &mut ctx, std::f64::consts::FRAC_PI_2)
        .expect("defined sample");
    assert!(
        (sample - DVec3::new(0.0, 1.0, 0.0)).length() < 1e-12,
        "{sample}"
    );
}

#[test]
fn sampling_leaves_the_variable_stack_as_it_found_it() {
    let mut ctx = EvalContext::default();
    let sqrt = graph_of(builtin(BuiltInFn::Sqrt, vec![x()]));
    assert!(sample_point(&sqrt, &mut ctx, 4.0).is_some());
    assert!(ctx.variables.is_empty());
    assert_eq!(sample_point(&sqrt, &mut ctx, -1.0), None);
    assert!(ctx.variables.is_empty());
    assert_eq!(sample_point(&unit_circle(), &mut ctx, f64::NAN), None);
    assert!(ctx.variables.is_empty());
}

#[test]
fn bound_variable_hides_an_outer_one_with_the_same_name() {
    let mut ctx = EvalContext::default();
    ctx.variables.push(("x".to_string(), Value::Real(100.0)));
    assert_eq!(
        sample_point(&graph_of(x()), &mut ctx, 3.0),
        Some(DVec3::new(3.0, 3.0, 0.0))
    );
    assert_eq!(ctx.variables, vec![("x".to_string(), Value::Real(100.0))]);
}

#[test]
fn graph_binds_the_name_it_declares() {
    let mut ctx = EvalContext::default();
    let u = Construction::Graph {
        variable: "u".to_string(),
        expression: squared(Expr::Variable("u".to_string())),
        from: constant(0.0),
        to: constant(1.0),
    };
    assert_eq!(
        sample_point(&u, &mut ctx, 2.0),
        Some(DVec3::new(2.0, 4.0, 0.0))
    );
    assert_eq!(
        sample_point(&graph_of(Expr::Variable("u".to_string())), &mut ctx, 2.0),
        None
    );
}

#[test]
fn graph_sample_must_be_real_and_curve_sample_a_vector() {
    let mut ctx = EvalContext::default();
    let imaginary = binary(
        BinaryOp::Mul,
        Expr::Constant(Value::Complex(num_complex::Complex::new(0.0, 1.0))),
        x(),
    );
    assert_eq!(sample_point(&graph_of(imaginary), &mut ctx, 1.0), None);
    let nearly_real = builtin(BuiltInFn::ToComplex, vec![x()]);
    assert_eq!(
        sample_point(&graph_of(nearly_real), &mut ctx, 2.0),
        Some(DVec3::new(2.0, 2.0, 0.0))
    );
    let vector = Expr::Constant(Value::Vector(DVec3::X));
    assert_eq!(sample_point(&graph_of(vector), &mut ctx, 1.0), None);
    let real_curve = Construction::Curve {
        variable: "t".to_string(),
        position: constant(1.0),
        from: constant(0.0),
        to: constant(1.0),
    };
    assert_eq!(sample_point(&real_curve, &mut ctx, 1.0), None);
}

#[test]
fn other_constructions_have_no_samples() {
    let mut ctx = EvalContext::default();
    let point = Construction::Point {
        position: Expr::Constant(Value::Vector(DVec3::ZERO)),
    };
    assert_eq!(sample_point(&point, &mut ctx, 0.0), None);
}

#[test]
fn graph_reads_the_objects_of_the_scene() {
    let mut ctx = EvalContext::default();
    ctx.objects
        .insert(ObjectId(4), ObjectValue::Number(Value::Real(2.5)));
    let a_times_x = binary(
        BinaryOp::Mul,
        Expr::Reference {
            object: ObjectId(4),
            property: Property::Value,
        },
        x(),
    );
    assert_eq!(
        sample_point(&graph_of(a_times_x), &mut ctx, 2.0),
        Some(DVec3::new(2.0, 5.0, 0.0))
    );
}

fn pieces_of(construction: &Construction, from: f64, to: f64) -> Vec<Vec<DVec2>> {
    let mut ctx = EvalContext::default();
    sample_pieces(
        construction,
        from,
        to,
        &Camera::default(),
        &Canvas::default(),
        &mut ctx,
    )
}

#[test]
fn defined_graph_is_one_piece_of_regular_samples_through_the_camera() {
    let pieces = pieces_of(&graph_of(x()), -1.0, 1.0);
    assert_eq!(pieces.len(), 1);
    let piece = &pieces[0];
    assert_eq!(piece.len(), SAMPLES as usize + 1);
    assert_eq!(piece[0], DVec2::new(825.0, 675.0));
    assert_eq!(piece[piece.len() - 1], DVec2::new(1095.0, 405.0));
    let step = piece[1].x - piece[0].x;
    assert!((step - 270.0 / f64::from(SAMPLES)).abs() < 1e-9, "{step}");
}

#[test]
fn undefined_samples_split_the_curve() {
    let inverse = graph_of(binary(BinaryOp::Div, constant(1.0), x()));
    let pieces = pieces_of(&inverse, -1.0, 1.0);
    assert_eq!(pieces.len(), 2);
    assert!(pieces[0].iter().all(|p| p.x < 960.0));
    assert!(pieces[1].iter().all(|p| p.x > 960.0));
    let half = SAMPLES as usize / 2;
    assert_eq!((pieces[0].len(), pieces[1].len()), (half, half));
}

#[test]
fn curve_starts_after_the_first_defined_sample() {
    let ln = graph_of(builtin(BuiltInFn::Ln, vec![x()]));
    let pieces = pieces_of(&ln, -1.0, 1.0);
    assert_eq!(pieces.len(), 1);
    assert!(pieces[0][0].x > 960.0);
}

#[test]
fn isolated_sample_is_dropped() {
    let only_at_zero = graph_of(builtin(
        BuiltInFn::Sqrt,
        vec![binary(BinaryOp::Sub, constant(0.0), squared(x()))],
    ));
    assert!(pieces_of(&only_at_zero, -1.0, 1.0).is_empty());
    let sqrt = graph_of(builtin(BuiltInFn::Sqrt, vec![x()]));
    assert!(pieces_of(&sqrt, -1.0, 0.0).is_empty());
}

#[test]
fn nothing_is_sampled_with_an_invalid_camera() {
    let mut ctx = EvalContext::default();
    let broken = Camera {
        extent: 0.0,
        ..Camera::default()
    };
    let pieces = sample_pieces(
        &graph_of(x()),
        -1.0,
        1.0,
        &broken,
        &Canvas::default(),
        &mut ctx,
    );
    assert!(pieces.is_empty());
}
