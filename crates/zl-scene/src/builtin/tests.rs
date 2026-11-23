use super::*;

fn r(x: f64) -> Value {
    Value::Real(x)
}

fn c(re: f64, im: f64) -> Value {
    Value::Complex(Complex::new(re, im))
}

fn v(x: f64, y: f64, z: f64) -> Value {
    Value::Vector(DVec3::new(x, y, z))
}

fn call(f: BuiltInFn, args: &[Value]) -> Option<Value> {
    f.apply(args)
}

fn real_close(value: Option<Value>, expected: f64) {
    match value {
        Some(Value::Real(x)) => assert!((x - expected).abs() < 1e-12, "{x} != {expected}"),
        other => panic!("expected a real {expected}, got {other:?}"),
    }
}

fn complex_close(value: Option<Value>, re: f64, im: f64) {
    match value {
        Some(Value::Complex(z)) => assert!(
            (z.re - re).abs() < 1e-12 && (z.im - im).abs() < 1e-12,
            "{z} != {re} + {im}i"
        ),
        other => panic!("expected a complex, got {other:?}"),
    }
}

#[test]
fn arity_is_consistent_with_apply() {
    assert_eq!(BuiltInFn::Sin.arity(), 1);
    assert_eq!(BuiltInFn::Atan2.arity(), 2);
    assert_eq!(call(BuiltInFn::Sin, &[]), None);
    assert_eq!(call(BuiltInFn::Sin, &[r(1.0), r(2.0)]), None);
    assert_eq!(call(BuiltInFn::Atan2, &[r(1.0)]), None);
}

#[test]
fn elementary_functions_on_reals() {
    real_close(call(BuiltInFn::Sin, &[r(PI / 2.0)]), 1.0);
    real_close(call(BuiltInFn::Cos, &[r(PI)]), -1.0);
    real_close(call(BuiltInFn::Tan, &[r(PI / 4.0)]), 1.0);
    real_close(call(BuiltInFn::Sqrt, &[r(9.0)]), 3.0);
    real_close(call(BuiltInFn::Exp, &[r(0.0)]), 1.0);
    real_close(call(BuiltInFn::Ln, &[r(std::f64::consts::E)]), 1.0);
    real_close(call(BuiltInFn::Abs, &[r(-3.5)]), 3.5);
}

#[test]
fn real_domains_are_strict() {
    assert_eq!(call(BuiltInFn::Sqrt, &[r(-1.0)]), None);
    assert_eq!(call(BuiltInFn::Ln, &[r(0.0)]), None);
    assert_eq!(call(BuiltInFn::Ln, &[r(-1.0)]), None);
    assert_eq!(call(BuiltInFn::Asin, &[r(2.0)]), None);
    assert_eq!(call(BuiltInFn::Acos, &[r(-2.0)]), None);
    assert_eq!(call(BuiltInFn::Log10, &[r(-10.0)]), None);
}

#[test]
fn complex_versions_follow_the_argument() {
    complex_close(call(BuiltInFn::Sqrt, &[c(-1.0, 0.0)]), 0.0, 1.0);
    complex_close(call(BuiltInFn::Ln, &[c(-1.0, 0.0)]), 0.0, PI);
    complex_close(call(BuiltInFn::Exp, &[c(0.0, PI)]), -1.0, 0.0);
    complex_close(call(BuiltInFn::Cos, &[c(0.0, 1.0)]), 1f64.cosh(), 0.0);
    complex_close(
        call(BuiltInFn::Asin, &[c(2.0, 0.0)]),
        PI / 2.0,
        -(2.0f64 + 3f64.sqrt()).ln(),
    );
    real_close(call(BuiltInFn::Abs, &[c(3.0, 4.0)]), 5.0);
}

#[test]
fn inverse_and_hyperbolic() {
    real_close(call(BuiltInFn::Asin, &[r(1.0)]), PI / 2.0);
    real_close(call(BuiltInFn::Acos, &[r(1.0)]), 0.0);
    real_close(call(BuiltInFn::Atan, &[r(1.0)]), PI / 4.0);
    real_close(call(BuiltInFn::Atan2, &[r(1.0), r(-1.0)]), 3.0 * PI / 4.0);
    real_close(call(BuiltInFn::Atan2, &[r(-1.0), r(-1.0)]), -3.0 * PI / 4.0);
    real_close(call(BuiltInFn::Sinh, &[r(0.0)]), 0.0);
    real_close(call(BuiltInFn::Cosh, &[r(0.0)]), 1.0);
    real_close(call(BuiltInFn::Tanh, &[r(0.0)]), 0.0);
}

#[test]
fn logarithms() {
    real_close(call(BuiltInFn::Log10, &[r(1000.0)]), 3.0);
    real_close(call(BuiltInFn::Log, &[r(8.0), r(2.0)]), 3.0);
    real_close(call(BuiltInFn::Log, &[r(2.0), r(8.0)]), 1.0 / 3.0);
    assert_eq!(call(BuiltInFn::Log, &[r(8.0), r(1.0)]), None);
    complex_close(
        call(BuiltInFn::Log, &[r(-8.0), c(2.0, 0.0)]),
        3.0,
        PI / 2f64.ln(),
    );
}

#[test]
fn odd_roots_of_negatives_are_real() {
    real_close(call(BuiltInFn::Cbrt, &[r(-8.0)]), -2.0);
    real_close(call(BuiltInFn::NthRoot, &[r(-32.0), r(5.0)]), -2.0);
    real_close(call(BuiltInFn::NthRoot, &[r(16.0), r(4.0)]), 2.0);
    assert_eq!(call(BuiltInFn::NthRoot, &[r(-16.0), r(4.0)]), None);
    assert_eq!(call(BuiltInFn::NthRoot, &[r(8.0), r(2.5)]), None);
    assert_eq!(call(BuiltInFn::NthRoot, &[r(8.0), r(0.0)]), None);
    complex_close(
        call(BuiltInFn::NthRoot, &[c(-8.0, 0.0), r(3.0)]),
        1.0,
        3f64.sqrt(),
    );
}

#[test]
fn complex_parts() {
    let z = c(3.0, -4.0);
    assert_eq!(call(BuiltInFn::Re, std::slice::from_ref(&z)), Some(r(3.0)));
    assert_eq!(call(BuiltInFn::Im, std::slice::from_ref(&z)), Some(r(-4.0)));
    assert_eq!(
        call(BuiltInFn::Conj, std::slice::from_ref(&z)),
        Some(c(3.0, 4.0))
    );
    real_close(call(BuiltInFn::Arg, &[c(0.0, 1.0)]), PI / 2.0);
    assert_eq!(call(BuiltInFn::Re, &[r(2.0)]), Some(r(2.0)));
    assert_eq!(call(BuiltInFn::Im, &[r(2.0)]), Some(r(0.0)));
    assert_eq!(call(BuiltInFn::Arg, &[r(-2.0)]), Some(r(PI)));
    assert_eq!(call(BuiltInFn::Arg, &[r(2.0)]), Some(r(0.0)));
    assert_eq!(call(BuiltInFn::Conj, &[r(2.0)]), Some(r(2.0)));
}

#[test]
fn plane_and_complex_conversions() {
    assert_eq!(
        call(BuiltInFn::ToComplex, &[v(1.0, 2.0, 0.0)]),
        Some(c(1.0, 2.0))
    );
    assert_eq!(call(BuiltInFn::ToComplex, &[v(1.0, 2.0, 3.0)]), None);
    assert_eq!(
        call(BuiltInFn::ToComplex, &[v(1.0, 2.0, 1e-17)]),
        Some(c(1.0, 2.0))
    );
    assert_eq!(call(BuiltInFn::ToComplex, &[v(1.0, 2.0, 1e-6)]), None);
    assert_eq!(call(BuiltInFn::ToComplex, &[r(-1.0)]), Some(c(-1.0, 0.0)));
    assert_eq!(
        call(BuiltInFn::ToPoint, &[c(1.0, 2.0)]),
        Some(v(1.0, 2.0, 0.0))
    );
    assert_eq!(call(BuiltInFn::ToPoint, &[r(5.0)]), Some(v(5.0, 0.0, 0.0)));
}

#[test]
fn vector_products_and_norm() {
    let i = v(1.0, 0.0, 0.0);
    let j = v(0.0, 1.0, 0.0);
    assert_eq!(
        call(BuiltInFn::Dot, &[v(1.0, 2.0, 3.0), v(4.0, 5.0, 6.0)]),
        Some(r(32.0))
    );
    assert_eq!(
        call(BuiltInFn::Cross, &[i.clone(), j.clone()]),
        Some(v(0.0, 0.0, 1.0))
    );
    assert_eq!(call(BuiltInFn::Cross, &[j, i]), Some(v(0.0, 0.0, -1.0)));
    assert_eq!(call(BuiltInFn::Norm, &[v(3.0, 4.0, 0.0)]), Some(r(5.0)));
    assert_eq!(call(BuiltInFn::Dot, &[r(1.0), r(2.0)]), None);
    assert_eq!(call(BuiltInFn::Cross, &[v(1.0, 0.0, 0.0), r(2.0)]), None);
}

#[test]
fn rounding_and_comparisons() {
    assert_eq!(call(BuiltInFn::Floor, &[r(-1.5)]), Some(r(-2.0)));
    assert_eq!(call(BuiltInFn::Ceil, &[r(-1.5)]), Some(r(-1.0)));
    assert_eq!(call(BuiltInFn::Round, &[r(2.5)]), Some(r(3.0)));
    assert_eq!(call(BuiltInFn::Sign, &[r(-3.0)]), Some(r(-1.0)));
    assert_eq!(call(BuiltInFn::Sign, &[r(0.0)]), Some(r(0.0)));
    assert_eq!(call(BuiltInFn::Sign, &[r(7.0)]), Some(r(1.0)));
    assert_eq!(call(BuiltInFn::Min, &[r(2.0), r(-1.0)]), Some(r(-1.0)));
    assert_eq!(call(BuiltInFn::Max, &[r(2.0), r(-1.0)]), Some(r(2.0)));
    assert_eq!(call(BuiltInFn::Floor, &[c(1.5, 1.0)]), None);
}

#[test]
fn chop_cleans_tiny_values_only() {
    assert_eq!(call(BuiltInFn::Chop, &[r(1e-17)]), Some(r(0.0)));
    assert_eq!(call(BuiltInFn::Chop, &[r(1e-3)]), Some(r(1e-3)));
    assert_eq!(call(BuiltInFn::Chop, &[c(2.0, 1e-16)]), Some(c(2.0, 0.0)));
    assert_eq!(
        call(BuiltInFn::Chop, &[v(1e-12, 1.0, -1e-15)]),
        Some(v(0.0, 1.0, 0.0))
    );
}

#[test]
fn integer_functions() {
    assert_eq!(call(BuiltInFn::Factorial, &[r(0.0)]), Some(r(1.0)));
    assert_eq!(call(BuiltInFn::Factorial, &[r(5.0)]), Some(r(120.0)));
    assert_eq!(call(BuiltInFn::Factorial, &[r(2.5)]), None);
    assert_eq!(call(BuiltInFn::Factorial, &[r(-1.0)]), None);
    assert_eq!(call(BuiltInFn::Factorial, &[r(200.0)]), None);
    assert_eq!(call(BuiltInFn::Binomial, &[r(5.0), r(2.0)]), Some(r(10.0)));
    assert_eq!(call(BuiltInFn::Binomial, &[r(5.0), r(0.0)]), Some(r(1.0)));
    assert_eq!(call(BuiltInFn::Binomial, &[r(5.0), r(5.0)]), Some(r(1.0)));
    assert_eq!(call(BuiltInFn::Binomial, &[r(5.0), r(7.0)]), Some(r(0.0)));
    assert_eq!(
        call(BuiltInFn::Binomial, &[r(52.0), r(5.0)]),
        Some(r(2598960.0))
    );
    assert_eq!(call(BuiltInFn::Binomial, &[r(5.0), r(1.5)]), None);
}

#[test]
fn huge_integer_arguments_return_quickly() {
    assert!(call(BuiltInFn::Factorial, &[r(170.0)]).is_some());
    assert_eq!(call(BuiltInFn::Factorial, &[r(171.0)]), None);
    assert_eq!(call(BuiltInFn::Factorial, &[r(1e12)]), None);
    assert_eq!(call(BuiltInFn::Binomial, &[r(1e13), r(5e12)]), None);
    assert_eq!(call(BuiltInFn::Binomial, &[r(1e13), r(1.0)]), Some(r(1e13)));
}

#[test]
fn modulo_is_the_euclidean_remainder() {
    assert_eq!(call(BuiltInFn::Mod, &[r(7.0), r(3.0)]), Some(r(1.0)));
    assert_eq!(call(BuiltInFn::Mod, &[r(-7.0), r(3.0)]), Some(r(2.0)));
    assert_eq!(call(BuiltInFn::Mod, &[r(7.0), r(-3.0)]), Some(r(1.0)));
    assert_eq!(call(BuiltInFn::Mod, &[r(-7.0), r(-3.0)]), Some(r(2.0)));
    assert_eq!(call(BuiltInFn::Mod, &[r(1e17 + 2.0), r(3.0)]), Some(r(1.0)));
    real_close(call(BuiltInFn::Mod, &[r(7.0), r(2.0 * PI)]), 7.0 - 2.0 * PI);
    assert_eq!(call(BuiltInFn::Mod, &[r(7.0), r(0.0)]), None);
}

#[test]
fn vectors_are_rejected_by_number_functions() {
    let p = v(1.0, 2.0, 3.0);
    for f in [
        BuiltInFn::Sin,
        BuiltInFn::Sqrt,
        BuiltInFn::Ln,
        BuiltInFn::Abs,
        BuiltInFn::Re,
        BuiltInFn::Floor,
        BuiltInFn::Factorial,
    ] {
        assert_eq!(call(f, std::slice::from_ref(&p)), None, "{f:?}");
    }
}

#[test]
fn near_real_complexes_are_accepted_where_a_real_is_required() {
    let two = c(2.0, 1e-17);
    assert_eq!(
        call(BuiltInFn::Floor, std::slice::from_ref(&two)),
        Some(r(2.0))
    );
    assert_eq!(call(BuiltInFn::Sign, &[c(-3.0, 0.0)]), Some(r(-1.0)));
    assert_eq!(call(BuiltInFn::Max, &[two.clone(), r(1.0)]), Some(r(2.0)));
    assert_eq!(
        call(BuiltInFn::Mod, &[c(7.0, 0.0), c(3.0, 0.0)]),
        Some(r(1.0))
    );
    assert_eq!(call(BuiltInFn::Factorial, &[c(5.0, 0.0)]), Some(r(120.0)));
    assert_eq!(
        call(BuiltInFn::Binomial, &[c(5.0, 0.0), two.clone()]),
        Some(r(10.0))
    );
    real_close(call(BuiltInFn::NthRoot, &[r(-8.0), c(3.0, 0.0)]), -2.0);
    real_close(call(BuiltInFn::Atan2, &[c(1.0, 0.0), r(1.0)]), PI / 4.0);
}

#[test]
fn truly_complex_values_are_rejected_where_a_real_is_required() {
    let z = c(2.0, 0.5);
    assert_eq!(call(BuiltInFn::Floor, std::slice::from_ref(&z)), None);
    assert_eq!(call(BuiltInFn::Min, &[z.clone(), r(1.0)]), None);
    assert_eq!(call(BuiltInFn::Factorial, std::slice::from_ref(&z)), None);
    assert_eq!(call(BuiltInFn::NthRoot, &[r(8.0), z]), None);
}

#[test]
fn near_real_tolerance_scales_with_magnitude() {
    assert_eq!(call(BuiltInFn::Floor, &[c(1e12, 1e-3)]), Some(r(1e12)));
    assert_eq!(call(BuiltInFn::Floor, &[c(0.5, 1e-3)]), None);
}
