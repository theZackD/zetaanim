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
mod tests {
    use super::*;

    fn constant(x: f64) -> Expr {
        Expr::Constant(Value::Real(x))
    }

    fn reference(id: u64, property: Property) -> Expr {
        Expr::Reference {
            object: ObjectId(id),
            property,
        }
    }

    fn later_object() -> Expr {
        reference(9, Property::Position)
    }

    fn builtin(f: BuiltInFn, args: Vec<Expr>) -> Expr {
        Expr::Call {
            callee: Callee::BuiltIn(f),
            args,
        }
    }

    fn user(id: u64, args: Vec<Expr>) -> Expr {
        Expr::Call {
            callee: Callee::User(ObjectId(id)),
            args,
        }
    }

    fn binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
        Expr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn neg(arg: Expr) -> Expr {
        Expr::Unary {
            op: UnaryOp::Neg,
            arg: Box::new(arg),
        }
    }

    fn vector(x: Expr, y: Expr, z: Expr) -> Expr {
        Expr::Vector {
            x: Box::new(x),
            y: Box::new(y),
            z: Box::new(z),
        }
    }

    fn ids(ids: &[u64]) -> BTreeSet<ObjectId> {
        ids.iter().map(|id| ObjectId(*id)).collect()
    }

    #[test]
    fn constants_and_variables_have_no_dependencies() {
        let x = || Expr::Variable("x".to_string());
        assert_eq!(constant(2.0).dependencies(), ids(&[]));
        assert_eq!(x().dependencies(), ids(&[]));
        let graph = binary(
            BinaryOp::Mul,
            constant(3.0),
            builtin(BuiltInFn::Sin, vec![neg(x())]),
        );
        assert_eq!(graph.dependencies(), ids(&[]));
    }

    #[test]
    fn reference_is_a_dependency_whatever_the_property() {
        assert_eq!(reference(9, Property::Position).dependencies(), ids(&[9]));
        assert_eq!(reference(9, Property::Value).dependencies(), ids(&[9]));
        assert_eq!(reference(9, Property::Radius).dependencies(), ids(&[9]));
    }

    #[test]
    fn finds_a_reference_to_a_later_object_in_every_position() {
        let cases = [
            ("unary argument", neg(later_object())),
            (
                "binary left",
                binary(BinaryOp::Sub, later_object(), constant(1.0)),
            ),
            (
                "binary right",
                binary(BinaryOp::Sub, constant(1.0), later_object()),
            ),
            (
                "first builtin argument",
                builtin(BuiltInFn::Atan2, vec![later_object(), constant(1.0)]),
            ),
            (
                "last builtin argument",
                builtin(BuiltInFn::Atan2, vec![constant(1.0), later_object()]),
            ),
            (
                "user function argument",
                user(4, vec![constant(1.0), later_object()]),
            ),
            (
                "vector x",
                vector(later_object(), constant(0.0), constant(0.0)),
            ),
            (
                "vector y",
                vector(constant(0.0), later_object(), constant(0.0)),
            ),
            (
                "vector z",
                vector(constant(0.0), constant(0.0), later_object()),
            ),
        ];
        for (position, expr) in cases {
            match &expr {
                Expr::Unary { .. }
                | Expr::Binary { .. }
                | Expr::Call { .. }
                | Expr::Vector { .. } => {}
                Expr::Constant(_) | Expr::Variable(_) | Expr::Reference { .. } => {
                    panic!("{position} has no sub-expression to search")
                }
            }
            assert!(
                expr.dependencies().contains(&ObjectId(9)),
                "reference missed in {position}"
            );
        }
    }

    #[test]
    fn user_function_is_a_dependency_with_or_without_arguments() {
        assert_eq!(user(4, vec![]).dependencies(), ids(&[4]));
        assert_eq!(
            user(4, vec![reference(9, Property::Value), constant(1.0)]).dependencies(),
            ids(&[4, 9])
        );
        let nested = binary(BinaryOp::Add, constant(1.0), neg(user(6, vec![])));
        assert_eq!(nested.dependencies(), ids(&[6]));
    }

    #[test]
    fn builtin_function_is_not_a_dependency() {
        assert_eq!(
            builtin(BuiltInFn::Cos, vec![constant(0.0)]).dependencies(),
            ids(&[])
        );
    }

    #[test]
    fn repeated_reference_is_counted_once() {
        let theta = || reference(0, Property::Value);
        let on_unit_circle = vector(
            builtin(BuiltInFn::Cos, vec![theta()]),
            builtin(BuiltInFn::Sin, vec![theta()]),
            constant(0.0),
        );
        assert_eq!(on_unit_circle.dependencies(), ids(&[0]));
    }

    #[test]
    fn collects_every_reference_of_a_nested_expression_in_increasing_order() {
        let expr = vector(
            builtin(BuiltInFn::Cos, vec![reference(11, Property::Value)]),
            neg(binary(
                BinaryOp::Add,
                reference(7, Property::Value),
                user(5, vec![reference(0, Property::Value)]),
            )),
            binary(
                BinaryOp::Pow,
                reference(3, Property::Radius),
                reference(7, Property::Value),
            ),
        );
        let found: Vec<ObjectId> = expr.dependencies().into_iter().collect();
        assert_eq!(
            found,
            vec![
                ObjectId(0),
                ObjectId(3),
                ObjectId(5),
                ObjectId(7),
                ObjectId(11)
            ]
        );
    }
}
