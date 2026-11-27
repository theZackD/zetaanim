use serde::{Deserialize, Serialize};
use zl_primitives::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum LineStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectStyle {
    pub color: Option<Color>,
    pub width: f64,
    pub line: LineStyle,
    pub fill: Option<Color>,
    pub point_radius: f64,
}

impl Default for ObjectStyle {
    fn default() -> Self {
        ObjectStyle {
            color: None,
            width: 4.0,
            line: LineStyle::default(),
            fill: None,
            point_radius: 8.0,
        }
    }
}

#[cfg(test)]
mod tests;
