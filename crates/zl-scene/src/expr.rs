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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Property {
    Value,
    Position,
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
