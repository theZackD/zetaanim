use crate::{EvalContext, Expr, Property, Value};

#[derive(Clone, Debug, PartialEq)]
pub enum Origin {
    Mouse,
    Script,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Construction {
    Point { position: Expr },
    Number { value: Expr },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    pub name: String,
    pub origin: Origin,
    pub construction: Construction,
}

impl Construction {
    pub fn evaluate(&self, ctx: &EvalContext) -> Option<Vec<(Property, Value)>> {
        match self {
            Construction::Point { position } => {
                let val = position.evaluate(ctx)?;
                match val {
                    Value::Vector(_) => Some(vec![(Property::Position, val)]),
                    Value::Complex(_) | Value::Real(_) => None,
                }
            }
            Construction::Number { value } => {
                let val = value.evaluate(ctx)?;
                match val {
                    Value::Complex(_) | Value::Real(_) => Some(vec![(Property::Value, val)]),
                    Value::Vector(_) => None,
                }
            }
        }
    }
}
