mod common;

use std::io;

use common::{
    BLUE, add, call, colored, div, free_point, object, position, real, save, segment, sub,
    triangle_sides,
};
use zl_scene::{BuiltInFn, Construction, LineStyle, ObjectStyle, Scene};

fn main() -> io::Result<()> {
    let mut scene = Scene::new();
    let a = scene.add(free_point("A", -2.5, -0.5));
    let b = scene.add(free_point("B", 2.5, -0.5));
    let c = scene.add(free_point("C", 1.0, 3.0));
    triangle_sides(&mut scene, a, b, c);

    let m = scene.add(object(
        "M",
        Construction::Point {
            position: div(add(position(a), position(b)), real(2.0)),
        },
        ObjectStyle::default(),
    ));
    scene.add(object(
        "circle of diameter [AB]",
        Construction::Circle {
            center: position(m),
            radius: call(BuiltInFn::Norm, vec![sub(position(b), position(m))]),
        },
        colored(BLUE, LineStyle::Solid),
    ));
    let dashed = ObjectStyle {
        line: LineStyle::Dashed,
        ..ObjectStyle::default()
    };
    scene.add(segment("[CM]", c, m, dashed));

    save(&scene, "first_scene.png")
}
