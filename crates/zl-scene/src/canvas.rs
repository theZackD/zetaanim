use zl_primitives::Color;

pub const CANVAS_SHORT_SIDE: f64 = 1080.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CanvasFormat {
    #[default]
    Landscape,
    Portrait,
    Classic,
    Square,
}

impl CanvasFormat {
    pub fn width(&self) -> u32 {
        match self {
            CanvasFormat::Landscape => 1920,
            CanvasFormat::Portrait => 1080,
            CanvasFormat::Classic => 1440,
            CanvasFormat::Square => 1080,
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            CanvasFormat::Landscape => 1080,
            CanvasFormat::Portrait => 1920,
            CanvasFormat::Classic => 1080,
            CanvasFormat::Square => 1080,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Canvas {
    pub format: CanvasFormat,
    pub background: Color,
    pub foreground: Color,
}

impl Default for Canvas {
    fn default() -> Self {
        Canvas {
            format: CanvasFormat::default(),
            background: Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            },
            foreground: Color {
                r: 30,
                g: 30,
                b: 30,
                a: 255,
            },
        }
    }
}

#[cfg(test)]
pub(crate) fn all_formats() -> [CanvasFormat; 4] {
    let all = [
        CanvasFormat::Landscape,
        CanvasFormat::Portrait,
        CanvasFormat::Classic,
        CanvasFormat::Square,
    ];
    for format in all {
        match format {
            CanvasFormat::Landscape
            | CanvasFormat::Portrait
            | CanvasFormat::Classic
            | CanvasFormat::Square => {}
        }
    }
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_format_has_a_short_side_of_1080() {
        for format in all_formats() {
            let short_side = format.width().min(format.height());
            assert_eq!(f64::from(short_side), CANVAS_SHORT_SIDE, "{format:?}");
        }
    }

    #[test]
    fn format_sizes() {
        let sizes: Vec<(u32, u32)> = all_formats()
            .iter()
            .map(|format| (format.width(), format.height()))
            .collect();
        assert_eq!(
            sizes,
            vec![(1920, 1080), (1080, 1920), (1440, 1080), (1080, 1080)]
        );
    }

    #[test]
    fn default_canvas_is_landscape_dark_on_white() {
        let canvas = Canvas::default();
        assert_eq!(canvas.format, CanvasFormat::Landscape);
        assert_eq!(
            canvas.background,
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 255
            }
        );
        assert_eq!(
            canvas.foreground,
            Color {
                r: 30,
                g: 30,
                b: 30,
                a: 255
            }
        );
    }
}
