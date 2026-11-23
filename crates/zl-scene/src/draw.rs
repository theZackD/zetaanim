use glam::{DVec2, DVec3};
use kurbo::{BezPath, Circle, Line, Shape};
use zl_primitives::{COORDINATE_LIMIT, Color, Dash, Primitive, Stroke, Style};

use crate::{
    Camera, Canvas, Construction, EvalContext, LineStyle, Object, ObjectStyle, ObjectValue, Value,
};

const TOLERANCE: f64 = 0.1;
const SAMPLES: u32 = 256;
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
    object: &Object,
    value: &ObjectValue,
    camera: &Camera,
    canvas: &Canvas,
    ctx: &mut EvalContext,
) -> Option<Primitive> {
    let style = &object.style;
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

        ObjectValue::Curve { from, to } => {
            let stroke = stroke(style, color)?;
            let pieces = sample_pieces(&object.construction, *from, *to, camera, canvas, ctx);
            if pieces.is_empty() {
                return None;
            }
            let mut path = BezPath::new();
            for piece in &pieces {
                path.move_to((piece[0].x, piece[0].y));
                for p in &piece[1..] {
                    path.line_to((p.x, p.y));
                }
            }
            Some(Primitive::Path {
                path,
                style: Style {
                    stroke: Some(stroke),
                    fill: None,
                },
            })
        }
    }
}

fn sample_point(construction: &Construction, ctx: &mut EvalContext, t: f64) -> Option<DVec3> {
    match construction {
        Construction::Graph {
            variable,
            expression,
            ..
        } => {
            ctx.variables.push((variable.clone(), Value::Real(t)));
            let y = expression.evaluate(ctx);
            ctx.variables.pop();
            Some(DVec3::new(t, y?.as_real()?, 0.0))
        }
        Construction::Curve {
            variable, position, ..
        } => {
            ctx.variables.push((variable.clone(), Value::Real(t)));
            let p = position.evaluate(ctx);
            ctx.variables.pop();
            p?.as_vector()
        }
        Construction::Circle { .. }
        | Construction::Number { .. }
        | Construction::Point { .. }
        | Construction::Segment { .. } => None,
    }
}

fn sample_pieces(
    construction: &Construction,
    from: f64,
    to: f64,
    camera: &Camera,
    canvas: &Canvas,
    ctx: &mut EvalContext,
) -> Vec<Vec<DVec2>> {
    let mut pieces: Vec<Vec<DVec2>> = Vec::new();
    let mut current: Vec<DVec2> = Vec::new();

    for i in 0..=SAMPLES {
        let t = from + (to - from) * i as f64 / SAMPLES as f64;
        if let Some(p) = sample_point(construction, ctx, t)
            && let Some(q) = camera.to_canvas(p, canvas)
        {
            current.push(q);
        } else {
            if current.len() >= 2 {
                pieces.push(current);
            }
            current = Vec::new();
        }
    }

    if current.len() >= 2 {
        pieces.push(current);
    }

    pieces
}

#[cfg(test)]
mod tests;
