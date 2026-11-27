use std::collections::BTreeSet;

use glam::DVec3;

use crate::{EvalContext, Expr, ObjectId, ObjectStyle, Property, Value};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Origin {
    Mouse,
    Script,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Construction {
    Point {
        position: Expr,
    },
    Number {
        value: Expr,
    },
    Segment {
        start: Expr,
        end: Expr,
    },
    Circle {
        center: Expr,
        radius: Expr,
    },
    Graph {
        variable: String,
        expression: Expr,
        from: Expr,
        to: Expr,
    },
    Curve {
        variable: String,
        position: Expr,
        from: Expr,
        to: Expr,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ObjectValue {
    Number(Value),
    Point(DVec3),
    Segment { start: DVec3, end: DVec3 },
    Circle { center: DVec3, radius: f64 },
    Curve { from: f64, to: f64 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Object {
    pub name: String,
    pub origin: Origin,
    pub construction: Construction,
    pub style: ObjectStyle,
}

impl Construction {
    pub fn evaluate(&self, ctx: &EvalContext) -> Option<ObjectValue> {
        match self {
            Construction::Point { position } => {
                Some(ObjectValue::Point(position.evaluate(ctx)?.as_vector()?))
            }

            Construction::Number { value } => {
                let val = value.evaluate(ctx)?;
                match val {
                    Value::Complex(_) | Value::Real(_) => Some(ObjectValue::Number(val)),
                    Value::Vector(_) => None,
                }
            }

            Construction::Segment { start, end } => {
                let s = start.evaluate(ctx)?.as_vector()?;
                let e = end.evaluate(ctx)?.as_vector()?;
                Some(ObjectValue::Segment { start: s, end: e })
            }

            Construction::Circle { center, radius } => {
                let c = center.evaluate(ctx)?.as_vector()?;
                let r = radius.evaluate(ctx)?.as_real()?;

                if r >= 0.0 {
                    Some(ObjectValue::Circle {
                        center: c,
                        radius: r,
                    })
                } else {
                    None
                }
            }

            Construction::Graph {
                expression,
                from,
                to,
                ..
            } => {
                for dep in expression.dependencies() {
                    if !(ctx.objects.contains_key(&dep)) {
                        return None;
                    }
                }
                let from = from.evaluate(ctx)?.as_real()?;
                let to = to.evaluate(ctx)?.as_real()?;
                if from < to {
                    Some(ObjectValue::Curve { from, to })
                } else {
                    None
                }
            }
            Construction::Curve {
                position, from, to, ..
            } => {
                for dep in position.dependencies() {
                    if !(ctx.objects.contains_key(&dep)) {
                        return None;
                    }
                }
                let from = from.evaluate(ctx)?.as_real()?;
                let to = to.evaluate(ctx)?.as_real()?;
                if from < to {
                    Some(ObjectValue::Curve { from, to })
                } else {
                    None
                }
            }
        }
    }

    pub fn dependencies(&self) -> BTreeSet<ObjectId> {
        match self {
            Construction::Point { position } => position.dependencies(),
            Construction::Number { value } => value.dependencies(),
            Construction::Circle { center, radius } => {
                let mut c = center.dependencies();
                c.extend(radius.dependencies());
                c
            }
            Construction::Segment { start, end } => {
                let mut s = start.dependencies();
                s.extend(end.dependencies());
                s
            }
            Construction::Graph {
                expression,
                from,
                to,
                ..
            } => {
                let mut g = expression.dependencies();
                g.extend(from.dependencies());
                g.extend(to.dependencies());
                g
            }
            Construction::Curve {
                position, from, to, ..
            } => {
                let mut c = position.dependencies();
                c.extend(from.dependencies());
                c.extend(to.dependencies());
                c
            }
        }
    }

    pub(crate) fn property_mut(&mut self, property: Property) -> Option<&mut Expr> {
        match self {
            Construction::Number { value } => match property {
                Property::Value => Some(value),
                _ => None,
            },
            Construction::Point { position } => match property {
                Property::Position => Some(position),
                _ => None,
            },
            Construction::Segment { start, end } => match property {
                Property::Start => Some(start),
                Property::End => Some(end),
                _ => None,
            },
            Construction::Circle { center, radius } => match property {
                Property::Center => Some(center),
                Property::Radius => Some(radius),
                _ => None,
            },
            Construction::Graph { .. } | Construction::Curve { .. } => None,
        }
    }
}

impl ObjectValue {
    pub fn property(&self, property: Property) -> Option<Value> {
        match self {
            ObjectValue::Number(n) => match property {
                Property::Value => Some(n.clone()),
                _ => None,
            },
            ObjectValue::Point(v) => match property {
                Property::Position => Some(Value::Vector(*v)),
                _ => None,
            },
            ObjectValue::Circle { center, radius } => match property {
                Property::Center => Some(Value::Vector(*center)),
                Property::Radius => Some(Value::Real(*radius)),
                _ => None,
            },
            ObjectValue::Segment { start, end } => match property {
                Property::Start => Some(Value::Vector(*start)),
                Property::End => Some(Value::Vector(*end)),
                _ => None,
            },
            ObjectValue::Curve { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests;
