use std::{io, path::Path};

use kurbo::{BezPath, PathEl};
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, StrokeDash, Transform};
use zl_primitives::{Color, Primitive, Stroke};

const LIMIT: f32 = 1e7;

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
mod tests {
    use super::*;
    use kurbo::{Circle, Line, Rect, Shape};
    use zl_primitives::{Dash, Style};

    const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    const RED: Color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    const BLUE: Color = Color {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
    const CLEAR: Color = Color {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    fn filled(path: BezPath, color: Color) -> Primitive {
        Primitive::Path {
            path,
            style: Style {
                stroke: None,
                fill: Some(color),
            },
        }
    }

    fn stroked(path: BezPath, width: f64, dash: Option<Dash>) -> Primitive {
        Primitive::Path {
            path,
            style: Style {
                stroke: Some(Stroke {
                    color: RED,
                    width,
                    dash,
                }),
                fill: None,
            },
        }
    }

    fn disc(x: f64, y: f64, radius: f64, color: Color) -> Primitive {
        filled(Circle::new((x, y), radius).to_path(0.1), color)
    }

    fn horizontal_line(width: f64, dash: Option<Dash>) -> Primitive {
        stroked(
            Line::new((0.0, 50.5), (100.0, 50.5)).to_path(0.1),
            width,
            dash,
        )
    }

    fn triangle(far_x: f64, far_y: f64) -> BezPath {
        let mut path = BezPath::new();
        path.move_to((10.0, 10.0));
        path.line_to((90.0, 10.0));
        path.line_to((far_x, far_y));
        path.close_path();
        path
    }

    fn draw(primitives: &[Primitive]) -> Image {
        render(primitives, WHITE, 100, 100, 1.0).expect("valid size and scale")
    }

    fn red_pixels_on_row(image: &Image, y: u32) -> usize {
        (5..95).filter(|x| image.pixel(*x, y) == Some(RED)).count()
    }

    fn assert_close(actual: Color, expected: Color, tolerance: u8) {
        let channels = [
            (actual.r, expected.r),
            (actual.g, expected.g),
            (actual.b, expected.b),
            (actual.a, expected.a),
        ];
        for (a, e) in channels {
            assert!(a.abs_diff(e) <= tolerance, "{actual:?} != {expected:?}");
        }
    }

    #[test]
    fn background_fills_every_pixel() {
        let color = Color {
            r: 10,
            g: 20,
            b: 30,
            a: 255,
        };
        let image = render(&[], color, 4, 3, 1.0).expect("valid size and scale");
        assert_eq!((image.width(), image.height()), (4, 3));
        for y in 0..3 {
            for x in 0..4 {
                assert_eq!(image.pixel(x, y), Some(color), "pixel ({x}, {y})");
            }
        }
    }

    #[test]
    fn pixel_outside_the_image_is_none() {
        let image = render(&[], WHITE, 4, 3, 1.0).expect("valid size and scale");
        assert_eq!(image.pixel(3, 2), Some(WHITE));
        assert_eq!(image.pixel(4, 0), None);
        assert_eq!(image.pixel(0, 3), None);
        assert_eq!(image.pixel(u32::MAX, u32::MAX), None);
    }

    #[test]
    fn zero_size_gives_no_image() {
        assert!(render(&[], WHITE, 0, 10, 1.0).is_none());
        assert!(render(&[], WHITE, 10, 0, 1.0).is_none());
    }

    #[test]
    fn invalid_scale_gives_no_image() {
        for scale in [0.0, -1.0, f64::NAN, f64::INFINITY, 1e300, 1e-50] {
            assert!(
                render(&[], WHITE, 10, 10, scale).is_none(),
                "scale = {scale:e}"
            );
        }
    }

    #[test]
    fn to_rgba_is_unpremultiplied_and_row_major() {
        let square = filled(Rect::new(2.0, 1.0, 3.0, 2.0).to_path(0.1), RED);
        let image = render(&[square], WHITE, 4, 3, 1.0).expect("valid size and scale");
        let rgba = image.to_rgba();
        assert_eq!(rgba.len(), 4 * 3 * 4);
        let index = (4 + 2) * 4;
        assert_eq!(&rgba[index..index + 4], &[255, 0, 0, 255]);
        assert_eq!(&rgba[index - 4..index], &[255, 255, 255, 255]);
        let translucent = Color {
            r: 200,
            g: 100,
            b: 50,
            a: 128,
        };
        let rgba = render(&[], translucent, 2, 2, 1.0)
            .expect("valid size and scale")
            .to_rgba();
        let first = Color {
            r: rgba[0],
            g: rgba[1],
            b: rgba[2],
            a: rgba[3],
        };
        assert_close(first, translucent, 1);
    }

    #[test]
    fn transparent_png_round_trip_keeps_its_colors() {
        let translucent = Color {
            r: 200,
            g: 100,
            b: 50,
            a: 128,
        };
        let image = render(&[disc(10.0, 10.0, 8.0, translucent)], CLEAR, 20, 20, 1.0)
            .expect("valid size and scale");
        let path =
            std::env::temp_dir().join(format!("zl_render_round_trip_{}.png", std::process::id()));
        image.save_png(&path).expect("temporary file is writable");
        let reloaded = Pixmap::load_png(&path).expect("file was just written");
        std::fs::remove_file(&path).expect("file was just written");
        let center = image.pixel(10, 10).expect("inside the image");
        assert_close(center, translucent, 2);
        let read = reloaded
            .pixel(10, 10)
            .expect("inside the image")
            .demultiply();
        let read = Color {
            r: read.red(),
            g: read.green(),
            b: read.blue(),
            a: read.alpha(),
        };
        assert_close(read, center, 1);
        assert_eq!(reloaded.pixel(0, 0).expect("inside the image").alpha(), 0);
    }

    #[test]
    fn saving_into_a_missing_directory_is_an_error() {
        let image = draw(&[]);
        let path = std::env::temp_dir()
            .join(format!("zl_render_missing_{}", std::process::id()))
            .join("image.png");
        assert!(image.save_png(&path).is_err());
    }

    #[test]
    fn filled_disc_covers_its_center_and_nothing_far() {
        let image = draw(&[disc(50.0, 50.0, 10.0, RED)]);
        assert_eq!(image.pixel(50, 50), Some(RED));
        assert_eq!(image.pixel(5, 5), Some(WHITE));
        assert_eq!(image.pixel(50, 65), Some(WHITE));
    }

    #[test]
    fn scale_maps_positions_and_widths_to_pixels() {
        let image = render(&[disc(25.0, 25.0, 5.0, RED)], WHITE, 100, 100, 2.0)
            .expect("valid size and scale");
        assert_eq!(image.pixel(50, 50), Some(RED));
        assert_eq!(image.pixel(25, 25), Some(WHITE));
        let line = stroked(Line::new((0.0, 25.0), (50.0, 25.0)).to_path(0.1), 4.0, None);
        let image = render(&[line], WHITE, 100, 100, 2.0).expect("valid size and scale");
        assert_eq!(image.pixel(50, 47), Some(RED));
        assert_eq!(image.pixel(50, 52), Some(RED));
        assert_eq!(image.pixel(50, 44), Some(WHITE));
        assert_eq!(image.pixel(50, 56), Some(WHITE));
    }

    #[test]
    fn stroke_is_drawn_over_fill() {
        let ringed = Primitive::Path {
            path: Circle::new((50.0, 50.0), 10.0).to_path(0.1),
            style: Style {
                stroke: Some(Stroke {
                    color: BLUE,
                    width: 4.0,
                    dash: None,
                }),
                fill: Some(RED),
            },
        };
        let image = draw(&[ringed]);
        assert_eq!(image.pixel(50, 50), Some(RED));
        assert_eq!(image.pixel(50, 40), Some(BLUE));
        assert_eq!(image.pixel(50, 37), Some(WHITE));
    }

    #[test]
    fn later_primitives_are_drawn_on_top() {
        let image = draw(&[disc(50.0, 50.0, 10.0, RED), disc(50.0, 50.0, 5.0, BLUE)]);
        assert_eq!(image.pixel(50, 50), Some(BLUE));
        assert_eq!(image.pixel(50, 42), Some(RED));
    }

    #[test]
    fn dashed_line_has_gaps_and_solid_line_has_none() {
        let solid = draw(&[horizontal_line(3.0, None)]);
        assert_eq!(red_pixels_on_row(&solid, 50), 90);
        let dashed = draw(&[horizontal_line(
            3.0,
            Some(Dash {
                on: 10.0,
                off: 10.0,
            }),
        )]);
        let red = red_pixels_on_row(&dashed, 50);
        assert!((30..60).contains(&red), "{red} red pixels");
    }

    #[test]
    fn dash_length_is_the_drawn_part_and_gap_the_empty_part() {
        let long_dashes = draw(&[horizontal_line(3.0, Some(Dash { on: 15.0, off: 5.0 }))]);
        let red = red_pixels_on_row(&long_dashes, 50);
        assert!((60..80).contains(&red), "{red} red pixels");
        let short_dashes = draw(&[horizontal_line(3.0, Some(Dash { on: 5.0, off: 15.0 }))]);
        let red = red_pixels_on_row(&short_dashes, 50);
        assert!((10..30).contains(&red), "{red} red pixels");
    }

    #[test]
    fn edges_are_anti_aliased() {
        let half_covered = filled(Rect::new(0.0, 0.0, 50.5, 100.0).to_path(0.1), RED);
        let image = draw(&[half_covered]);
        assert_eq!(image.pixel(49, 50), Some(RED));
        assert_eq!(image.pixel(51, 50), Some(WHITE));
        let edge = image.pixel(50, 50).expect("inside the image");
        assert_eq!((edge.r, edge.a), (255, 255));
        assert!((100..160).contains(&edge.g), "{edge:?}");
        assert_eq!(edge.g, edge.b);
    }

    #[test]
    fn invalid_dash_falls_back_to_a_solid_line() {
        let invalid = [
            Dash { on: 0.0, off: 5.0 },
            Dash { on: 5.0, off: 0.0 },
            Dash { on: -1.0, off: 5.0 },
            Dash { on: 5.0, off: -1.0 },
            Dash {
                on: f64::NAN,
                off: 5.0,
            },
            Dash {
                on: 5.0,
                off: f64::NAN,
            },
            Dash {
                on: f64::INFINITY,
                off: 5.0,
            },
            Dash {
                on: 5.0,
                off: 1e300,
            },
            Dash {
                on: 1e-50,
                off: 5.0,
            },
        ];
        for dash in invalid {
            let image = draw(&[horizontal_line(3.0, Some(dash))]);
            assert_eq!(red_pixels_on_row(&image, 50), 90, "{dash:?}");
        }
    }

    #[test]
    fn invalid_stroke_width_draws_no_stroke_but_keeps_the_fill() {
        for width in [0.0, -1.0, f64::NAN, f64::INFINITY, 1e300, 1e-50, 1e10] {
            let image = draw(&[horizontal_line(width, None)]);
            assert_eq!(red_pixels_on_row(&image, 50), 0, "width = {width:e}");
            let ringed = Primitive::Path {
                path: Circle::new((50.0, 50.0), 10.0).to_path(0.1),
                style: Style {
                    stroke: Some(Stroke {
                        color: BLUE,
                        width,
                        dash: None,
                    }),
                    fill: Some(RED),
                },
            };
            assert_eq!(
                draw(&[ringed]).pixel(50, 50),
                Some(RED),
                "width = {width:e}"
            );
        }
    }

    #[test]
    fn invalid_paths_are_skipped_and_the_others_drawn() {
        let not_a_number = stroked(
            Line::new((0.0, f64::NAN), (100.0, 50.0)).to_path(0.1),
            3.0,
            None,
        );
        let infinite = stroked(
            Line::new((0.0, 50.0), (f64::INFINITY, 50.0)).to_path(0.1),
            3.0,
            None,
        );
        let empty = filled(BezPath::new(), BLUE);
        let image = draw(&[not_a_number, infinite, empty, disc(50.0, 20.0, 5.0, RED)]);
        assert_eq!(image.pixel(50, 20), Some(RED));
        assert_eq!(red_pixels_on_row(&image, 50), 0);
    }

    #[test]
    fn degenerate_segment_does_not_panic() {
        let point = stroked(
            Line::new((50.0, 50.0), (50.0, 50.0)).to_path(0.1),
            3.0,
            None,
        );
        draw(&[point]);
    }

    #[test]
    fn primitive_beyond_the_limit_is_skipped_without_panic() {
        let image = draw(&[filled(triangle(-6e8, -6e8), RED)]);
        assert_eq!(image.pixel(50, 5), Some(WHITE));
        let image = draw(&[filled(triangle(-5e6, -5e6), RED)]);
        assert_eq!(image.pixel(50, 5), Some(RED));
    }

    #[test]
    fn limit_takes_the_scale_into_account() {
        let too_far = render(&[filled(triangle(-6e6, -6e6), RED)], WHITE, 100, 100, 2.0)
            .expect("valid size and scale");
        assert_eq!(too_far.pixel(50, 10), Some(WHITE));
        let close_enough = render(&[filled(triangle(-4e6, -4e6), RED)], WHITE, 100, 100, 2.0)
            .expect("valid size and scale");
        assert_eq!(close_enough.pixel(50, 10), Some(RED));
        let line = |width| {
            stroked(
                Line::new((0.0, 25.25), (50.0, 25.25)).to_path(0.1),
                width,
                None,
            )
        };
        let too_thick = render(&[line(6e6)], WHITE, 100, 100, 2.0).expect("valid size and scale");
        assert_eq!(too_thick.pixel(50, 50), Some(WHITE));
        let thick = render(&[line(4e6)], WHITE, 100, 100, 2.0).expect("valid size and scale");
        assert_eq!(thick.pixel(50, 50), Some(RED));
    }

    fn spike(base: [(f64, f64); 2], tip: (f64, f64)) -> BezPath {
        let mut path = BezPath::new();
        path.move_to(base[0]);
        path.line_to(base[1]);
        path.line_to(tip);
        path.close_path();
        path
    }

    #[test]
    fn limit_applies_in_every_direction() {
        let far = 2e7;
        let spikes = [
            ("left", spike([(90.0, 10.0), (90.0, 90.0)], (-far, 50.0))),
            ("right", spike([(10.0, 10.0), (10.0, 90.0)], (far, 50.0))),
            ("up", spike([(10.0, 90.0), (90.0, 90.0)], (50.0, -far))),
            ("down", spike([(10.0, 10.0), (90.0, 10.0)], (50.0, far))),
        ];
        for (direction, path) in spikes {
            let image = draw(&[filled(path, RED)]);
            assert_eq!(image.pixel(50, 50), Some(WHITE), "spike going {direction}");
        }
        let near = draw(&[filled(
            spike([(10.0, 10.0), (10.0, 90.0)], (5e6, 50.0)),
            RED,
        )]);
        assert_eq!(near.pixel(50, 50), Some(RED));
    }
}
