use glam::DVec3;

#[derive(Clone, Debug, PartialEq)]
pub enum Origin {
    Mouse,
    Script,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Construction {
    FreePoint { position: DVec3 },
    Number { value: f64 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    pub name: String,
    pub origin: Origin,
    pub construction: Construction,
}
