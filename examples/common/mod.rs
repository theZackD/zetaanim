#![allow(dead_code)]

use std::{
    io::{self, ErrorKind},
    path::Path,
};

use glam::DVec3;
use zl_export::{Video, frame_count, frame_time};
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

pub const GREEN: Color = Color {
    r: 40,
    g: 150,
    b: 80,
    a: 255,
};

pub const PURPLE: Color = Color {
    r: 140,
    g: 60,
    b: 170,
    a: 255,
};

pub const GRAY: Color = Color {
    r: 170,
    g: 170,
    b: 170,
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

pub fn vector_of(x: Expr, y: Expr) -> Expr {
    Expr::Vector {
        x: Box::new(x),
        y: Box::new(y),
        z: Box::new(real(0.0)),
    }
}

pub fn point_at(name: &str, x: Expr, y: Expr, style: ObjectStyle) -> Object {
    object(
        name,
        Construction::Point {
            position: vector_of(x, y),
        },
        style,
    )
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

pub fn variable(name: &str) -> Expr {
    Expr::Variable(name.to_string())
}

pub fn number(name: &str, x: f64) -> Object {
    object(
        name,
        Construction::Number { value: real(x) },
        ObjectStyle::default(),
    )
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

pub fn graph(name: &str, expression: Expr, from: f64, to: f64, style: ObjectStyle) -> Object {
    object(
        name,
        Construction::Graph {
            variable: "x".to_string(),
            expression,
            from: real(from),
            to: real(to),
        },
        style,
    )
}

pub fn curve(name: &str, position: Expr, from: f64, to: f64, style: ObjectStyle) -> Object {
    object(
        name,
        Construction::Curve {
            variable: "t".to_string(),
            position,
            from: real(from),
            to: real(to),
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

pub fn styled(color: Color, line: LineStyle, width: f64) -> ObjectStyle {
    ObjectStyle {
        width,
        ..colored(color, line)
    }
}

const TICK: f64 = 0.08;

pub fn axis_line(scene: &mut Scene, name: &str, start: (f64, f64), end: (f64, f64), width: f64) {
    let construction = Construction::Segment {
        start: vector(start.0, start.1, 0.0),
        end: vector(end.0, end.1, 0.0),
    };
    scene.add(object(
        name,
        construction,
        styled(GRAY, LineStyle::Solid, width),
    ));
}

pub fn axes(scene: &mut Scene, width: f64) {
    axis_line(scene, "x axis", (-100.0, 0.0), (100.0, 0.0), width);
    axis_line(scene, "y axis", (0.0, -100.0), (0.0, 100.0), width);
    for k in (-7..=7).filter(|&k| k != 0) {
        let k = f64::from(k);
        axis_line(scene, "x tick", (k, -TICK), (k, TICK), width);
    }
    for k in (-4..=4).filter(|&k| k != 0) {
        let k = f64::from(k);
        axis_line(scene, "y tick", (-TICK, k), (TICK, k), width);
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

pub fn export_video(scene: &Scene, file_name: &str, short_side: u32, fps: u32) -> io::Result<()> {
    let canvas = scene.canvas();
    if canvas.background.a != 255 {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            "an MP4 has no transparency: the background must be opaque",
        ));
    }
    let canvas_width = f64::from(canvas.format.width());
    let canvas_height = f64::from(canvas.format.height());
    let scale = f64::from(short_side) / canvas_width.min(canvas_height);
    let even = |side: f64| (side * scale / 2.0).round() as u32 * 2;
    let (width, height) = (even(canvas_width), even(canvas_height));
    let duration = scene.duration();
    let frames = frame_count(duration, fps);
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(file_name);
    let mut video = Video::open(Path::new("ffmpeg"), &path, width, height, fps)?;
    for index in 0..frames {
        let evaluated = scene.evaluate(frame_time(index, fps, duration));
        let primitives = scene.primitives(&evaluated);
        let image = render(&primitives, canvas.background, width, height, scale)
            .ok_or(io::Error::other("invalid image size"))?;
        video.write_frame(&image.to_rgba())?;
    }
    video.finish()?;
    println!("{}", path.display());
    Ok(())
}
