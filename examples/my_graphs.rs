mod common;

use std::io;

use common::*;
use zl_scene::{BuiltInFn, LineStyle, Scene};

fn main() -> io::Result<()> {
    let x = || variable("x");
    let mut scene = Scene::new();
    axes(&mut scene, 2.0);
    scene.add(graph(
        "f",
        add(add(mul(x(), x()), mul(real(4.0), x())), real(4.0)),
        -8.0,
        8.0,
        colored(BLUE, LineStyle::Solid),
    ));
    scene.add(graph(
        "g",
        call(BuiltInFn::Sqrt, vec![x()]),
        -8.0,
        8.0,
        colored(GREEN, LineStyle::Dashed),
    ));
    save(&scene, "my_graphs.png")
}
