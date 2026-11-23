use std::{io, path::Path};

use kurbo::{BezPath, PathEl};
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, StrokeDash, Transform};
use zl_primitives::{COORDINATE_LIMIT, Color, Primitive, Stroke};

const LIMIT: f32 = COORDINATE_LIMIT as f32;

pub struct Image {
    pixmap: Pixmap,
}

impl Image {
    pub fn width(&self) -> u32 {
        self.pixmap.width()
    }

    pub fn height(&self) -> u32 {
        self.pixmap.height()
    }

    pub fn pixel(&self, x: u32, y: u32) -> Option<Color> {
        if x >= self.pixmap.width() || y >= self.pixmap.height() {
            return None;
        }

        let c = self.pixmap.pixel(x, y)?.demultiply();
        Some(Color {
            r: c.red(),
            g: c.green(),
            b: c.blue(),
            a: c.alpha(),
        })
    }

    pub fn to_rgba(&self) -> Vec<u8> {
        self.pixmap.clone().take_demultiplied()
    }

    pub fn save_png(&self, path: &Path) -> io::Result<()> {
        self.pixmap.save_png(path).map_err(io::Error::other)
    }
}

pub fn render(
    primitives: &[Primitive],
    background: Color,
    width: u32,
    height: u32,
    scale: f64,
) -> Option<Image> {
    let scale = scale as f32;
    if !(scale > 0.0 && scale.is_finite()) {
        return None;
    }

    let mut pixmap = Pixmap::new(width, height)?;
    pixmap.fill(tiny_skia::Color::from_rgba8(
        background.r,
        background.g,
        background.b,
        background.a,
    ));

    let transform = Transform::from_scale(scale, scale);
    for p in primitives {
        match p {
            Primitive::Path { path, style } => {
                if let Some(sk_path) = skia_path(path)
                    && fits(&sk_path, scale)
                {
                    if let Some(color) = style.fill {
                        pixmap.fill_path(
                            &sk_path,
                            &paint(color),
                            FillRule::Winding,
                            transform,
                            None,
                        );
                    }
                    if let Some(stroke) = style.stroke
                        && let Some(sk_stroke) = skia_stroke(&stroke, scale)
                    {
                        pixmap.stroke_path(
                            &sk_path,
                            &paint(stroke.color),
                            &sk_stroke,
                            transform,
                            None,
                        );
                    }
                }
            }
        }
    }

    Some(Image { pixmap })
}

fn skia_path(path: &BezPath) -> Option<tiny_skia::Path> {
    let mut builder = PathBuilder::new();
    for element in path.elements() {
        match element {
            PathEl::MoveTo(p) => builder.move_to(p.x as f32, p.y as f32),
            PathEl::LineTo(p) => builder.line_to(p.x as f32, p.y as f32),
            PathEl::QuadTo(p1, p2) => {
                builder.quad_to(p1.x as f32, p1.y as f32, p2.x as f32, p2.y as f32)
            }
            PathEl::CurveTo(p1, p2, p3) => builder.cubic_to(
                p1.x as f32,
                p1.y as f32,
                p2.x as f32,
                p2.y as f32,
                p3.x as f32,
                p3.y as f32,
            ),
            PathEl::ClosePath => builder.close(),
        }
    }
    builder.finish()
}

fn fits(path: &tiny_skia::Path, scale: f32) -> bool {
    let bounds = path.bounds();
    bounds.left().abs() * scale <= LIMIT
        && bounds.top().abs() * scale <= LIMIT
        && bounds.right().abs() * scale <= LIMIT
        && bounds.bottom().abs() * scale <= LIMIT
}

fn paint(color: Color) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color_rgba8(color.r, color.g, color.b, color.a);
    paint
}

fn skia_stroke(stroke: &Stroke, scale: f32) -> Option<tiny_skia::Stroke> {
    let width = stroke.width as f32;
    if !(width > 0.0 && width.is_finite() && width * scale <= LIMIT) {
        return None;
    }
    let mut s = tiny_skia::Stroke {
        width,
        ..tiny_skia::Stroke::default()
    };
    if let Some(dash) = stroke.dash {
        let on = dash.on as f32;
        let off = dash.off as f32;
        if on > 0.0 && off > 0.0 {
            s.dash = StrokeDash::new(vec![on, off], 0.0);
        }
    }
    Some(s)
}

#[cfg(test)]
mod tests;
