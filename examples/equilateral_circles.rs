mod common;

use std::io;

use common::{
    add, call, circumcircle_and_incircle, div, free_point, mul, object, position, real, save, sub,
    triangle_sides, vector,
};
use zl_scene::{BuiltInFn, Construction, LineStyle, ObjectStyle, Scene};

fn main() -> io::Result<()> {
    let mut scene = Scene::new();
    let a = scene.add(free_point("A", -2.5, -1.5));
    let b = scene.add(free_point("B", 2.5, -1.5));
    let apex = add(
        div(add(position(a), position(b)), real(2.0)),
        mul(
            real(3.0_f64.sqrt() / 2.0),
            call(
                BuiltInFn::Cross,
                vec![vector(0.0, 0.0, 1.0), sub(position(b), position(a))],
            ),
        ),
    );
    let c = scene.add(object(
        "C",
        Construction::Point { position: apex },
        ObjectStyle::default(),
    ));
    triangle_sides(&mut scene, a, b, c);
    circumcircle_and_incircle(&mut scene, [a, b, c], LineStyle::Solid, LineStyle::Solid);
    save(&scene, "equilateral_circles.png")
}
