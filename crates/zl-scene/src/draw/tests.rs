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

fn grid_step(from: f64, to: f64) -> f64 {
    (to - from) / f64::from(GRID)
}

#[test]
fn grid_starts_and_ends_exactly_on_the_bounds() {
    assert_eq!(grid_parameter(-1.3, 2.7, 0), -1.3);
    assert_eq!(grid_parameter(-1.3, 2.7, GRID), 2.7);
}

#[test]
fn grid_steps_are_irregular_but_stay_within_half_a_step() {
    let (from, to) = (-1.3, 2.7);
    let step = grid_step(from, to);
    let steps: Vec<f64> = (1..=GRID)
        .map(|i| grid_parameter(from, to, i) - grid_parameter(from, to, i - 1))
        .collect();
    for (i, s) in steps.iter().enumerate() {
        assert!(*s > 0.5 * step && *s < 1.5 * step, "step {i}: {s}");
    }
    let inner = &steps[1..steps.len() - 1];
    let widest = inner.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let narrowest = inner.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(widest - narrowest > 0.5 * step, "{narrowest} .. {widest}");
}

#[test]
fn distance_to_segment_is_measured_to_the_nearest_point_of_the_segment() {
    let (a, b) = (DVec2::ZERO, DVec2::new(10.0, 0.0));
    assert_eq!(distance_to_segment(DVec2::new(5.0, 3.0), a, b), 3.0);
    assert_eq!(distance_to_segment(DVec2::new(15.0, 0.0), a, b), 5.0);
    assert_eq!(distance_to_segment(DVec2::new(-3.0, 4.0), a, b), 5.0);
    let oblique = distance_to_segment(DVec2::new(0.0, 10.0), a, DVec2::new(10.0, 10.0));
    assert!((oblique - 50f64.sqrt()).abs() < 1e-12, "{oblique}");
}

#[test]
fn distance_to_a_degenerate_segment_is_the_distance_to_its_point() {
    let a = DVec2::new(1.0, 1.0);
    assert_eq!(distance_to_segment(DVec2::new(4.0, 5.0), a, a), 5.0);
}

#[test]
fn jump_does_not_shrink_when_the_interval_is_halved() {
    let p = DVec2::new;
    let cases = [
        (
            "straight",
            [
                p(0.0, 0.0),
                p(2.5, 0.0),
                p(5.0, 0.0),
                p(7.5, 0.0),
                p(10.0, 0.0),
            ],
            false,
        ),
        (
            "step",
            [
                p(0.0, 0.0),
                p(0.001, 0.0),
                p(0.002, 0.0),
                p(0.003, 135.0),
                p(0.004, 135.0),
            ],
            true,
        ),
        (
            "pole",
            [
                p(0.0, 5000.0),
                p(0.001, 9000.0),
                p(0.002, -9000.0),
                p(0.003, -5000.0),
                p(0.004, -3000.0),
            ],
            true,
        ),
        (
            "root edge",
            [
                p(0.0, 0.0),
                p(0.001, 0.5),
                p(0.002, 0.707),
                p(0.003, 0.866),
                p(0.004, 1.0),
            ],
            false,
        ),
        (
            "corner",
            [
                p(0.0, 12.5),
                p(1.0, 7.5),
                p(2.0, 2.5),
                p(3.0, 2.5),
                p(4.0, 7.5),
            ],
            false,
        ),
        (
            "short chord",
            [
                p(0.0, 0.0),
                p(0.0, 0.0),
                p(0.0, 0.0),
                p(0.0, 0.0),
                p(0.0, 0.15),
            ],
            false,
        ),
        (
            "almost all",
            [
                p(0.0, 0.0),
                p(0.0, 0.05),
                p(0.0, 10.0),
                p(0.0, 10.0),
                p(0.0, 10.0),
            ],
            true,
        ),
        (
            "most",
            [
                p(0.0, 0.0),
                p(0.0, 0.5),
                p(0.0, 10.0),
                p(0.0, 10.0),
                p(0.0, 10.0),
            ],
            false,
        ),
    ];
    for (name, [a, q1, m, q3, b], expected) in cases {
        assert_eq!(is_jump(a, q1, m, q3, b), expected, "{name}");
    }
}

fn sampler<'a>(
    construction: &'a Construction,
    ctx: &'a mut EvalContext,
    camera: &'a Camera,
    canvas: &'a Canvas,
) -> Sampler<'a> {
    Sampler {
        construction,
        ctx,
        camera,
        canvas,
        margin: 2.0,
        pieces: Vec::new(),
        current: Vec::new(),
    }
}

#[test]
fn emit_lifts_the_pen_and_drops_single_points() {
    let (construction, camera, canvas) = (graph_of(x()), Camera::default(), Canvas::default());
    let mut ctx = EvalContext::default();
    let mut sampler = sampler(&construction, &mut ctx, &camera, &canvas);
    let (a, b, c) = (DVec2::X, DVec2::Y, DVec2::ONE);
    for point in [Some(a), Some(b), None, Some(c), None, None] {
        sampler.emit(point);
    }
    assert_eq!(sampler.pieces, vec![vec![a, b]]);
    assert!(sampler.current.is_empty());
}

#[test]
fn off_screen_means_beyond_one_edge_and_its_margin() {
    let (construction, camera, canvas) = (graph_of(x()), Camera::default(), Canvas::default());
    let mut ctx = EvalContext::default();
    let sampler = sampler(&construction, &mut ctx, &camera, &canvas);
    let p = DVec2::new;
    let cases = [
        (
            "left",
            [p(-3.0, 10.0), p(-50.0, 500.0), p(-9.0, 2000.0)],
            true,
        ),
        (
            "right",
            [p(1923.0, 10.0), p(5000.0, 500.0), p(1925.0, -70.0)],
            true,
        ),
        (
            "above",
            [p(100.0, -50.0), p(300.0, -400.0), p(500.0, -3.0)],
            true,
        ),
        (
            "below",
            [p(100.0, 1083.0), p(300.0, 9000.0), p(-500.0, 1090.0)],
            true,
        ),
        (
            "inside the left margin",
            [p(-1.0, 10.0), p(-50.0, 500.0), p(-9.0, 2000.0)],
            false,
        ),
        (
            "inside the right margin",
            [p(1921.0, 10.0), p(5000.0, 500.0), p(1925.0, -70.0)],
            false,
        ),
        (
            "inside the top margin",
            [p(100.0, -1.0), p(300.0, -400.0), p(500.0, -90.0)],
            false,
        ),
        (
            "inside the bottom margin",
            [p(100.0, 1081.0), p(300.0, 9000.0), p(-500.0, 1090.0)],
            false,
        ),
        (
            "back inside",
            [p(100.0, -50.0), p(300.0, 600.0), p(500.0, -90.0)],
            false,
        ),
        (
            "around a corner",
            [p(-10.0, 5.0), p(-5.0, -5.0), p(5.0, -10.0)],
            false,
        ),
        (
            "two sides",
            [p(-10.0, 500.0), p(500.0, -10.0), p(-10.0, -10.0)],
            false,
        ),
    ];
    for (name, [a, m, b], expected) in cases {
        assert_eq!(sampler.off_screen(a, m, b), expected, "{name}");
    }
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
        2.0,
    )
}

fn sub(left: Expr, right: Expr) -> Expr {
    binary(BinaryOp::Sub, left, right)
}

fn mul(left: Expr, right: Expr) -> Expr {
    binary(BinaryOp::Mul, left, right)
}

fn div(left: Expr, right: Expr) -> Expr {
    binary(BinaryOp::Div, left, right)
}

fn call(f: BuiltInFn, arg: Expr) -> Expr {
    builtin(f, vec![arg])
}

fn assert_near(p: DVec2, expected: (f64, f64)) {
    let expected = DVec2::new(expected.0, expected.1);
    assert!(p.distance(expected) < 0.01, "{p} instead of {expected}");
}

fn crosses(pieces: &[Vec<DVec2>], x: f64) -> bool {
    pieces
        .iter()
        .any(|piece| piece.windows(2).any(|s| (s[0].x - x) * (s[1].x - x) < 0.0))
}

#[test]
fn straight_graph_is_one_piece_from_bound_to_bound() {
    let pieces = pieces_of(&graph_of(x()), -1.0, 1.0);
    assert_eq!(pieces.len(), 1);
    let piece = &pieces[0];
    assert_eq!(piece[0], DVec2::new(825.0, 675.0));
    assert_eq!(piece[piece.len() - 1], DVec2::new(1095.0, 405.0));
    for (p, q) in piece.iter().zip(&piece[1..]) {
        assert!(q.x > p.x, "{p} then {q}");
        assert!((q.x + q.y - 1500.0).abs() < 1e-9, "{q}");
    }
}

#[test]
fn asymptote_splits_the_curve_without_crossing_it() {
    let inverse = graph_of(div(constant(1.0), x()));
    for (from, to) in [(-1.0, 1.0), (-1.0, 1.1), (-2.0, 2.0)] {
        let pieces = pieces_of(&inverse, from, to);
        assert_eq!(pieces.len(), 2, "[{from}, {to}]");
        assert!(pieces[0].iter().all(|p| p.x < 960.0), "[{from}, {to}]");
        assert!(pieces[1].iter().all(|p| p.x > 960.0), "[{from}, {to}]");
    }
}

#[test]
fn tangent_is_cut_at_each_asymptote() {
    let pieces = pieces_of(&graph_of(call(BuiltInFn::Tan, x())), -7.0, 7.0);
    assert_eq!(pieces.len(), 5);
    for k in -2..2 {
        let asymptote = 960.0 + 135.0 * (f64::from(k) + 0.5) * std::f64::consts::PI;
        assert!(!crosses(&pieces, asymptote), "{asymptote}");
    }
}

#[test]
fn curve_starts_at_the_edge_of_its_domain() {
    let ln = pieces_of(&graph_of(call(BuiltInFn::Ln, x())), -1.0, 1.0);
    assert_eq!(ln.len(), 1);
    assert!(ln[0][0].x >= 960.0 && ln[0][0].x < 960.001, "{}", ln[0][0]);
    let sqrt = pieces_of(&graph_of(call(BuiltInFn::Sqrt, x())), -1.0, 1.0);
    assert_eq!(sqrt.len(), 1);
    assert_near(sqrt[0][0], (960.0, 540.0));
}

#[test]
fn semicircle_is_one_piece_closed_on_the_axis() {
    let semicircle = graph_of(call(BuiltInFn::Sqrt, sub(constant(1.0), squared(x()))));
    for (from, to) in [(-1.0, 1.0), (-1.3, 1.7), (-10.0, 10.0)] {
        let pieces = pieces_of(&semicircle, from, to);
        assert_eq!(pieces.len(), 1, "[{from}, {to}]");
        let piece = &pieces[0];
        assert_near(piece[0], (825.0, 540.0));
        assert_near(piece[piece.len() - 1], (1095.0, 540.0));
    }
}

#[test]
fn hole_inside_a_grid_interval_is_found_and_closed() {
    let outside = graph_of(call(BuiltInFn::Sqrt, sub(squared(x()), constant(0.0001))));
    let pieces = pieces_of(&outside, -1.0, 1.0);
    assert_eq!(pieces.len(), 2);
    assert_near(pieces[0][pieces[0].len() - 1], (958.65, 540.0));
    assert_near(pieces[1][0], (961.35, 540.0));
}

fn worst_gap(construction: &Construction, from: f64, to: f64) -> f64 {
    let pieces = pieces_of(construction, from, to);
    let (camera, canvas) = (Camera::default(), Canvas::default());
    let mut ctx = EvalContext::default();
    let mut worst: f64 = 0.0;
    let n = 20_000;
    for i in 0..=n {
        let t = from + (to - from) * f64::from(i) / f64::from(n);
        let Some(p) =
            sample_point(construction, &mut ctx, t).and_then(|p| camera.to_canvas(p, &canvas))
        else {
            continue;
        };
        if !(0.0..=1920.0).contains(&p.x) || !(0.0..=1080.0).contains(&p.y) {
            continue;
        }
        let mut gap = f64::INFINITY;
        for piece in &pieces {
            if p.x < piece[0].x || p.x > piece[piece.len() - 1].x {
                continue;
            }
            let j = piece
                .partition_point(|q| q.x < p.x)
                .clamp(1, piece.len() - 1);
            for k in j.saturating_sub(2).max(1)..(j + 3).min(piece.len()) {
                gap = gap.min(distance_to_segment(p, piece[k - 1], piece[k]));
            }
        }
        worst = worst.max(gap);
    }
    worst
}

#[test]
fn samples_follow_the_true_curve_within_a_pixel() {
    let tau = std::f64::consts::TAU;
    let sine = |a: f64, b: f64| {
        graph_of(mul(
            constant(a),
            call(BuiltInFn::Sin, mul(constant(b), x())),
        ))
    };
    let cases = [
        ("x^2", graph_of(squared(x())), -2.0, 2.0),
        ("exp", graph_of(call(BuiltInFn::Exp, x())), 0.0, 10.0),
        ("S around an inflection", sine(3.0, 19.0), -7.0, 7.0),
        (
            "in phase with the midpoints",
            sine(1.0, 16.0 * tau),
            0.0,
            8.0,
        ),
        ("in phase with a regular grid", sine(1.0, tau), 0.0, 256.0),
    ];
    for (name, construction, from, to) in cases {
        let gap = worst_gap(&construction, from, to);
        assert!(gap <= 1.0, "{name}: {gap} px");
    }
}

fn assert_cut_without_risers(construction: &Construction, from: f64, to: f64, count: usize) {
    let pieces = pieces_of(construction, from, to);
    assert_eq!(pieces.len(), count, "[{from}, {to}]");
    for piece in &pieces {
        for (p, q) in piece.iter().zip(&piece[1..]) {
            let riser = (q.y - p.y).abs() > 1.0 && (q.x - p.x).abs() < 0.1;
            assert!(!riser, "{p} then {q}");
        }
    }
}

#[test]
fn steps_are_cut_without_risers() {
    assert_cut_without_risers(&graph_of(call(BuiltInFn::Floor, x())), -2.5, 2.5, 6);
    assert_cut_without_risers(&graph_of(call(BuiltInFn::Sign, x())), -1.0, 1.0, 2);
    let modulo = builtin(BuiltInFn::Mod, vec![x(), constant(1.0)]);
    assert_cut_without_risers(&graph_of(modulo), -2.0, 2.0, 4);
    let stairs = div(
        call(BuiltInFn::Floor, mul(constant(10.0), x())),
        constant(10.0),
    );
    assert_cut_without_risers(&graph_of(stairs), -1.0, 1.0, 20);
}

#[test]
fn continuous_curves_are_never_cut() {
    let t = || Expr::Variable("t".to_string());
    let cusp = Construction::Curve {
        variable: "t".to_string(),
        position: Expr::Vector {
            x: Box::new(squared(t())),
            y: Box::new(mul(t(), squared(t()))),
            z: Box::new(constant(0.0)),
        },
        from: constant(0.0),
        to: constant(1.0),
    };
    let mut cases = vec![
        ("sqrt", graph_of(call(BuiltInFn::Sqrt, x())), -1.0, 1.0),
        ("asin", graph_of(call(BuiltInFn::Asin, x())), -2.0, 2.0),
        ("acos", graph_of(call(BuiltInFn::Acos, x())), -2.0, 2.0),
        ("cbrt", graph_of(call(BuiltInFn::Cbrt, x())), -1.0, 1.0),
        (
            "steep atan",
            graph_of(call(BuiltInFn::Atan, mul(constant(1000.0), x()))),
            -1.0,
            1.0,
        ),
        ("cusp", cusp, -1.0, 1.0),
    ];
    for n in 2..=12 {
        let root = binary(BinaryOp::Pow, x(), constant(1.0 / f64::from(n)));
        cases.push(("root", graph_of(root), -1.0, 1.0));
    }
    for (name, construction, from, to) in cases {
        assert_eq!(pieces_of(&construction, from, to).len(), 1, "{name}");
    }
}

#[test]
fn corner_finer_than_the_deepest_level_keeps_its_midpoint() {
    let corner = DVec2::new(960.0 + 0.3 * 135.0, 540.0);
    let v = graph_of(call(BuiltInFn::Abs, sub(x(), constant(0.3))));
    let pieces = pieces_of(&v, -100.0, 100.0);
    assert_eq!(pieces.len(), 1);
    let gap = pieces[0]
        .windows(2)
        .map(|s| distance_to_segment(corner, s[0], s[1]))
        .fold(f64::INFINITY, f64::min);
    assert!(gap < 0.1, "{gap} px");
}

#[test]
fn narrow_bump_on_a_quarter_is_found() {
    let (from, to) = (-7.0, 7.0);
    let (a, b) = (grid_parameter(from, to, 20), grid_parameter(from, to, 21));
    let width = (b - a) / 16.0;
    for quarter in [1.0, 3.0] {
        let top = a + (b - a) * quarter / 4.0;
        let bump = graph_of(mul(
            constant(5.0 / 135.0),
            call(
                BuiltInFn::Exp,
                mul(
                    constant(-1.0 / (width * width)),
                    squared(sub(x(), constant(top))),
                ),
            ),
        ));
        let peak = DVec2::new(960.0 + 135.0 * top, 540.0 - 5.0);
        let gap = pieces_of(&bump, from, to)
            .iter()
            .flat_map(|piece| piece.windows(2))
            .map(|s| distance_to_segment(peak, s[0], s[1]))
            .fold(f64::INFINITY, f64::min);
        assert!(gap < 1.0, "quarter {quarter}: {gap} px");
    }
}

#[test]
fn corners_are_never_cut() {
    let corner = |slope: f64, at: f64, shift: f64| {
        graph_of(sub(
            call(BuiltInFn::Abs, mul(constant(slope), sub(x(), constant(at)))),
            constant(shift),
        ))
    };
    for k in 0..40 {
        let at = -0.5 + f64::from(k) * 0.0273;
        assert_eq!(
            pieces_of(&corner(5.0, at, 3.0), -50.0, 50.0).len(),
            1,
            "{at}"
        );
    }
    assert_eq!(pieces_of(&corner(5.0, 0.3, 0.0), -100.0, 100.0).len(), 1);
    let cusp = graph_of(call(
        BuiltInFn::Sqrt,
        call(BuiltInFn::Abs, sub(x(), constant(0.3))),
    ));
    assert_eq!(pieces_of(&cusp, -100.0, 100.0).len(), 1);
}

#[test]
fn narrow_arches_between_undefined_samples_are_found() {
    let arches = graph_of(call(
        BuiltInFn::Sqrt,
        call(BuiltInFn::Sin, mul(constant(50.0), x())),
    ));
    let pieces = pieces_of(&arches, -7.0, 7.0);
    assert_eq!(pieces.len(), 112);
    for piece in &pieces {
        assert!(piece.iter().any(|p| p.y < 440.0), "{:?}", piece[0]);
    }
}

#[test]
fn off_screen_intervals_are_not_refined() {
    let high_wave = graph_of(binary(
        BinaryOp::Add,
        constant(100.0),
        call(BuiltInFn::Sin, mul(constant(50.0), x())),
    ));
    let pieces = pieces_of(&high_wave, -1.0, 1.0);
    assert_eq!(pieces.len(), 1);
    assert_eq!(pieces[0].len(), GRID as usize + 1);
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
        2.0,
    );
    assert!(pieces.is_empty());
}

fn square() -> Rect {
    Rect::new(0.0, 0.0, 100.0, 100.0)
}

#[test]
fn clip_segment_keeps_the_part_inside_the_area() {
    let p = DVec2::new;
    let cases = [
        ("across", p(-50.0, 50.0), p(150.0, 50.0), Some((0.25, 0.75))),
        ("entering", p(-50.0, 50.0), p(50.0, 50.0), Some((0.5, 1.0))),
        ("inside", p(10.0, 10.0), p(20.0, 30.0), Some((0.0, 1.0))),
        (
            "vertical inside",
            p(50.0, 10.0),
            p(50.0, 90.0),
            Some((0.0, 1.0)),
        ),
        (
            "horizontal inside",
            p(10.0, 50.0),
            p(90.0, 50.0),
            Some((0.0, 1.0)),
        ),
        (
            "single point inside",
            p(50.0, 50.0),
            p(50.0, 50.0),
            Some((0.0, 1.0)),
        ),
        ("parallel below", p(-50.0, 150.0), p(150.0, 150.0), None),
        ("single point outside", p(150.0, 50.0), p(150.0, 50.0), None),
        ("past a corner", p(-10.0, 5.0), p(5.0, -10.0), None),
        ("touching a corner", p(-10.0, 10.0), p(10.0, -10.0), None),
    ];
    for (name, a, b, expected) in cases {
        assert_eq!(clip_segment(a, b, square()), expected, "{name}");
    }
}

#[test]
fn clip_pieces_lifts_the_pen_where_the_curve_leaves_the_area() {
    let p = DVec2::new;
    let out_and_back = vec![
        p(50.0, 50.0),
        p(150.0, 50.0),
        p(150.0, 70.0),
        p(50.0, 70.0),
        p(40.0, 70.0),
    ];
    assert_eq!(
        clip_pieces(&[out_and_back], square()),
        vec![
            vec![p(50.0, 50.0), p(100.0, 50.0)],
            vec![p(100.0, 70.0), p(50.0, 70.0), p(40.0, 70.0)],
        ]
    );
}

#[test]
fn clip_pieces_keeps_inside_pieces_whole_and_apart() {
    let p = DVec2::new;
    let first = vec![p(10.0, 10.0), p(20.0, 20.0), p(30.0, 10.0)];
    let second = vec![p(30.0, 10.0), p(40.0, 40.0)];
    let outside = vec![p(150.0, 10.0), p(170.0, 90.0)];
    assert_eq!(
        clip_pieces(&[first.clone(), outside, second.clone()], square()),
        vec![first, second]
    );
}
