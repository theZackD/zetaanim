use std::collections::BTreeSet;

use glam::DVec3;

use crate::{EvalContext, Expr, ObjectId, ObjectStyle, Property, Value};

#[derive(Clone, Debug, PartialEq)]
pub enum Origin {
    Mouse,
    Script,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Construction {
    Point { position: Expr },
    Number { value: Expr },
    Segment { start: Expr, end: Expr },
    Circle { center: Expr, radius: Expr },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ObjectValue {
    Number(Value),
    Point(DVec3),
    Segment { start: DVec3, end: DVec3 },
    Circle { center: DVec3, radius: f64 },
}

#[derive(Clone, Debug, PartialEq)]
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BinaryOp, BuiltInFn, Callee, ObjectId, UnaryOp};
    use glam::DVec3;
    use num_complex::Complex;
    use std::f64::consts::FRAC_PI_2;

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

    fn segment(start: Expr, end: Expr) -> Construction {
        Construction::Segment { start, end }
    }

    fn circle(center: Expr, radius: Expr) -> Construction {
        Construction::Circle { center, radius }
    }

    fn all_properties() -> [Property; 6] {
        let all = [
            Property::Value,
            Property::Position,
            Property::Start,
            Property::End,
            Property::Center,
            Property::Radius,
        ];
        for property in all {
            match property {
                Property::Value
                | Property::Position
                | Property::Start
                | Property::End
                | Property::Center
                | Property::Radius => {}
            }
        }
        all
    }

    fn ctx_with(entries: &[(u64, ObjectValue)]) -> EvalContext {
        EvalContext {
            objects: entries
                .iter()
                .map(|(id, object)| (ObjectId(*id), object.clone()))
                .collect(),
            ..EvalContext::default()
        }
    }

    fn property_of(result: Option<ObjectValue>, property: Property) -> Value {
        match result {
            Some(object) => match object.property(property) {
                Some(value) => value,
                None => panic!("{object:?} has no {property:?}"),
            },
            None => panic!("expected a defined object with {property:?}"),
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
    fn object_value_answers_only_its_own_properties() {
        let cases = [
            (
                ObjectValue::Number(complex(0.5, 14.13)),
                vec![(Property::Value, complex(0.5, 14.13))],
            ),
            (
                ObjectValue::Point(DVec3::new(1.0, -2.0, 3.0)),
                vec![(Property::Position, vector(1.0, -2.0, 3.0))],
            ),
            (
                ObjectValue::Segment {
                    start: DVec3::new(1.0, 2.0, 0.0),
                    end: DVec3::new(-3.0, 4.0, 5.0),
                },
                vec![
                    (Property::Start, vector(1.0, 2.0, 0.0)),
                    (Property::End, vector(-3.0, 4.0, 5.0)),
                ],
            ),
            (
                ObjectValue::Circle {
                    center: DVec3::new(1.0, 1.0, 0.0),
                    radius: 2.5,
                },
                vec![
                    (Property::Center, vector(1.0, 1.0, 0.0)),
                    (Property::Radius, real(2.5)),
                ],
            ),
        ];
        for (object, answers) in &cases {
            for property in all_properties() {
                let expected = answers
                    .iter()
                    .find(|(answered, _)| *answered == property)
                    .map(|(_, value)| value.clone());
                assert_eq!(
                    object.property(property),
                    expected,
                    "{object:?} asked for {property:?}"
                );
            }
        }
    }

    #[test]
    fn point_with_constant_position_exposes_its_position() {
        let ctx = EvalContext::default();
        assert_eq!(
            point(constant(vector(1.0, -2.0, 3.0))).evaluate(&ctx),
            Some(ObjectValue::Point(DVec3::new(1.0, -2.0, 3.0)))
        );
    }

    #[test]
    fn point_on_unit_circle_from_a_number() {
        let ctx = ctx_with(&[(0, ObjectValue::Number(real(FRAC_PI_2)))]);
        let theta = || reference(0, Property::Value);
        let position = vector_expr(
            call(BuiltInFn::Cos, vec![theta()]),
            call(BuiltInFn::Sin, vec![theta()]),
            constant(real(0.0)),
        );
        let value = property_of(point(position).evaluate(&ctx), Property::Position);
        assert_vector_close(value, 0.0, 1.0, 0.0);
    }

    #[test]
    fn point_copies_another_point_position() {
        let ctx = ctx_with(&[(0, ObjectValue::Point(DVec3::new(4.0, 5.0, 0.0)))]);
        assert_eq!(
            point(reference(0, Property::Position)).evaluate(&ctx),
            Some(ObjectValue::Point(DVec3::new(4.0, 5.0, 0.0)))
        );
    }

    #[test]
    fn point_from_complex_through_explicit_conversion() {
        let ctx = ctx_with(&[(0, ObjectValue::Number(complex(0.5, 14.13)))]);
        let position = call(BuiltInFn::ToPoint, vec![reference(0, Property::Value)]);
        assert_eq!(
            point(position).evaluate(&ctx),
            Some(ObjectValue::Point(DVec3::new(0.5, 14.13, 0.0)))
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
            (0, ObjectValue::Number(real(3.0))),
            (1, ObjectValue::Number(complex(1.0, 1.0))),
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
            Some(ObjectValue::Number(real(0.7)))
        );
    }

    #[test]
    fn number_with_complex_value() {
        let ctx = EvalContext::default();
        assert_eq!(
            number(constant(complex(0.5, 14.13))).evaluate(&ctx),
            Some(ObjectValue::Number(complex(0.5, 14.13)))
        );
    }

    #[test]
    fn number_keeps_complex_with_zero_imaginary_part() {
        let ctx = EvalContext::default();
        assert_eq!(
            number(constant(complex(2.0, 0.0))).evaluate(&ctx),
            Some(ObjectValue::Number(complex(2.0, 0.0)))
        );
    }

    #[test]
    fn number_defined_by_another_number() {
        let ctx = ctx_with(&[(0, ObjectValue::Number(real(2.0)))]);
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
            Some(ObjectValue::Number(real(5.0)))
        );
    }

    #[test]
    fn number_with_vector_value_is_undefined() {
        let ctx = EvalContext::default();
        assert_eq!(number(constant(vector(1.0, 2.0, 0.0))).evaluate(&ctx), None);
    }

    #[test]
    fn number_referencing_a_point_position_is_undefined() {
        let ctx = ctx_with(&[(0, ObjectValue::Point(DVec3::new(1.0, 2.0, 0.0)))]);
        assert_eq!(
            number(reference(0, Property::Position)).evaluate(&ctx),
            None
        );
    }

    #[test]
    fn reference_to_a_property_the_object_does_not_have_is_undefined() {
        let ctx = ctx_with(&[
            (0, ObjectValue::Point(DVec3::new(1.0, 2.0, 0.0))),
            (1, ObjectValue::Number(real(3.0))),
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
            (0, ObjectValue::Number(real(f64::NAN))),
            (1, ObjectValue::Point(DVec3::new(0.0, f64::INFINITY, 0.0))),
        ]);
        assert_eq!(number(reference(0, Property::Value)).evaluate(&ctx), None);
        assert_eq!(point(reference(1, Property::Position)).evaluate(&ctx), None);
    }

    #[test]
    fn evaluated_values_never_carry_negative_zero() {
        let ctx = EvalContext::default();
        match property_of(
            number(neg(constant(real(0.0)))).evaluate(&ctx),
            Property::Value,
        ) {
            Value::Real(x) => assert!(!x.is_sign_negative()),
            other => panic!("expected a real, got {other:?}"),
        }
        match property_of(
            number(neg(constant(complex(4.0, 0.0)))).evaluate(&ctx),
            Property::Value,
        ) {
            Value::Complex(z) => assert!(!z.im.is_sign_negative()),
            other => panic!("expected a complex, got {other:?}"),
        }
        match property_of(
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
    fn segment_exposes_each_endpoint_under_its_own_property() {
        let ctx = EvalContext::default();
        let result = segment(
            constant(vector(1.0, 2.0, 0.0)),
            constant(vector(-3.0, 4.0, 5.0)),
        )
        .evaluate(&ctx);
        assert_eq!(
            result,
            Some(ObjectValue::Segment {
                start: DVec3::new(1.0, 2.0, 0.0),
                end: DVec3::new(-3.0, 4.0, 5.0),
            })
        );
        assert_eq!(
            property_of(result.clone(), Property::Start),
            vector(1.0, 2.0, 0.0)
        );
        assert_eq!(property_of(result, Property::End), vector(-3.0, 4.0, 5.0));
    }

    #[test]
    fn segment_with_coincident_endpoints_is_defined() {
        let ctx = EvalContext::default();
        assert_eq!(
            segment(
                constant(vector(2.0, 2.0, 0.0)),
                constant(vector(2.0, 2.0, 0.0))
            )
            .evaluate(&ctx),
            Some(ObjectValue::Segment {
                start: DVec3::new(2.0, 2.0, 0.0),
                end: DVec3::new(2.0, 2.0, 0.0),
            })
        );
    }

    #[test]
    fn segment_endpoints_must_be_vectors() {
        let ctx = EvalContext::default();
        let good = || constant(vector(1.0, 0.0, 0.0));
        for bad in [real(2.0), complex(1.0, 0.0), complex(1.0, 2.0)] {
            assert_eq!(
                segment(constant(bad.clone()), good()).evaluate(&ctx),
                None,
                "start = {bad:?}"
            );
            assert_eq!(
                segment(good(), constant(bad.clone())).evaluate(&ctx),
                None,
                "end = {bad:?}"
            );
        }
    }

    #[test]
    fn segment_with_an_undefined_endpoint_is_undefined() {
        let ctx = EvalContext::default();
        let good = || constant(vector(1.0, 0.0, 0.0));
        let missing = || reference(7, Property::Position);
        let division_by_zero = || {
            vector_expr(
                binary(BinaryOp::Div, constant(real(1.0)), constant(real(0.0))),
                constant(real(0.0)),
                constant(real(0.0)),
            )
        };
        assert_eq!(segment(missing(), good()).evaluate(&ctx), None);
        assert_eq!(segment(good(), missing()).evaluate(&ctx), None);
        assert_eq!(segment(division_by_zero(), good()).evaluate(&ctx), None);
        assert_eq!(segment(good(), division_by_zero()).evaluate(&ctx), None);
    }

    #[test]
    fn segment_between_two_referenced_points() {
        let ctx = ctx_with(&[
            (0, ObjectValue::Point(DVec3::new(0.0, 0.0, 0.0))),
            (1, ObjectValue::Point(DVec3::new(3.0, 4.0, 0.0))),
        ]);
        assert_eq!(
            segment(
                reference(1, Property::Position),
                reference(0, Property::Position)
            )
            .evaluate(&ctx),
            Some(ObjectValue::Segment {
                start: DVec3::new(3.0, 4.0, 0.0),
                end: DVec3::new(0.0, 0.0, 0.0),
            })
        );
    }

    #[test]
    fn circle_exposes_its_center_and_radius() {
        let ctx = EvalContext::default();
        let result = circle(constant(vector(1.0, -1.0, 2.0)), constant(real(2.5))).evaluate(&ctx);
        assert_eq!(
            result,
            Some(ObjectValue::Circle {
                center: DVec3::new(1.0, -1.0, 2.0),
                radius: 2.5,
            })
        );
        assert_eq!(
            property_of(result.clone(), Property::Center),
            vector(1.0, -1.0, 2.0)
        );
        assert_eq!(property_of(result, Property::Radius), real(2.5));
    }

    #[test]
    fn circle_with_zero_radius_is_defined() {
        let ctx = EvalContext::default();
        let center = || constant(vector(1.0, 1.0, 0.0));
        let expected = Some(ObjectValue::Circle {
            center: DVec3::new(1.0, 1.0, 0.0),
            radius: 0.0,
        });
        assert_eq!(
            circle(center(), constant(real(0.0))).evaluate(&ctx),
            expected
        );
        let from_negative_zero = circle(center(), neg(constant(real(0.0)))).evaluate(&ctx);
        assert_eq!(from_negative_zero, expected);
        match property_of(from_negative_zero, Property::Radius) {
            Value::Real(r) => assert!(!r.is_sign_negative()),
            other => panic!("expected a real, got {other:?}"),
        }
    }

    #[test]
    fn circle_with_negative_radius_is_undefined_without_tolerance() {
        let ctx = EvalContext::default();
        let center = || constant(vector(0.0, 0.0, 0.0));
        for radius in [-1.0, -1e-15, -f64::MIN_POSITIVE, -5e-324] {
            assert_eq!(
                circle(center(), constant(real(radius))).evaluate(&ctx),
                None,
                "radius = {radius:e}"
            );
        }
    }

    #[test]
    fn circle_accepts_a_nearly_real_complex_radius_as_a_real() {
        let ctx = EvalContext::default();
        let center = || constant(vector(0.0, 0.0, 0.0));
        for radius in [complex(2.0, 0.0), complex(2.0, 1e-12)] {
            let result = circle(center(), constant(radius.clone())).evaluate(&ctx);
            assert_eq!(
                result,
                Some(ObjectValue::Circle {
                    center: DVec3::ZERO,
                    radius: 2.0,
                }),
                "radius = {radius:?}"
            );
            assert_eq!(property_of(result, Property::Radius), real(2.0));
        }
    }

    #[test]
    fn circle_rejects_a_truly_complex_or_nearly_real_negative_radius() {
        let ctx = EvalContext::default();
        let center = || constant(vector(0.0, 0.0, 0.0));
        assert_eq!(
            circle(center(), constant(complex(2.0, 0.5))).evaluate(&ctx),
            None
        );
        assert_eq!(
            circle(center(), constant(complex(-1.0, 1e-12))).evaluate(&ctx),
            None
        );
    }

    #[test]
    fn circle_needs_a_vector_center_and_a_number_radius() {
        let ctx = EvalContext::default();
        let center = || constant(vector(0.0, 0.0, 0.0));
        let radius = || constant(real(1.0));
        assert_eq!(
            circle(center(), constant(vector(1.0, 0.0, 0.0))).evaluate(&ctx),
            None
        );
        assert_eq!(circle(constant(real(1.0)), radius()).evaluate(&ctx), None);
        assert_eq!(
            circle(constant(complex(1.0, 1.0)), radius()).evaluate(&ctx),
            None
        );
    }

    #[test]
    fn circle_with_an_undefined_center_or_radius_is_undefined() {
        let ctx = EvalContext::default();
        let center = || constant(vector(0.0, 0.0, 0.0));
        let radius = || constant(real(1.0));
        let sqrt_of_negative_real = call(BuiltInFn::Sqrt, vec![constant(real(-1.0))]);
        assert_eq!(
            circle(reference(7, Property::Position), radius()).evaluate(&ctx),
            None
        );
        assert_eq!(
            circle(center(), reference(7, Property::Value)).evaluate(&ctx),
            None
        );
        assert_eq!(circle(center(), sqrt_of_negative_real).evaluate(&ctx), None);
    }

    #[test]
    fn non_finite_endpoints_or_radius_are_undefined() {
        let ctx = EvalContext::default();
        let origin = || constant(vector(0.0, 0.0, 0.0));
        assert_eq!(
            segment(origin(), constant(vector(f64::NAN, 0.0, 0.0))).evaluate(&ctx),
            None
        );
        assert_eq!(
            circle(
                constant(vector(0.0, f64::INFINITY, 0.0)),
                constant(real(1.0))
            )
            .evaluate(&ctx),
            None
        );
        assert_eq!(
            circle(origin(), constant(real(f64::INFINITY))).evaluate(&ctx),
            None
        );
        assert_eq!(
            circle(origin(), constant(real(f64::NAN))).evaluate(&ctx),
            None
        );
    }

    #[test]
    fn circle_centered_at_a_point_through_another() {
        let ctx = ctx_with(&[
            (0, ObjectValue::Point(DVec3::new(1.0, 1.0, 0.0))),
            (1, ObjectValue::Point(DVec3::new(4.0, 5.0, 0.0))),
        ]);
        let radius = call(
            BuiltInFn::Norm,
            vec![binary(
                BinaryOp::Sub,
                reference(1, Property::Position),
                reference(0, Property::Position),
            )],
        );
        let result = circle(reference(0, Property::Position), radius).evaluate(&ctx);
        assert_vector_close(property_of(result.clone(), Property::Center), 1.0, 1.0, 0.0);
        assert_real_close(property_of(result, Property::Radius), 5.0);
    }

    #[test]
    fn circle_with_radius_a_minus_one_vanishes_below_one_instead_of_bouncing() {
        for (a, expected_radius) in [(2.0, Some(1.0)), (1.0, Some(0.0)), (0.5, None)] {
            let ctx = ctx_with(&[(0, ObjectValue::Number(real(a)))]);
            let radius = binary(
                BinaryOp::Sub,
                reference(0, Property::Value),
                constant(real(1.0)),
            );
            assert_eq!(
                circle(constant(vector(0.0, 0.0, 0.0)), radius).evaluate(&ctx),
                expected_radius.map(|radius| ObjectValue::Circle {
                    center: DVec3::ZERO,
                    radius,
                }),
                "a = {a}"
            );
        }
    }

    #[test]
    fn dependents_read_the_requested_property_of_a_segment_or_circle() {
        let ctx = ctx_with(&[
            (
                0,
                ObjectValue::Segment {
                    start: DVec3::new(1.0, 2.0, 0.0),
                    end: DVec3::new(5.0, 6.0, 0.0),
                },
            ),
            (
                1,
                ObjectValue::Circle {
                    center: DVec3::new(-1.0, 3.0, 0.0),
                    radius: 4.0,
                },
            ),
        ]);
        assert_eq!(
            point(reference(0, Property::Start)).evaluate(&ctx),
            Some(ObjectValue::Point(DVec3::new(1.0, 2.0, 0.0)))
        );
        assert_eq!(
            point(reference(0, Property::End)).evaluate(&ctx),
            Some(ObjectValue::Point(DVec3::new(5.0, 6.0, 0.0)))
        );
        assert_eq!(
            point(reference(1, Property::Center)).evaluate(&ctx),
            Some(ObjectValue::Point(DVec3::new(-1.0, 3.0, 0.0)))
        );
        assert_eq!(
            number(reference(1, Property::Radius)).evaluate(&ctx),
            Some(ObjectValue::Number(real(4.0)))
        );
        assert_eq!(point(reference(1, Property::Radius)).evaluate(&ctx), None);
        assert_eq!(number(reference(0, Property::Start)).evaluate(&ctx), None);
        assert_eq!(point(reference(0, Property::Position)).evaluate(&ctx), None);
    }

    fn ids(ids: &[u64]) -> BTreeSet<ObjectId> {
        ids.iter().map(|id| ObjectId(*id)).collect()
    }

    #[test]
    fn every_field_of_every_construction_is_a_dependency() {
        let later = || reference(9, Property::Position);
        let later_number = || reference(9, Property::Value);
        let origin = || constant(vector(0.0, 0.0, 0.0));
        let one = || constant(real(1.0));
        let cases = [
            ("point position", point(later())),
            ("number value", number(later_number())),
            ("segment start", segment(later(), origin())),
            ("segment end", segment(origin(), later())),
            ("circle center", circle(later(), one())),
            ("circle radius", circle(origin(), later_number())),
        ];
        for (field, construction) in cases {
            assert_eq!(
                construction.dependencies(),
                ids(&[9]),
                "dependency missed in {field}"
            );
        }
    }

    #[test]
    fn constructions_from_constants_have_no_dependencies() {
        let origin = || constant(vector(0.0, 0.0, 0.0));
        let one = || constant(real(1.0));
        assert_eq!(point(origin()).dependencies(), ids(&[]));
        assert_eq!(number(one()).dependencies(), ids(&[]));
        assert_eq!(segment(origin(), origin()).dependencies(), ids(&[]));
        assert_eq!(circle(origin(), one()).dependencies(), ids(&[]));
    }

    #[test]
    fn construction_dependencies_merge_all_fields() {
        assert_eq!(
            segment(
                reference(3, Property::Position),
                reference(9, Property::Position)
            )
            .dependencies(),
            ids(&[3, 9])
        );
        let radius = call(
            BuiltInFn::Norm,
            vec![binary(
                BinaryOp::Sub,
                reference(1, Property::Position),
                reference(0, Property::Position),
            )],
        );
        assert_eq!(
            circle(reference(0, Property::Position), radius).dependencies(),
            ids(&[0, 1])
        );
    }

    #[test]
    fn object_shared_by_both_fields_is_counted_once() {
        let a = || reference(2, Property::Position);
        let shifted = binary(BinaryOp::Add, a(), constant(vector(1.0, 0.0, 0.0)));
        assert_eq!(segment(a(), shifted).dependencies(), ids(&[2]));
    }

    #[test]
    fn self_reference_is_reported_as_a_dependency() {
        let a_moved_by_itself = point(binary(
            BinaryOp::Add,
            reference(0, Property::Position),
            constant(vector(0.1, 0.0, 0.0)),
        ));
        assert_eq!(a_moved_by_itself.dependencies(), ids(&[0]));
    }
}
