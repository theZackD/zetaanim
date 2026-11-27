mod common;

use std::{
    f64::consts::{FRAC_PI_2, TAU},
    io,
};

use common::*;
use zl_scene::{
    Animation, BuiltInFn, Construction, Easing, Expr, LineStyle, ObjectId, Property, Scene, Step,
    Value,
};

const CENTER_X: f64 = -5.0;
const CENTER_Y: f64 = 2.6;
const RADIUS: f64 = 1.0;
const STRETCH: f64 = 0.41;
const TURNS: f64 = 2.0;
const SINE_START: f64 = -3.4;
const COSINE_START: f64 = 1.3;
const ARC_RADIUS: f64 = 0.3;
const SMALL_TICK: f64 = 0.06;
const LARGE_TICK: f64 = 0.12;
const GUIDE_WIDTH: f64 = 2.0;
const TAIL: f64 = 0.3;

fn cos(angle: Expr) -> Expr {
    call(BuiltInFn::Cos, vec![angle])
}

fn sin(angle: Expr) -> Expr {
    call(BuiltInFn::Sin, vec![angle])
}

fn tick_length(k: u32) -> f64 {
    if k.is_multiple_of(2) {
        LARGE_TICK
    } else {
        SMALL_TICK
    }
}

fn value_ticks() -> [(f64, f64); 4] {
    [
        (-RADIUS, LARGE_TICK),
        (-RADIUS / 2.0, SMALL_TICK),
        (RADIUS / 2.0, SMALL_TICK),
        (RADIUS, LARGE_TICK),
    ]
}

fn length() -> f64 {
    STRETCH * TURNS * TAU
}

fn place(merge: ObjectId, u: Expr, v: Expr) -> (Expr, Expr) {
    let turn = || mul(value(merge), real(FRAC_PI_2));
    let pivot_x = add(
        real(CENTER_X),
        mul(value(merge), real(SINE_START - CENTER_X)),
    );
    let pivot_y = add(
        real(COSINE_START),
        mul(value(merge), real(CENTER_Y - COSINE_START)),
    );
    let x = add(
        pivot_x,
        sub(mul(u.clone(), cos(turn())), mul(v.clone(), sin(turn()))),
    );
    let y = add(pivot_y, add(mul(u, sin(turn())), mul(v, cos(turn()))));
    (x, y)
}

fn guide(scene: &mut Scene, name: &str, start: (f64, f64), end: (f64, f64)) {
    axis_line(scene, name, start, end, GUIDE_WIDTH);
}

fn moving_guide(
    scene: &mut Scene,
    merge: ObjectId,
    name: &str,
    start: (f64, f64),
    end: (f64, f64),
) {
    let (start_x, start_y) = place(merge, real(start.0), real(start.1));
    let (end_x, end_y) = place(merge, real(end.0), real(end.1));
    scene.add(object(
        name,
        Construction::Segment {
            start: vector_of(start_x, start_y),
            end: vector_of(end_x, end_y),
        },
        styled(GRAY, LineStyle::Solid, GUIDE_WIDTH),
    ));
}

fn circle_and_sine_guides(scene: &mut Scene) {
    let reach = RADIUS + 0.3;
    guide(
        scene,
        "horizontal diameter",
        (CENTER_X - reach, CENTER_Y),
        (CENTER_X + reach, CENTER_Y),
    );
    guide(
        scene,
        "vertical diameter",
        (CENTER_X, CENTER_Y - RADIUS - 0.15),
        (CENTER_X, CENTER_Y + reach),
    );
    guide(
        scene,
        "sine time axis",
        (SINE_START, CENTER_Y),
        (SINE_START + length() + TAIL, CENTER_Y),
    );
    guide(
        scene,
        "sine value axis",
        (SINE_START, CENTER_Y - reach),
        (SINE_START, CENTER_Y + reach),
    );
    for k in 1..=(4.0 * TURNS) as u32 {
        let x = SINE_START + STRETCH * f64::from(k) * FRAC_PI_2;
        let tick = tick_length(k);
        guide(
            scene,
            "sine tick",
            (x, CENTER_Y - tick),
            (x, CENTER_Y + tick),
        );
    }
    for (offset, tick) in value_ticks() {
        let y = CENTER_Y + offset;
        guide(
            scene,
            "sine tick",
            (SINE_START - tick, y),
            (SINE_START + tick, y),
        );
    }
}

fn cosine_guides(scene: &mut Scene, merge: ObjectId) {
    let reach = RADIUS + 0.3;
    moving_guide(
        scene,
        merge,
        "cosine time axis",
        (0.0, 0.0),
        (0.0, -(length() + TAIL)),
    );
    moving_guide(
        scene,
        merge,
        "cosine value axis",
        (-reach, 0.0),
        (reach, 0.0),
    );
    for k in 1..=(4.0 * TURNS) as u32 {
        let v = -STRETCH * f64::from(k) * FRAC_PI_2;
        let tick = tick_length(k);
        moving_guide(scene, merge, "cosine tick", (-tick, v), (tick, v));
    }
    for (u, tick) in value_ticks() {
        moving_guide(scene, merge, "cosine tick", (u, -tick), (u, tick));
    }
}

fn animate(object: ObjectId, to: f64, duration: f64, easing: Easing, pause: f64) -> Step {
    Step {
        animations: vec![Animation {
            object,
            property: Property::Value,
            to: Value::Real(to),
            duration,
            delay: 0.0,
            easing,
        }],
        pause,
    }
}

fn main() -> io::Result<()> {
    let mut scene = Scene::new();
    let alpha = scene.add(number("alpha", 0.0));
    let merge = scene.add(number("merge", 0.0));
    let p_x = || add(real(CENTER_X), mul(real(RADIUS), cos(value(alpha))));
    let p_y = || add(real(CENTER_Y), mul(real(RADIUS), sin(value(alpha))));
    let sine_head = || add(real(SINE_START), mul(real(STRETCH), value(alpha)));
    let cosine_head = || {
        place(
            merge,
            mul(real(RADIUS), cos(value(alpha))),
            mul(real(-STRETCH), value(alpha)),
        )
    };

    circle_and_sine_guides(&mut scene);
    cosine_guides(&mut scene, merge);
    scene.add(object(
        "circle",
        Construction::Circle {
            center: vector(CENTER_X, CENTER_Y, 0.0),
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
                add(real(CENTER_Y), mul(real(ARC_RADIUS), sin(variable("t")))),
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
            expression: add(
                real(CENTER_Y),
                mul(
                    real(RADIUS),
                    sin(div(sub(variable("x"), real(SINE_START)), real(STRETCH))),
                ),
            ),
            from: real(SINE_START),
            to: sine_head(),
        },
        colored(GREEN, LineStyle::Solid),
    ));
    let (cosine_x, cosine_y) = place(
        merge,
        mul(real(RADIUS), cos(variable("s"))),
        mul(real(-STRETCH), variable("s")),
    );
    scene.add(object(
        "cosine",
        Construction::Curve {
            variable: "s".to_string(),
            position: vector_of(cosine_x, cosine_y),
            from: real(0.0),
            to: value(alpha),
        },
        colored(PURPLE, LineStyle::Solid),
    ));

    let center = scene.add(free_point("C", CENTER_X, CENTER_Y));
    let p = scene.add(point_at(
        "P",
        p_x(),
        p_y(),
        colored(ORANGE, LineStyle::Solid),
    ));
    let sine_point = scene.add(point_at(
        "sine head",
        sine_head(),
        p_y(),
        colored(GREEN, LineStyle::Solid),
    ));
    let (head_x, head_y) = cosine_head();
    scene.add(point_at(
        "cosine head",
        head_x,
        head_y,
        colored(PURPLE, LineStyle::Solid),
    ));
    scene.add(segment("CP", center, p, colored(ORANGE, LineStyle::Solid)));
    scene.add(segment(
        "sine projection",
        p,
        sine_point,
        colored(GRAY, LineStyle::Dashed),
    ));
    let (head_x, head_y) = cosine_head();
    let (end_x, end_y) = cosine_head();
    scene.add(object(
        "cosine projection",
        Construction::Segment {
            start: vector_of(
                add(p_x(), mul(value(merge), sub(head_x, p_x()))),
                add(p_y(), mul(value(merge), sub(head_y, p_y()))),
            ),
            end: vector_of(end_x, end_y),
        },
        colored(GRAY, LineStyle::Dashed),
    ));

    scene.add_step(Step {
        animations: Vec::new(),
        pause: 0.5,
    });
    scene.add_step(animate(
        alpha,
        TURNS * TAU,
        5.0 * TURNS,
        Easing::Linear,
        0.5,
    ));
    scene.add_step(animate(merge, 1.0, 3.0, Easing::Smooth, 2.0));

    export_video(&scene, "demo_merge.mp4", 720, 60)
}
