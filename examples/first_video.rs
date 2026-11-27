mod common;

use std::{
    f64::consts::{FRAC_PI_2, TAU},
    io,
};

use common::*;
use zl_scene::{
    Animation, BuiltInFn, Construction, Easing, Expr, LineStyle, Property, Scene, Step, Value,
};

const CENTER_X: f64 = -5.3;
const RADIUS: f64 = 1.3;
const ARC_RADIUS: f64 = 0.35;
const GRAPH_START: f64 = -3.2;
const STRETCH: f64 = 0.75;
const TURNS: f64 = 2.0;
const SMALL_TICK: f64 = 0.07;
const LARGE_TICK: f64 = 0.14;

fn guide(scene: &mut Scene, name: &str, start: (f64, f64), end: (f64, f64)) {
    scene.add(object(
        name,
        Construction::Segment {
            start: vector(start.0, start.1, 0.0),
            end: vector(end.0, end.1, 0.0),
        },
        styled(GRAY, LineStyle::Solid, 2.0),
    ));
}

fn graph_axes(scene: &mut Scene) {
    let graph_end = GRAPH_START + STRETCH * TURNS * TAU;
    let reach = RADIUS + 0.3;
    guide(scene, "x axis", (GRAPH_START, 0.0), (graph_end + 0.4, 0.0));
    guide(scene, "y axis", (GRAPH_START, -reach), (GRAPH_START, reach));
    for k in 1..=(4.0 * TURNS) as u32 {
        let x = GRAPH_START + STRETCH * f64::from(k) * FRAC_PI_2;
        let tick = if k.is_multiple_of(2) {
            LARGE_TICK
        } else {
            SMALL_TICK
        };
        guide(scene, "x tick", (x, -tick), (x, tick));
    }
    for y in [-RADIUS, -RADIUS / 2.0, RADIUS / 2.0, RADIUS] {
        let tick = if y.abs() == RADIUS {
            LARGE_TICK
        } else {
            SMALL_TICK
        };
        guide(
            scene,
            "y tick",
            (GRAPH_START - tick, y),
            (GRAPH_START + tick, y),
        );
    }
}

fn main() -> io::Result<()> {
    let mut scene = Scene::new();
    let alpha = scene.add(number("alpha", 0.0));
    let cos = |angle: Expr| call(BuiltInFn::Cos, vec![angle]);
    let sin = |angle: Expr| call(BuiltInFn::Sin, vec![angle]);
    let graph_end = || add(real(GRAPH_START), mul(real(STRETCH), value(alpha)));

    let reach = RADIUS + 0.3;
    guide(
        &mut scene,
        "horizontal diameter",
        (CENTER_X - reach, 0.0),
        (CENTER_X + reach, 0.0),
    );
    guide(
        &mut scene,
        "vertical diameter",
        (CENTER_X, -reach),
        (CENTER_X, reach),
    );
    graph_axes(&mut scene);

    scene.add(object(
        "circle",
        Construction::Circle {
            center: vector(CENTER_X, 0.0, 0.0),
            radius: real(RADIUS),
        },
        colored(BLUE, LineStyle::Solid),
    ));
    scene.add(object(
        "angle",
        Construction::Curve {
            variable: "t".to_string(),
            position: vector_of(
                add(real(CENTER_X), mul(real(ARC_RADIUS), cos(variable("t")))),
                mul(real(ARC_RADIUS), sin(variable("t"))),
            ),
            from: real(0.0),
            to: call(BuiltInFn::Mod, vec![value(alpha), real(TAU)]),
        },
        colored(ORANGE, LineStyle::Solid),
    ));
    scene.add(object(
        "sine",
        Construction::Graph {
            variable: "x".to_string(),
            expression: mul(
                real(RADIUS),
                sin(div(sub(variable("x"), real(GRAPH_START)), real(STRETCH))),
            ),
            from: real(GRAPH_START),
            to: graph_end(),
        },
        colored(GREEN, LineStyle::Solid),
    ));

    let center = scene.add(free_point("C", CENTER_X, 0.0));
    let p = scene.add(point_at(
        "P",
        add(real(CENTER_X), mul(real(RADIUS), cos(value(alpha)))),
        mul(real(RADIUS), sin(value(alpha))),
        colored(ORANGE, LineStyle::Solid),
    ));
    let q = scene.add(point_at(
        "Q",
        graph_end(),
        mul(real(RADIUS), sin(value(alpha))),
        colored(GREEN, LineStyle::Solid),
    ));
    scene.add(segment("CP", center, p, colored(ORANGE, LineStyle::Solid)));
    scene.add(segment("PQ", p, q, colored(GRAY, LineStyle::Dashed)));

    scene.add_step(Step {
        animations: Vec::new(),
        pause: 0.5,
    });
    scene.add_step(Step {
        animations: vec![Animation {
            object: alpha,
            property: Property::Value,
            to: Value::Real(TURNS * TAU),
            duration: 5.0 * TURNS,
            delay: 0.0,
            easing: Easing::Linear,
        }],
        pause: 1.0,
    });

    export_video(&scene, "first_video.mp4", 720, 60)
}
