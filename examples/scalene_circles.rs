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

fn value(id: ObjectId) -> Expr {
    Expr::Reference {
        object: id,
        property: Property::Value,
    }
}

fn real(x: f64) -> Expr {
    Expr::Constant(Value::Real(x))
}

fn binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
    Expr::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
    }
}

fn add(left: Expr, right: Expr) -> Expr {
    binary(BinaryOp::Add, left, right)
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

fn call(f: BuiltInFn, args: Vec<Expr>) -> Expr {
    Expr::Call {
        callee: Callee::BuiltIn(f),
        args,
    }
}

fn distance(p: ObjectId, q: ObjectId) -> Expr {
    call(BuiltInFn::Norm, vec![sub(position(p), position(q))])
}

fn segment(name: &str, a: ObjectId, b: ObjectId) -> Object {
    object(
        name,
        Construction::Segment {
            start: position(a),
            end: position(b),
        },
        ObjectStyle::default(),
    )
}

fn colored(color: Color, line: LineStyle) -> ObjectStyle {
    ObjectStyle {
        color: Some(color),
        line,
        ..ObjectStyle::default()
    }
}

fn main() -> io::Result<()> {
    let blue = Color {
        r: 40,
        g: 90,
        b: 200,
        a: 255,
    };
    let orange = Color {
        r: 220,
        g: 110,
        b: 30,
        a: 255,
    };

    let mut scene = Scene::new();
    let a = scene.add(free_point("A", -3.0, -1.6));
    let b = scene.add(free_point("B", 2.6, -2.0));
    let c = scene.add(free_point("C", 0.6, 2.4));

    scene.add(segment("[AB]", a, b));
    scene.add(segment("[BC]", b, c));
    scene.add(segment("[CA]", c, a));

    let side_a = scene.add(object(
        "a = BC",
        Construction::Number {
            value: distance(b, c),
        },
        ObjectStyle::default(),
    ));
    let side_b = scene.add(object(
        "b = CA",
        Construction::Number {
            value: distance(c, a),
        },
        ObjectStyle::default(),
    ));
    let side_c = scene.add(object(
        "c = AB",
        Construction::Number {
            value: distance(a, b),
        },
        ObjectStyle::default(),
    ));
    let perimeter = || add(add(value(side_a), value(side_b)), value(side_c));

    let u = || sub(position(b), position(a));
    let v = || sub(position(c), position(a));
    let w = || call(BuiltInFn::Cross, vec![u(), v()]);
    let squared = |e: Expr| call(BuiltInFn::Dot, vec![e.clone(), e]);

    let circumcenter = add(
        position(a),
        div(
            call(
                BuiltInFn::Cross,
                vec![sub(mul(squared(u()), v()), mul(squared(v()), u())), w()],
            ),
            mul(real(2.0), squared(w())),
        ),
    );
    let o = scene.add(object(
        "O",
        Construction::Point {
            position: circumcenter,
        },
        colored(blue, LineStyle::Solid),
    ));
    scene.add(object(
        "circumcircle",
        Construction::Circle {
            center: position(o),
            radius: distance(a, o),
        },
        colored(blue, LineStyle::Dashed),
    ));

    let incenter = div(
        add(
            add(
                mul(value(side_a), position(a)),
                mul(value(side_b), position(b)),
            ),
            mul(value(side_c), position(c)),
        ),
        perimeter(),
    );
    let i = scene.add(object(
        "I",
        Construction::Point { position: incenter },
        colored(orange, LineStyle::Solid),
    ));
    let inradius = div(call(BuiltInFn::Norm, vec![w()]), perimeter());
    scene.add(object(
        "incircle",
        Construction::Circle {
            center: position(i),
            radius: inradius,
        },
        colored(orange, LineStyle::Dotted),
    ));

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

    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/scalene_circles.png");
    image.save_png(&path)?;
    println!("{}", path.display());
    Ok(())
}
