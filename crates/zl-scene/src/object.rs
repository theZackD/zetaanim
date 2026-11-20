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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BinaryOp, BuiltInFn, Callee, ObjectId, UnaryOp};
    use glam::DVec3;
    use num_complex::Complex;
    use std::f64::consts::{FRAC_PI_2, FRAC_PI_3};

    const EPS: f64 = 1e-12;

    fn real(x: f64) -> Value {
        Value::Real(x)
    }

    fn complex(re: f64, im: f64) -> Value {
        Value::Complex(Complex::new(re, im))
    }

    fn vector(x: f64, y: f64, z: f64) -> Value {
        Value::Vector(DVec3::new(x, y, z))
    }

    fn constant(value: Value) -> Expr {
        Expr::Constant(value)
    }

    fn reference(id: u64, property: Property) -> Expr {
        Expr::Reference {
            object: ObjectId(id),
            property,
        }
    }

    fn call(f: BuiltInFn, args: Vec<Expr>) -> Expr {
        Expr::Call {
            callee: Callee::BuiltIn(f),
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

    fn vector_expr(x: Expr, y: Expr, z: Expr) -> Expr {
        Expr::Vector {
            x: Box::new(x),
            y: Box::new(y),
            z: Box::new(z),
        }
    }

    fn point(position: Expr) -> Construction {
        Construction::Point { position }
    }

    fn number(value: Expr) -> Construction {
        Construction::Number { value }
    }

    fn ctx_with(entries: &[(u64, Property, Value)]) -> EvalContext {
        EvalContext {
            objects: entries
                .iter()
                .map(|(id, property, value)| ((ObjectId(*id), *property), value.clone()))
                .collect(),
            ..EvalContext::default()
        }
    }

    fn store(ctx: &mut EvalContext, id: u64, values: Vec<(Property, Value)>) {
        for (property, value) in values {
            ctx.objects.insert((ObjectId(id), property), value);
        }
    }

    fn single(result: Option<Vec<(Property, Value)>>, expected: Property) -> Value {
        match result {
            Some(values) => {
                assert_eq!(values.len(), 1, "expected one property, got {values:?}");
                let (property, value) = values.into_iter().next().unwrap();
                assert_eq!(property, expected);
                value
            }
            None => panic!("expected a defined object with {expected:?}"),
        }
    }

    fn assert_vector_close(value: Value, x: f64, y: f64, z: f64) {
        match value {
            Value::Vector(p) => assert!(
                (p.x - x).abs() < EPS && (p.y - y).abs() < EPS && (p.z - z).abs() < EPS,
                "{p} != ({x}, {y}, {z})"
            ),
            other => panic!("expected a vector, got {other:?}"),
        }
    }

    fn assert_real_close(value: Value, expected: f64) {
        match value {
            Value::Real(x) => assert!((x - expected).abs() < EPS, "{x} != {expected}"),
            other => panic!("expected a real, got {other:?}"),
        }
    }

    #[test]
    fn point_with_constant_position_exposes_its_position() {
        let ctx = EvalContext::default();
        assert_eq!(
            point(constant(vector(1.0, -2.0, 3.0))).evaluate(&ctx),
            Some(vec![(Property::Position, vector(1.0, -2.0, 3.0))])
        );
    }

    #[test]
    fn point_on_unit_circle_from_a_number() {
        let ctx = ctx_with(&[(0, Property::Value, real(FRAC_PI_2))]);
        let theta = || reference(0, Property::Value);
        let position = vector_expr(
            call(BuiltInFn::Cos, vec![theta()]),
            call(BuiltInFn::Sin, vec![theta()]),
            constant(real(0.0)),
        );
        let value = single(point(position).evaluate(&ctx), Property::Position);
        assert_vector_close(value, 0.0, 1.0, 0.0);
    }

    #[test]
    fn point_copies_another_point_position() {
        let ctx = ctx_with(&[(0, Property::Position, vector(4.0, 5.0, 0.0))]);
        assert_eq!(
            point(reference(0, Property::Position)).evaluate(&ctx),
            Some(vec![(Property::Position, vector(4.0, 5.0, 0.0))])
        );
    }

    #[test]
    fn point_from_complex_through_explicit_conversion() {
        let ctx = ctx_with(&[(0, Property::Value, complex(0.5, 14.13))]);
        let position = call(BuiltInFn::ToPoint, vec![reference(0, Property::Value)]);
        assert_eq!(
            point(position).evaluate(&ctx),
            Some(vec![(Property::Position, vector(0.5, 14.13, 0.0))])
        );
    }

    #[test]
    fn point_with_number_position_is_undefined() {
        let ctx = EvalContext::default();
        assert_eq!(point(constant(real(2.0))).evaluate(&ctx), None);
    }

    #[test]
    fn point_rejects_complex_even_with_zero_imaginary_part() {
        let ctx = EvalContext::default();
        assert_eq!(point(constant(complex(1.0, 2.0))).evaluate(&ctx), None);
        assert_eq!(point(constant(complex(3.0, 0.0))).evaluate(&ctx), None);
    }

    #[test]
    fn point_referencing_a_number_is_undefined() {
        let ctx = ctx_with(&[
            (0, Property::Value, real(3.0)),
            (1, Property::Value, complex(1.0, 1.0)),
        ]);
        assert_eq!(point(reference(0, Property::Value)).evaluate(&ctx), None);
        assert_eq!(point(reference(1, Property::Value)).evaluate(&ctx), None);
    }

    #[test]
    fn point_with_undefined_position_is_undefined() {
        let ctx = EvalContext::default();
        let missing = reference(7, Property::Position);
        let division_by_zero = vector_expr(
            binary(BinaryOp::Div, constant(real(1.0)), constant(real(0.0))),
            constant(real(0.0)),
            constant(real(0.0)),
        );
        assert_eq!(point(missing).evaluate(&ctx), None);
        assert_eq!(point(division_by_zero).evaluate(&ctx), None);
    }

    #[test]
    fn number_with_real_value() {
        let ctx = EvalContext::default();
        assert_eq!(
            number(constant(real(0.7))).evaluate(&ctx),
            Some(vec![(Property::Value, real(0.7))])
        );
    }

    #[test]
    fn number_with_complex_value() {
        let ctx = EvalContext::default();
        assert_eq!(
            number(constant(complex(0.5, 14.13))).evaluate(&ctx),
            Some(vec![(Property::Value, complex(0.5, 14.13))])
        );
    }

    #[test]
    fn number_keeps_complex_with_zero_imaginary_part() {
        let ctx = EvalContext::default();
        assert_eq!(
            number(constant(complex(2.0, 0.0))).evaluate(&ctx),
            Some(vec![(Property::Value, complex(2.0, 0.0))])
        );
    }

    #[test]
    fn number_defined_by_another_number() {
        let ctx = ctx_with(&[(0, Property::Value, real(2.0))]);
        let value = binary(
            BinaryOp::Add,
            binary(
                BinaryOp::Mul,
                constant(real(2.0)),
                reference(0, Property::Value),
            ),
            constant(real(1.0)),
        );
        assert_eq!(
            number(value).evaluate(&ctx),
            Some(vec![(Property::Value, real(5.0))])
        );
    }

    #[test]
    fn number_with_vector_value_is_undefined() {
        let ctx = EvalContext::default();
        assert_eq!(number(constant(vector(1.0, 2.0, 0.0))).evaluate(&ctx), None);
    }

    #[test]
    fn number_referencing_a_point_position_is_undefined() {
        let ctx = ctx_with(&[(0, Property::Position, vector(1.0, 2.0, 0.0))]);
        assert_eq!(
            number(reference(0, Property::Position)).evaluate(&ctx),
            None
        );
    }

    #[test]
    fn reference_to_a_property_the_object_does_not_have_is_undefined() {
        let ctx = ctx_with(&[
            (0, Property::Position, vector(1.0, 2.0, 0.0)),
            (1, Property::Value, real(3.0)),
        ]);
        assert_eq!(number(reference(0, Property::Value)).evaluate(&ctx), None);
        assert_eq!(point(reference(1, Property::Position)).evaluate(&ctx), None);
    }

    #[test]
    fn number_with_undefined_value_is_undefined() {
        let ctx = EvalContext::default();
        let sqrt_of_negative_real = call(BuiltInFn::Sqrt, vec![constant(real(-1.0))]);
        assert_eq!(number(sqrt_of_negative_real).evaluate(&ctx), None);
        assert_eq!(number(reference(3, Property::Value)).evaluate(&ctx), None);
    }

    #[test]
    fn non_finite_constants_make_objects_undefined() {
        let ctx = EvalContext::default();
        assert_eq!(number(constant(real(f64::INFINITY))).evaluate(&ctx), None);
        assert_eq!(number(constant(real(f64::NAN))).evaluate(&ctx), None);
        assert_eq!(
            number(constant(complex(0.0, f64::INFINITY))).evaluate(&ctx),
            None
        );
        assert_eq!(
            point(constant(vector(f64::INFINITY, 0.0, 0.0))).evaluate(&ctx),
            None
        );
    }

    #[test]
    fn non_finite_referenced_values_make_objects_undefined() {
        let ctx = ctx_with(&[
            (0, Property::Value, real(f64::NAN)),
            (1, Property::Position, vector(0.0, f64::INFINITY, 0.0)),
        ]);
        assert_eq!(number(reference(0, Property::Value)).evaluate(&ctx), None);
        assert_eq!(point(reference(1, Property::Position)).evaluate(&ctx), None);
    }

    #[test]
    fn evaluated_values_never_carry_negative_zero() {
        let ctx = EvalContext::default();
        match single(
            number(neg(constant(real(0.0)))).evaluate(&ctx),
            Property::Value,
        ) {
            Value::Real(x) => assert!(!x.is_sign_negative()),
            other => panic!("expected a real, got {other:?}"),
        }
        match single(
            number(neg(constant(complex(4.0, 0.0)))).evaluate(&ctx),
            Property::Value,
        ) {
            Value::Complex(z) => assert!(!z.im.is_sign_negative()),
            other => panic!("expected a complex, got {other:?}"),
        }
        match single(
            point(neg(constant(vector(0.0, 0.0, 0.0)))).evaluate(&ctx),
            Property::Position,
        ) {
            Value::Vector(p) => assert!(
                !p.x.is_sign_negative() && !p.y.is_sign_negative() && !p.z.is_sign_negative()
            ),
            other => panic!("expected a vector, got {other:?}"),
        }
    }

    #[test]
    fn results_can_be_stored_and_referenced_in_order() {
        let mut ctx = EvalContext::default();
        let theta_values = number(constant(real(FRAC_PI_3))).evaluate(&ctx).unwrap();
        store(&mut ctx, 0, theta_values);
        let theta = || reference(0, Property::Value);
        let on_circle = point(vector_expr(
            call(BuiltInFn::Cos, vec![theta()]),
            call(BuiltInFn::Sin, vec![theta()]),
            constant(real(0.0)),
        ));
        let point_values = on_circle.evaluate(&ctx).unwrap();
        store(&mut ctx, 1, point_values);
        let abscissa = number(call(
            BuiltInFn::Dot,
            vec![
                reference(1, Property::Position),
                constant(vector(1.0, 0.0, 0.0)),
            ],
        ));
        let value = single(abscissa.evaluate(&ctx), Property::Value);
        assert_real_close(value, 0.5);
    }

    #[test]
    fn undefined_object_propagates_to_its_dependents() {
        let mut ctx = EvalContext::default();
        let theta = number(constant(vector(1.0, 0.0, 0.0)));
        if let Some(values) = theta.evaluate(&ctx) {
            store(&mut ctx, 0, values);
        }
        let on_circle = point(vector_expr(
            call(BuiltInFn::Cos, vec![reference(0, Property::Value)]),
            constant(real(0.0)),
            constant(real(0.0)),
        ));
        assert_eq!(on_circle.evaluate(&ctx), None);
        if let Some(values) = on_circle.evaluate(&ctx) {
            store(&mut ctx, 1, values);
        }
        let abscissa = number(call(
            BuiltInFn::Dot,
            vec![
                reference(1, Property::Position),
                constant(vector(1.0, 0.0, 0.0)),
            ],
        ));
        assert_eq!(abscissa.evaluate(&ctx), None);
    }
}
