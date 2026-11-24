mod common;

use std::{f64::consts::TAU, io};

use common::*;
use zl_scene::{BuiltInFn, Expr, LineStyle, Scene};

fn x() -> Expr {
    variable("x")
}

fn usual_functions(width: f64) -> Scene {
    let mut scene = Scene::new();
    axes(&mut scene, width);
    let functions = [
        (
            "exp",
            call(BuiltInFn::Exp, vec![x()]),
            ORANGE,
            LineStyle::Solid,
        ),
        (
            "ln",
            call(BuiltInFn::Ln, vec![x()]),
            GREEN,
            LineStyle::Dotted,
        ),
        ("1/x", div(real(1.0), x()), BLUE, LineStyle::Solid),
        (
            "sin",
            call(BuiltInFn::Sin, vec![x()]),
            PURPLE,
            LineStyle::Dashed,
        ),
    ];
    for (name, expression, color, line) in functions {
        scene.add(graph(
            name,
            expression,
            -8.0,
            8.0,
            styled(color, line, width),
        ));
    }
    scene
}

fn trigonometry(width: f64) -> Scene {
    let mut scene = Scene::new();
    axes(&mut scene, width);
    let functions = [
        ("sin", BuiltInFn::Sin, BLUE, LineStyle::Solid),
        ("cos", BuiltInFn::Cos, ORANGE, LineStyle::Dashed),
        ("tan", BuiltInFn::Tan, GREEN, LineStyle::Solid),
    ];
    for (name, f, color, line) in functions {
        scene.add(graph(
            name,
            call(f, vec![x()]),
            -8.0,
            8.0,
            styled(color, line, width),
        ));
    }
    scene
}

fn sine_family(width: f64) -> Scene {
    let mut scene = Scene::new();
    axes(&mut scene, width);
    let b = scene.add(number("b", 1.5));
    let members = [
        ("a1", 1.0, BLUE, LineStyle::Solid),
        ("a2", 2.0, ORANGE, LineStyle::Dashed),
        ("a3", 3.0, GREEN, LineStyle::Dotted),
    ];
    for (name, amplitude, color, line) in members {
        let a = scene.add(number(name, amplitude));
        let wave = mul(value(a), call(BuiltInFn::Sin, vec![mul(value(b), x())]));
        scene.add(graph(
            &format!("{name} sin(b x)"),
            wave,
            -8.0,
            8.0,
            styled(color, line, width),
        ));
    }
    let t = || variable("t");
    let position = Expr::Vector {
        x: Box::new(mul(real(2.0), call(BuiltInFn::Cos, vec![t()]))),
        y: Box::new(mul(real(2.0), call(BuiltInFn::Sin, vec![t()]))),
        z: Box::new(real(0.0)),
    };
    scene.add(curve(
        "circle",
        position,
        0.0,
        TAU,
        styled(PURPLE, LineStyle::Solid, width),
    ));
    scene
}

fn main() -> io::Result<()> {
    for width in [1.0, 2.0] {
        save(&usual_functions(width), &format!("graphs_{width}px.png"))?;
        save(&trigonometry(width), &format!("trigonometry_{width}px.png"))?;
        save(&sine_family(width), &format!("sine_family_{width}px.png"))?;
    }
    Ok(())
}
