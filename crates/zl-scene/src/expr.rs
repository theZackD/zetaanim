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
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Property {
    Value,
    Position,
}
