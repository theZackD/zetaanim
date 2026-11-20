use crate::Expr;

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
