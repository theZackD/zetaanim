#![allow(dead_code)]

use std::{io, path::Path};

use glam::DVec3;
use zl_primitives::Color;
use zl_render::render;
use zl_scene::{
    BinaryOp, BuiltInFn, Callee, Construction, Expr, LineStyle, Object, ObjectId, ObjectStyle,
    Origin, Property, Scene, Value,
};

pub const BLUE: Color = Color {
    r: 40,
    g: 90,
    b: 200,
    a: 255,
};

pub const ORANGE: Color = Color {
    r: 220,
    g: 110,
    b: 30,
    a: 255,
};

pub fn object(name: &str, construction: Construction, style: ObjectStyle) -> Object {
    Object {
        name: name.to_string(),
        origin: Origin::Script,
        construction,
        style,
    }
}

pub fn free_point(name: &str, x: f64, y: f64) -> Object {
    let construction = Construction::Point {
        position: Expr::Constant(Value::Vector(DVec3::new(x, y, 0.0))),
    };
    object(name, construction, ObjectStyle::default())
}

pub fn position(id: ObjectId) -> Expr {
    Expr::Reference {
        object: id,
        property: Property::Position,
    }
}

pub fn value(id: ObjectId) -> Expr {
    Expr::Reference {
        object: id,
        property: Property::Value,
    }
}

pub fn real(x: f64) -> Expr {
    Expr::Constant(Value::Real(x))
}

pub fn vector(x: f64, y: f64, z: f64) -> Expr {
    Expr::Constant(Value::Vector(DVec3::new(x, y, z)))
}

pub fn binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
    Expr::Binary {
        op,
        left: Box::new(left),
        right: Box::new(right),
    }
}

pub fn add(left: Expr, right: Expr) -> Expr {
    binary(BinaryOp::Add, left, right)
}

pub fn sub(left: Expr, right: Expr) -> Expr {
    binary(BinaryOp::Sub, left, right)
}

pub fn mul(left: Expr, right: Expr) -> Expr {
    binary(BinaryOp::Mul, left, right)
}

pub fn div(left: Expr, right: Expr) -> Expr {
    binary(BinaryOp::Div, left, right)
}

pub fn call(f: BuiltInFn, args: Vec<Expr>) -> Expr {
    Expr::Call {
        callee: Callee::BuiltIn(f),
        args,
    }
}

pub fn distance(p: ObjectId, q: ObjectId) -> Expr {
    call(BuiltInFn::Norm, vec![sub(position(p), position(q))])
}

pub fn segment(name: &str, a: ObjectId, b: ObjectId, style: ObjectStyle) -> Object {
    object(
        name,
        Construction::Segment {
            start: position(a),
            end: position(b),
        },
        style,
    )
}

pub fn colored(color: Color, line: LineStyle) -> ObjectStyle {
    ObjectStyle {
        color: Some(color),
        line,
        ..ObjectStyle::default()
    }
}

pub fn triangle_sides(scene: &mut Scene, a: ObjectId, b: ObjectId, c: ObjectId) {
    scene.add(segment("[AB]", a, b, ObjectStyle::default()));
    scene.add(segment("[BC]", b, c, ObjectStyle::default()));
    scene.add(segment("[CA]", c, a, ObjectStyle::default()));
}

pub fn circumcircle_and_incircle(
    scene: &mut Scene,
    [a, b, c]: [ObjectId; 3],
    circumcircle_line: LineStyle,
    incircle_line: LineStyle,
) {
    let length = |scene: &mut Scene, name: &str, p, q| {
        scene.add(object(
            name,
            Construction::Number {
                value: distance(p, q),
            },
            ObjectStyle::default(),
        ))
    };
    let side_a = length(scene, "a = BC", b, c);
    let side_b = length(scene, "b = CA", c, a);
    let side_c = length(scene, "c = AB", a, b);
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
        colored(BLUE, LineStyle::Solid),
    ));
    scene.add(object(
        "circumcircle",
        Construction::Circle {
            center: position(o),
            radius: distance(a, o),
        },
        colored(BLUE, circumcircle_line),
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
        colored(ORANGE, LineStyle::Solid),
    ));
    scene.add(object(
        "incircle",
        Construction::Circle {
            center: position(i),
            radius: div(call(BuiltInFn::Norm, vec![w()]), perimeter()),
        },
        colored(ORANGE, incircle_line),
    ));
}

pub fn save(scene: &Scene, file_name: &str) -> io::Result<()> {
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
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(file_name);
    image.save_png(&path)?;
    println!("{}", path.display());
    Ok(())
}
