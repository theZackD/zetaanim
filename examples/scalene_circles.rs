mod common;

use std::io;

use common::{circumcircle_and_incircle, free_point, save, triangle_sides};
use zl_scene::{LineStyle, Scene};

fn main() -> io::Result<()> {
    let mut scene = Scene::new();
    let a = scene.add(free_point("A", -3.0, -1.6));
    let b = scene.add(free_point("B", 2.6, -2.0));
    let c = scene.add(free_point("C", 0.6, 2.4));
    triangle_sides(&mut scene, a, b, c);
    circumcircle_and_incircle(&mut scene, [a, b, c], LineStyle::Dashed, LineStyle::Dotted);
    save(&scene, "scalene_circles.png")
}
