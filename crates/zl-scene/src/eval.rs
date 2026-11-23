use glam::DVec3;

use crate::{BinaryOp, Callee, Expr, ObjectId, ObjectValue, UnaryOp, Value};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct EvalContext {
    pub objects: BTreeMap<ObjectId, ObjectValue>,
    pub variables: Vec<(String, Value)>,
    pub time: f64,
}

impl EvalContext {
    pub fn variable(&self, name: &str) -> Option<&Value> {
        for (var_name, value) in self.variables.iter().rev() {
            if var_name == name {
                return Some(value);
            }
        }
        None
    }
}

impl Expr {
    pub fn evaluate(&self, ctx: &EvalContext) -> Option<Value> {
        match self {
            Expr::Constant(value) => value.clone().finite(),
            Expr::Reference { object, property } => {
                ctx.objects.get(object)?.property(*property)?.finite()
            }
            Expr::Variable(name) => ctx.variable(name).cloned()?.finite(),
            Expr::Unary { op, arg } => {
                let value = arg.evaluate(ctx)?;
                match op {
                    UnaryOp::Neg => value.neg(),
                }
            }
            Expr::Binary { op, left, right } => {
                let val_l = left.evaluate(ctx)?;
                let val_r = right.evaluate(ctx)?;
                match op {
                    BinaryOp::Add => val_l.add(&val_r),
                    BinaryOp::Sub => val_l.sub(&val_r),
                    BinaryOp::Mul => val_l.mul(&val_r),
                    BinaryOp::Div => val_l.div(&val_r),
                    BinaryOp::Pow => val_l.pow(&val_r),
                }
            }
            Expr::Call { callee, args } => {
                let mut values: Vec<Value> = Vec::new();
                for arg in args {
                    let value = arg.evaluate(ctx)?;
                    values.push(value);
                }
                match callee {
                    Callee::BuiltIn(f) => f.apply(&values),
                    Callee::User(_) => None,
                }
            }
            Expr::Vector { x, y, z } => {
                let val_x = x.evaluate(ctx)?.as_real()?;
                let val_y = y.evaluate(ctx)?.as_real()?;
                let val_z = z.evaluate(ctx)?.as_real()?;
                Some(Value::Vector(DVec3::new(val_x, val_y, val_z)))
            }
        }
    }
}

#[cfg(test)]
mod tests;
