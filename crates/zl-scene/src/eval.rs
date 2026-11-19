use crate::{BinaryOp, Callee, Expr, ObjectId, Property, UnaryOp, Value};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct EvalContext {
    pub objects: BTreeMap<ObjectId, Value>,
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
            Expr::Constant(value) => Some(value.clone()),
            Expr::Reference { object, property } => {
                let value = ctx.objects.get(object)?;
                match (property, value) {
                    (Property::Value, Value::Real(_) | Value::Complex(_))
                    | (Property::Position, Value::Vector(_)) => Some(value.clone()),
                    _ => None,
                }
            }
            Expr::Variable(name) => ctx.variable(name).cloned(),
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
                    Callee::User(_) => todo!(),
                }
            }
            Expr::Vector { .. } => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BuiltInFn;
    use glam::DVec3;
    use num_complex::Complex;

    fn ctx_with(vars: &[(&str, f64)]) -> EvalContext {
        EvalContext {
            variables: vars
                .iter()
                .map(|(name, v)| (name.to_string(), Value::Real(*v)))
                .collect(),
            ..EvalContext::default()
        }
    }

    #[test]
    fn empty_context_has_no_variable() {
        assert_eq!(EvalContext::default().variable("x"), None);
    }

    #[test]
    fn finds_a_single_variable() {
        let ctx = ctx_with(&[("x", 0.3)]);
        assert_eq!(ctx.variable("x"), Some(&Value::Real(0.3)));
    }

    #[test]
    fn unknown_name_returns_none() {
        let ctx = ctx_with(&[("x", 0.3), ("n", 2.0)]);
        assert_eq!(ctx.variable("t"), None);
    }

    #[test]
    fn inner_variable_shadows_outer_one() {
        let ctx = ctx_with(&[("n", 1.0), ("n", 2.0)]);
        assert_eq!(ctx.variable("n"), Some(&Value::Real(2.0)));
    }

    #[test]
    fn outer_variable_reappears_after_pop() {
        let mut ctx = ctx_with(&[("n", 1.0), ("n", 2.0)]);
        ctx.variables.pop();
        assert_eq!(ctx.variable("n"), Some(&Value::Real(1.0)));
    }

    #[test]
    fn other_variables_stay_visible_under_shadowing() {
        let ctx = ctx_with(&[("x", 0.5), ("n", 1.0), ("n", 2.0)]);
        assert_eq!(ctx.variable("x"), Some(&Value::Real(0.5)));
    }

    #[test]
    fn names_must_match_exactly() {
        let ctx = ctx_with(&[("n", 1.0)]);
        assert_eq!(ctx.variable("nn"), None);
        assert_eq!(ctx.variable(""), None);
        assert_eq!(ctx.variable("N"), None);
    }

    fn ctx_with_objects(objects: &[(u64, Value)]) -> EvalContext {
        EvalContext {
            objects: objects
                .iter()
                .map(|(id, v)| (ObjectId(*id), v.clone()))
                .collect(),
            ..EvalContext::default()
        }
    }

    fn reference(id: u64, property: Property) -> Expr {
        Expr::Reference {
            object: ObjectId(id),
            property,
        }
    }

    #[test]
    fn constant_evaluates_to_its_value() {
        let ctx = EvalContext::default();
        for v in [
            Value::Real(2.5),
            Value::Complex(Complex::new(3.0, 4.0)),
            Value::Vector(DVec3::new(1.0, 2.0, 0.0)),
        ] {
            assert_eq!(Expr::Constant(v.clone()).evaluate(&ctx), Some(v));
        }
    }

    #[test]
    fn variable_evaluates_to_its_bound_value() {
        let ctx = ctx_with(&[("x", 0.3)]);
        let expr = Expr::Variable("x".to_string());
        assert_eq!(expr.evaluate(&ctx), Some(Value::Real(0.3)));
    }

    #[test]
    fn unbound_variable_is_undefined() {
        let ctx = ctx_with(&[("x", 0.3)]);
        let expr = Expr::Variable("t".to_string());
        assert_eq!(expr.evaluate(&ctx), None);
    }

    #[test]
    fn variable_uses_innermost_binding() {
        let ctx = ctx_with(&[("n", 1.0), ("n", 2.0)]);
        let expr = Expr::Variable("n".to_string());
        assert_eq!(expr.evaluate(&ctx), Some(Value::Real(2.0)));
    }

    #[test]
    fn reference_to_number_value() {
        let ctx = ctx_with_objects(&[
            (1, Value::Real(0.7)),
            (2, Value::Complex(Complex::new(0.5, 14.1))),
        ]);
        assert_eq!(
            reference(1, Property::Value).evaluate(&ctx),
            Some(Value::Real(0.7))
        );
        assert_eq!(
            reference(2, Property::Value).evaluate(&ctx),
            Some(Value::Complex(Complex::new(0.5, 14.1)))
        );
    }

    #[test]
    fn reference_to_point_position() {
        let p = Value::Vector(DVec3::new(1.0, -2.0, 0.0));
        let ctx = ctx_with_objects(&[(5, p.clone())]);
        assert_eq!(reference(5, Property::Position).evaluate(&ctx), Some(p));
    }

    #[test]
    fn reference_with_wrong_property_is_undefined() {
        let ctx = ctx_with_objects(&[
            (1, Value::Real(0.7)),
            (2, Value::Complex(Complex::new(0.5, 14.1))),
            (5, Value::Vector(DVec3::new(1.0, -2.0, 0.0))),
        ]);
        assert_eq!(reference(1, Property::Position).evaluate(&ctx), None);
        assert_eq!(reference(2, Property::Position).evaluate(&ctx), None);
        assert_eq!(reference(5, Property::Value).evaluate(&ctx), None);
    }

    #[test]
    fn reference_to_missing_object_is_undefined() {
        let ctx = ctx_with_objects(&[(1, Value::Real(0.7))]);
        assert_eq!(reference(2, Property::Value).evaluate(&ctx), None);
    }

    #[test]
    fn reference_reads_the_right_object() {
        let ctx = ctx_with_objects(&[(1, Value::Real(1.0)), (2, Value::Real(2.0))]);
        assert_eq!(
            reference(2, Property::Value).evaluate(&ctx),
            Some(Value::Real(2.0))
        );
    }

    fn real(x: f64) -> Expr {
        Expr::Constant(Value::Real(x))
    }

    fn unary(op: UnaryOp, arg: Expr) -> Expr {
        Expr::Unary {
            op,
            arg: Box::new(arg),
        }
    }

    fn binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
        Expr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    #[test]
    fn unary_neg_evaluates() {
        let ctx = EvalContext::default();
        assert_eq!(
            unary(UnaryOp::Neg, real(2.0)).evaluate(&ctx),
            Some(Value::Real(-2.0))
        );
    }

    #[test]
    fn binary_add_and_sub_evaluate_in_order() {
        let ctx = EvalContext::default();
        assert_eq!(
            binary(BinaryOp::Add, real(2.0), real(3.0)).evaluate(&ctx),
            Some(Value::Real(5.0))
        );
        assert_eq!(
            binary(BinaryOp::Sub, real(2.0), real(3.0)).evaluate(&ctx),
            Some(Value::Real(-1.0))
        );
    }

    #[test]
    fn nested_tree_evaluates() {
        let ctx = ctx_with(&[("x", 4.0)]);
        let expr = unary(
            UnaryOp::Neg,
            binary(
                BinaryOp::Sub,
                Expr::Variable("x".to_string()),
                binary(BinaryOp::Add, real(1.0), real(0.5)),
            ),
        );
        assert_eq!(expr.evaluate(&ctx), Some(Value::Real(-2.5)));
    }

    #[test]
    fn undefined_propagates_through_operations() {
        let ctx = EvalContext::default();
        let missing = Expr::Variable("t".to_string());
        assert_eq!(unary(UnaryOp::Neg, missing.clone()).evaluate(&ctx), None);
        assert_eq!(
            binary(BinaryOp::Add, missing.clone(), real(1.0)).evaluate(&ctx),
            None
        );
        assert_eq!(
            binary(BinaryOp::Sub, real(1.0), missing).evaluate(&ctx),
            None
        );
    }

    #[test]
    fn kind_mismatch_in_binary_is_undefined() {
        let ctx = EvalContext::default();
        let point = Expr::Constant(Value::Vector(DVec3::new(1.0, 0.0, 0.0)));
        assert_eq!(binary(BinaryOp::Add, real(1.0), point).evaluate(&ctx), None);
    }

    #[test]
    fn binary_mul_div_pow_evaluate_in_order() {
        let ctx = EvalContext::default();
        assert_eq!(
            binary(BinaryOp::Mul, real(2.0), real(3.0)).evaluate(&ctx),
            Some(Value::Real(6.0))
        );
        assert_eq!(
            binary(BinaryOp::Div, real(3.0), real(6.0)).evaluate(&ctx),
            Some(Value::Real(0.5))
        );
        assert_eq!(
            binary(BinaryOp::Pow, real(2.0), real(3.0)).evaluate(&ctx),
            Some(Value::Real(8.0))
        );
    }

    #[test]
    fn polynomial_in_a_variable() {
        let ctx = ctx_with(&[("x", 3.0)]);
        let x = || Expr::Variable("x".to_string());
        let expr = binary(
            BinaryOp::Add,
            binary(
                BinaryOp::Mul,
                real(2.0),
                binary(BinaryOp::Pow, x(), real(2.0)),
            ),
            binary(BinaryOp::Div, x(), real(4.0)),
        );
        assert_eq!(expr.evaluate(&ctx), Some(Value::Real(18.75)));
    }

    #[test]
    fn division_by_zero_in_tree_is_undefined() {
        let ctx = ctx_with(&[("x", 0.0)]);
        let expr = binary(BinaryOp::Div, real(1.0), Expr::Variable("x".to_string()));
        assert_eq!(expr.evaluate(&ctx), None);
    }

    fn call(f: BuiltInFn, args: Vec<Expr>) -> Expr {
        Expr::Call {
            callee: Callee::BuiltIn(f),
            args,
        }
    }

    #[test]
    fn call_evaluates_its_arguments() {
        let ctx = ctx_with(&[("x", 0.0)]);
        let expr = call(BuiltInFn::Cos, vec![Expr::Variable("x".to_string())]);
        assert_eq!(expr.evaluate(&ctx), Some(Value::Real(1.0)));
    }

    #[test]
    fn call_with_wrong_number_of_arguments_is_undefined() {
        let ctx = EvalContext::default();
        assert_eq!(call(BuiltInFn::Cos, vec![]).evaluate(&ctx), None);
        assert_eq!(
            call(BuiltInFn::Cos, vec![real(1.0), real(2.0)]).evaluate(&ctx),
            None
        );
    }

    #[test]
    fn call_with_undefined_argument_is_undefined() {
        let ctx = EvalContext::default();
        let expr = call(BuiltInFn::Sin, vec![Expr::Variable("t".to_string())]);
        assert_eq!(expr.evaluate(&ctx), None);
    }

    #[test]
    fn call_passes_arguments_in_order() {
        let ctx = EvalContext::default();
        let expr = call(BuiltInFn::Log, vec![real(8.0), real(2.0)]);
        match expr.evaluate(&ctx) {
            Some(Value::Real(x)) => assert!((x - 3.0).abs() < 1e-12),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn unit_circle_point_from_theta() {
        let ctx = ctx_with(&[("theta", std::f64::consts::FRAC_PI_2)]);
        let theta = || Expr::Variable("theta".to_string());
        let expr = Expr::Call {
            callee: Callee::BuiltIn(BuiltInFn::ToPoint),
            args: vec![binary(
                BinaryOp::Add,
                call(BuiltInFn::Cos, vec![theta()]),
                binary(
                    BinaryOp::Mul,
                    Expr::Constant(Value::Complex(Complex::new(0.0, 1.0))),
                    call(BuiltInFn::Sin, vec![theta()]),
                ),
            )],
        };
        match expr.evaluate(&ctx) {
            Some(Value::Vector(p)) => {
                assert!(p.x.abs() < 1e-12 && (p.y - 1.0).abs() < 1e-12 && p.z == 0.0)
            }
            other => panic!("{other:?}"),
        }
    }
}
