use zl_primitives::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LineStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
mod tests {
    use super::*;

    #[test]
    fn default_style_follows_the_canvas_foreground() {
        let style = ObjectStyle::default();
        assert_eq!(style.color, None);
        assert_eq!(style.width, 4.0);
        assert_eq!(style.line, LineStyle::Solid);
        assert_eq!(style.fill, None);
        assert_eq!(style.point_radius, 8.0);
        assert_eq!(LineStyle::default(), LineStyle::Solid);
    }
}
