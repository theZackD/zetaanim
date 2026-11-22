mod common;

use std::io;

use common::{circumcircle_and_incircle, free_point, save, triangle_sides};
use zl_scene::{LineStyle, Scene};

fn main() -> io::Result<()> {
    let mut scene = Scene::new();
    let a = scene.add(free_point("A", -2.5, -1.5));
    let b = scene.add(free_point("B", 2.5, -1.5));
    let c = scene.add(free_point("C", 0.0, 2.5));
    triangle_sides(&mut scene, a, b, c);
    circumcircle_and_incircle(&mut scene, [a, b, c], LineStyle::Solid, LineStyle::Solid);
    save(&scene, "isosceles_circles.png")
}
