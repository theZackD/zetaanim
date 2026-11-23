use kurbo::{Circle, Line, Shape};
use zl_primitives::{COORDINATE_LIMIT, Color, Dash, Primitive, Stroke, Style};

use crate::{Camera, Canvas, LineStyle, ObjectStyle, ObjectValue};

const TOLERANCE: f64 = 0.1;
pub(crate) const LAYER_COUNT: u8 = 3;

fn dash(line: LineStyle, width: f64) -> Option<Dash> {
    match line {
        LineStyle::Solid => None,
        LineStyle::Dashed => Some(Dash {
            on: 4.0 * width,
            off: 3.0 * width,
        }),
        LineStyle::Dotted => Some(Dash {
            on: width,
            off: 2.0 * width,
        }),
    }
}

fn stroke(style: &ObjectStyle, color: Color) -> Option<Stroke> {
    if !(style.width > 0.0 && style.width <= COORDINATE_LIMIT) {
        return None;
    }
    Some(Stroke {
        color,
        width: style.width,
        dash: dash(style.line, style.width),
    })
}

pub(crate) fn draw_layer(value: &ObjectValue, style: &ObjectStyle) -> Option<u8> {
    match value {
        ObjectValue::Number(_) => None,
        ObjectValue::Circle { .. } if style.fill.is_some() => Some(0),
        ObjectValue::Segment { .. } | ObjectValue::Circle { .. } => Some(1),
        ObjectValue::Point(_) => Some(2),
        ObjectValue::Curve { .. } => Some(1),
    }
}

pub(crate) fn object_primitive(
    value: &ObjectValue,
    style: &ObjectStyle,
    camera: &Camera,
    canvas: &Canvas,
) -> Option<Primitive> {
    let scale = camera.scale()?;
    let color = style.color.unwrap_or(canvas.foreground);
    match value {
        ObjectValue::Number(_) => None,
        ObjectValue::Point(p) => {
            let center = camera.to_canvas(*p, canvas)?;
            if !(style.point_radius > 0.0 && style.point_radius <= COORDINATE_LIMIT) {
                return None;
            }
            let path = Circle::new((center.x, center.y), style.point_radius).to_path(TOLERANCE);
            Some(Primitive::Path {
                path,
                style: Style {
                    stroke: None,
                    fill: Some(color),
                },
            })
        }
        ObjectValue::Segment { start, end } => {
            let s = camera.to_canvas(*start, canvas)?;
            let e = camera.to_canvas(*end, canvas)?;
            let stroke = stroke(style, color)?;
            let path = Line::new((s.x, s.y), (e.x, e.y)).to_path(TOLERANCE);
            Some(Primitive::Path {
                path,
                style: Style {
                    stroke: Some(stroke),
                    fill: None,
                },
            })
        }

        ObjectValue::Circle { center, radius } => {
            let c = camera.to_canvas(*center, canvas)?;
            let r = radius * scale;
            if !(r > 0.0 && r <= COORDINATE_LIMIT) {
                return None;
            }
            let stroke = stroke(style, color);
            if style.fill.is_none() && stroke.is_none() {
                return None;
            }
            Some(Primitive::Path {
                path: Circle::new((c.x, c.y), r).to_path(TOLERANCE),
                style: Style {
                    stroke,
                    fill: style.fill,
                },
            })
        }

        ObjectValue::Curve { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::DVec3;

    #[test]
    fn every_layer_is_below_the_layer_count() {
        let plain = ObjectStyle::default();
        let filled = ObjectStyle {
            fill: Some(Color {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            }),
            ..ObjectStyle::default()
        };
        let circle = ObjectValue::Circle {
            center: DVec3::ZERO,
            radius: 1.0,
        };
        let cases = [
            (ObjectValue::Number(crate::Value::Real(1.0)), plain, None),
            (circle.clone(), filled, Some(0)),
            (
                ObjectValue::Segment {
                    start: DVec3::ZERO,
                    end: DVec3::X,
                },
                plain,
                Some(1),
            ),
            (circle, plain, Some(1)),
            (ObjectValue::Point(DVec3::ZERO), plain, Some(2)),
            (ObjectValue::Curve { from: 0.0, to: 1.0 }, plain, Some(1)),
        ];
        for (value, style, expected) in cases {
            let layer = draw_layer(&value, &style);
            assert_eq!(layer, expected, "{value:?}");
            assert!(layer.is_none_or(|layer| layer < LAYER_COUNT), "{value:?}");
        }
    }
}
