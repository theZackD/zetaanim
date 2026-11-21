use kurbo::BezPath;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dash {
    pub on: f64,
    pub off: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stroke {
    pub color: Color,
    pub width: f64,
    pub dash: Option<Dash>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Style {
    pub stroke: Option<Stroke>,
    pub fill: Option<Color>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    Path { path: BezPath, style: Style },
}
