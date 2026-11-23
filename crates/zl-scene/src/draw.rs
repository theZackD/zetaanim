use glam::{DVec2, DVec3};
use kurbo::{BezPath, Circle, Line, Rect, Shape};
use zl_primitives::{COORDINATE_LIMIT, Color, Dash, Primitive, Stroke, Style};

use crate::{
    Camera, Canvas, Construction, EvalContext, LineStyle, Object, ObjectStyle, ObjectValue, Value,
};

const TOLERANCE: f64 = 0.1;
pub(crate) const LAYER_COUNT: u8 = 3;
const GRID: u32 = 64;
const OFFSET: f64 = 0.381966;
const MAX_DEPTH: u32 = 10;
const EDGE_DEPTH: u32 = 50;
const JUMP_RATIO: f64 = 0.99;

#[derive(Clone, Copy)]
struct Sample {
    t: f64,
    point: Option<DVec2>,
}

struct Sampler<'a> {
    construction: &'a Construction,
    ctx: &'a mut EvalContext,
    camera: &'a Camera,
    canvas: &'a Canvas,
    margin: f64,
    pieces: Vec<Vec<DVec2>>,
    current: Vec<DVec2>,
}

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
            let pieces = sample_pieces(
                &object.construction,
                *from,
                *to,
                camera,
                canvas,
                ctx,
                stroke.width / 2.0,
            );
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

impl Sampler<'_> {
    fn sample(&mut self, t: f64) -> Sample {
        let point = sample_point(self.construction, self.ctx, t)
            .and_then(|p| self.camera.to_canvas(p, self.canvas));
        Sample { t, point }
    }

    fn emit(&mut self, point: Option<DVec2>) {
        match point {
            Some(p) => self.current.push(p),
            None => {
                let piece = std::mem::take(&mut self.current);
                if piece.len() >= 2 {
                    self.pieces.push(piece);
                }
            }
        }
    }

    fn off_screen(&self, a: DVec2, m: DVec2, b: DVec2) -> bool {
        let low = a.min(m).min(b);
        let high = a.max(m).max(b);
        let area = visible_area(self.canvas, self.margin);

        high.x < area.x0 || high.y < area.y0 || low.x > area.x1 || low.y > area.y1
    }

    fn refine(&mut self, a: Sample, m: Sample, b: Sample, depth: u32) {
        if depth >= EDGE_DEPTH || m.t == a.t || m.t == b.t {
            self.emit(m.point);
            self.emit(b.point);
            return;
        }
        if let (Some(pa), Some(pm), Some(pb)) = (a.point, m.point, b.point)
            && self.off_screen(pa, pm, pb)
        {
            self.emit(b.point);
            return;
        }
        let q1 = self.sample((a.t + m.t) / 2.0);
        let q3 = self.sample((m.t + b.t) / 2.0);

        if a.point.is_none()
            && b.point.is_none()
            && m.point.is_none()
            && q1.point.is_none()
            && q3.point.is_none()
        {
            self.emit(None);
            return;
        }

        if let (Some(pa), Some(pm), Some(pb), Some(pq1), Some(pq3)) =
            (a.point, m.point, b.point, q1.point, q3.point)
        {
            if distance_to_segment(pq1, pa, pb) <= TOLERANCE
                && distance_to_segment(pm, pa, pb) <= TOLERANCE
                && distance_to_segment(pq3, pa, pb) <= TOLERANCE
                && !is_jump(pa, pq1, pm, pq3, pb)
            {
                self.emit(b.point);
                return;
            } else if depth >= MAX_DEPTH {
                if is_jump(pa, pq1, pm, pq3, pb) {
                    self.emit(None);
                } else {
                    self.emit(m.point);
                }
                self.emit(b.point);
                return;
            }
        }

        self.refine(a, q1, m, depth + 1);
        self.refine(m, q3, b, depth + 1);
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
    margin: f64,
) -> Vec<Vec<DVec2>> {
    let mut sampler = Sampler {
        construction,
        ctx,
        camera,
        canvas,
        margin,
        pieces: Vec::new(),
        current: Vec::new(),
    };
    let mut a = sampler.sample(from);
    sampler.emit(a.point);

    for i in 1..=GRID {
        let b = sampler.sample(grid_parameter(from, to, i));
        let m = sampler.sample((a.t + b.t) / 2.0);

        sampler.refine(a, m, b, 0);
        a = b;
    }
    sampler.emit(None);
    sampler.pieces
}

fn grid_parameter(from: f64, to: f64, i: u32) -> f64 {
    if i == 0 {
        from
    } else if i == GRID {
        to
    } else {
        let shift = (f64::from(i) * OFFSET).fract() / 2.0;
        from + (to - from) * (f64::from(i) - shift) / f64::from(GRID)
    }
}

fn distance_to_segment(p: DVec2, a: DVec2, b: DVec2) -> f64 {
    let ab = b - a;
    if ab.length_squared() == 0.0 {
        return p.distance(a);
    }
    let k = ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
    p.distance(a + ab * k)
}

fn is_jump(a: DVec2, q1: DVec2, m: DVec2, q3: DVec2, b: DVec2) -> bool {
    let half = a.distance(m).max(m.distance(b));
    let quarter = a
        .distance(q1)
        .max(q1.distance(m))
        .max(m.distance(q3))
        .max(q3.distance(b));
    half > 2.0 * TOLERANCE && quarter >= JUMP_RATIO * half
}

fn visible_area(canvas: &Canvas, margin: f64) -> Rect {
    Rect::new(
        -margin,
        -margin,
        f64::from(canvas.format.width()) + margin,
        f64::from(canvas.format.height()) + margin,
    )
}

#[cfg(test)]
mod tests;
