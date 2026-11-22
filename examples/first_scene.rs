use std::{io, path::Path};

use glam::DVec3;
use zl_primitives::Color;
use zl_render::render;
use zl_scene::{
    BinaryOp, BuiltInFn, Callee, Construction, Expr, LineStyle, Object, ObjectId, ObjectStyle,
    Origin, Property, Scene, Value,
};

fn object(name: &str, construction: Construction, style: ObjectStyle) -> Object {
    Object {
        name: name.to_string(),
        origin: Origin::Script,
        construction,
        style,
    }
}

fn free_point(name: &str, x: f64, y: f64) -> Object {
    let construction = Construction::Point {
        position: Expr::Constant(Value::Vector(DVec3::new(x, y, 0.0))),
    };
    object(name, construction, ObjectStyle::default())
}

fn position(id: ObjectId) -> Expr {
    Expr::Reference {
        object: id,
        property: Property::Position,
    }
}

fn segment(name: &str, a: ObjectId, b: ObjectId, style: ObjectStyle) -> Object {
    object(
        name,
        Construction::Segment {
            start: position(a),
            end: position(b),
        },
        style,
    )
}

fn main() -> io::Result<()> {
    let mut scene = Scene::new();
    let id_a = scene.add(free_point("A", -2.5, -0.5));
    let id_b = scene.add(free_point("B", 2.5, -0.5));
    let id_c = scene.add(free_point("C", 1.0, 3.0));

    scene.add(segment("[AB]", id_a, id_b, ObjectStyle::default()));
    scene.add(segment("[BC]", id_b, id_c, ObjectStyle::default()));
    scene.add(segment("[CA]", id_c, id_a, ObjectStyle::default()));

    let sum = Expr::Binary {
        op: BinaryOp::Add,
        left: Box::new(position(id_a)),
        right: Box::new(position(id_b)),
    };
    let half = Expr::Binary {
        op: BinaryOp::Div,
        left: Box::new(sum),
        right: Box::new(Expr::Constant(Value::Real(2.0))),
    };
    let id_m = scene.add(object(
        "M",
        Construction::Point { position: half },
        ObjectStyle::default(),
    ));

    let radius = Expr::Call {
        callee: Callee::BuiltIn(BuiltInFn::Norm),
        args: vec![Expr::Binary {
            op: BinaryOp::Sub,
            left: Box::new(position(id_b)),
            right: Box::new(position(id_m)),
        }],
    };
    let blue = ObjectStyle {
        color: Some(Color {
            r: 40,
            g: 90,
            b: 200,
            a: 255,
        }),
        ..ObjectStyle::default()
    };
    scene.add(object(
        "circle of diameter [AB]",
        Construction::Circle {
            center: position(id_m),
            radius,
        },
        blue,
    ));

    let dashed = ObjectStyle {
        line: LineStyle::Dashed,
        ..ObjectStyle::default()
    };
    scene.add(segment("[CM]", id_c, id_m, dashed));

    let evaluated = scene.evaluate(0.0);
    if !evaluated.blocked.is_empty() {
        eprintln!("blocked objects: {:?}", evaluated.blocked);
    }
    let primitives = scene.primitives(&evaluated);

    let canvas = scene.canvas();
    let image = render(
        &primitives,
        canvas.background,
        canvas.format.width(),
        canvas.format.height(),
        1.0,
    )
    .ok_or(io::Error::other("invalid image size"))?;

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/first_scene.png");
    image.save_png(&path)?;
    println!("{}", path.display());
    Ok(())
}
