use std::collections::BTreeSet;

use crate::{ObjectId, Value};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UnaryOp {
    Neg,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BuiltInFn {
    Sin,
    Cos,
    Tan,
    Sqrt,
    Exp,
    Ln,
    Abs,
    Asin,
    Acos,
    Atan,
    Atan2,
    Sinh,
    Cosh,
    Tanh,
    Log10,
    Log,
    Cbrt,
    NthRoot,
    Re,
    Im,
    Arg,
    Conj,
    ToComplex,
    ToPoint,
    Dot,
    Cross,
    Norm,
    Floor,
    Ceil,
    Round,
    Sign,
    Min,
    Max,
    Chop,
    Factorial,
    Mod,
    Binomial,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Property {
    Value,
    Position,
    Start,
    End,
    Center,
    Radius,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Callee {
    BuiltIn(BuiltInFn),
    User(ObjectId),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Constant(Value),
    Reference {
        object: ObjectId,
        property: Property,
    },
    Variable(String),
    Unary {
        op: UnaryOp,
        arg: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        callee: Callee,
        args: Vec<Expr>,
    },
    Vector {
        x: Box<Expr>,
        y: Box<Expr>,
        z: Box<Expr>,
    },
}

impl Expr {
    pub fn dependencies(&self) -> BTreeSet<ObjectId> {
        let mut dependencies: BTreeSet<ObjectId> = BTreeSet::new();
        self.collect_dependencies(&mut dependencies);
        dependencies
    }

    fn collect_dependencies(&self, dependencies: &mut BTreeSet<ObjectId>) {
        match self {
            Expr::Constant(_) | Expr::Variable(_) => {}

            Expr::Reference { object, .. } => {
                dependencies.insert(*object);
            }
            Expr::Unary { arg, .. } => {
                arg.collect_dependencies(dependencies);
            }
            Expr::Binary { left, right, .. } => {
                left.collect_dependencies(dependencies);
                right.collect_dependencies(dependencies);
            }
            Expr::Call { callee, args } => {
                match callee {
                    Callee::User(id) => {
                        dependencies.insert(*id);
                    }
                    Callee::BuiltIn(_) => {}
                }
                for arg in args {
                    arg.collect_dependencies(dependencies);
                }
            }
            Expr::Vector { x, y, z } => {
                x.collect_dependencies(dependencies);
                y.collect_dependencies(dependencies);
                z.collect_dependencies(dependencies);
            }
        }
    }
}

#[cfg(test)]
mod tests;
