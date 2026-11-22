use glam::DVec3;
use zl_primitives::Color;
use zl_render::{Image, render};
use zl_scene::{
    Construction, Expr, LineStyle, Object, ObjectId, ObjectStyle, Origin, Property, Scene, Value,
};

const FOREGROUND: Color = Color {
    r: 30,
    g: 30,
    b: 30,
    a: 255,
};
const WHITE: Color = Color {
    r: 255,
    g: 255,
    b: 255,
    a: 255,
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
    object(
        name,
        Construction::Point {
            position: Expr::Constant(Value::Vector(DVec3::new(x, y, 0.0))),
        },
        ObjectStyle::default(),
    )
}

fn position(id: ObjectId) -> Expr {
    Expr::Reference {
        object: id,
        property: Property::Position,
    }
}

fn dashed_segment_scene() -> Scene {
    let mut scene = Scene::new();
    let a = scene.add(free_point("A", -2.0, 0.0));
    let b = scene.add(free_point("B", 2.0, 0.0));
    scene.add(object(
        "[AB]",
        Construction::Segment {
            start: position(a),
            end: position(b),
        },
        ObjectStyle {
            line: LineStyle::Dashed,
            ..ObjectStyle::default()
        },
    ));
    scene
}

fn draw(scene: &Scene) -> Image {
    let evaluated = scene.evaluate(0.0);
    assert!(evaluated.blocked.is_empty());
    let primitives = scene.primitives(&evaluated);
    let canvas = scene.canvas();
    render(
        &primitives,
        canvas.background,
        canvas.format.width(),
        canvas.format.height(),
        1.0,
    )
    .expect("the default canvas has a valid size")
}

#[test]
fn image_has_the_size_of_the_canvas() {
    let image = draw(&dashed_segment_scene());
    assert_eq!((image.width(), image.height()), (1920, 1080));
}

#[test]
fn points_are_drawn_where_the_camera_puts_them() {
    let image = draw(&dashed_segment_scene());
    assert_eq!(image.pixel(690, 540), Some(FOREGROUND));
    assert_eq!(image.pixel(1230, 540), Some(FOREGROUND));
}

#[test]
fn background_fills_the_rest_of_the_canvas() {
    let image = draw(&dashed_segment_scene());
    for (x, y) in [(0, 0), (1919, 0), (0, 1079), (1919, 1079), (960, 300)] {
        assert_eq!(image.pixel(x, y), Some(WHITE), "pixel ({x}, {y})");
    }
}

#[test]
fn dashed_segment_alternates_dashes_and_gaps() {
    let image = draw(&dashed_segment_scene());
    for y in [539, 540] {
        assert_eq!(image.pixel(698, y), Some(FOREGROUND), "first dash, row {y}");
        assert_eq!(image.pixel(712, y), Some(WHITE), "first gap, row {y}");
        assert_eq!(
            image.pixel(726, y),
            Some(FOREGROUND),
            "second dash, row {y}"
        );
        assert_eq!(image.pixel(740, y), Some(WHITE), "second gap, row {y}");
    }
}
