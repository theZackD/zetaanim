use super::*;

fn real(x: f64) -> Value {
    Value::Real(x)
}

fn complex(re: f64, im: f64) -> Value {
    Value::Complex(Complex::new(re, im))
}

fn vector(x: f64, y: f64, z: f64) -> Value {
    Value::Vector(DVec3::new(x, y, z))
}

#[test]
fn neg_of_each_kind() {
    assert_eq!(real(2.0).neg(), Some(real(-2.0)));
    assert_eq!(complex(1.0, -3.0).neg(), Some(complex(-1.0, 3.0)));
    assert_eq!(vector(1.0, -2.0, 3.0).neg(), Some(vector(-1.0, 2.0, -3.0)));
}

#[test]
fn add_same_kinds() {
    assert_eq!(real(2.0).add(&real(3.0)), Some(real(5.0)));
    assert_eq!(
        complex(1.0, 2.0).add(&complex(3.0, -5.0)),
        Some(complex(4.0, -3.0))
    );
    assert_eq!(
        vector(1.0, 2.0, 3.0).add(&vector(10.0, 20.0, 30.0)),
        Some(vector(11.0, 22.0, 33.0))
    );
}

#[test]
fn add_promotes_real_to_complex_both_ways() {
    assert_eq!(real(2.0).add(&complex(1.0, 4.0)), Some(complex(3.0, 4.0)));
    assert_eq!(complex(1.0, 4.0).add(&real(2.0)), Some(complex(3.0, 4.0)));
}

#[test]
fn complex_sum_with_zero_imaginary_stays_complex() {
    assert_eq!(
        complex(-1.0, 1.0).add(&complex(-1.0, -1.0)),
        Some(complex(-2.0, 0.0))
    );
}

#[test]
fn sub_same_kinds() {
    assert_eq!(real(2.0).sub(&real(5.0)), Some(real(-3.0)));
    assert_eq!(
        complex(1.0, 2.0).sub(&complex(3.0, -5.0)),
        Some(complex(-2.0, 7.0))
    );
    assert_eq!(
        vector(1.0, 2.0, 3.0).sub(&vector(10.0, 20.0, 30.0)),
        Some(vector(-9.0, -18.0, -27.0))
    );
}

#[test]
fn sub_mixed_keeps_operand_order() {
    assert_eq!(real(1.0).sub(&complex(2.0, 1.0)), Some(complex(-1.0, -1.0)));
    assert_eq!(complex(2.0, 1.0).sub(&real(1.0)), Some(complex(1.0, 1.0)));
}

#[test]
fn mixing_numbers_and_vectors_is_undefined() {
    let v = vector(1.0, 2.0, 3.0);
    for n in [real(1.0), complex(1.0, 1.0)] {
        assert_eq!(n.add(&v), None);
        assert_eq!(v.add(&n), None);
        assert_eq!(n.sub(&v), None);
        assert_eq!(v.sub(&n), None);
    }
}

#[test]
fn non_finite_results_are_undefined() {
    assert_eq!(real(f64::MAX).add(&real(f64::MAX)), None);
    assert_eq!(real(f64::INFINITY).sub(&real(f64::INFINITY)), None);
    assert_eq!(complex(f64::MAX, 0.0).add(&complex(f64::MAX, 0.0)), None);
    assert_eq!(complex(0.0, f64::MAX).add(&complex(0.0, f64::MAX)), None);
    assert_eq!(
        vector(0.0, 0.0, f64::MAX).add(&vector(0.0, 0.0, f64::MAX)),
        None
    );
    assert_eq!(real(f64::NAN).add(&real(1.0)), None);
}

#[test]
fn finite_keeps_ordinary_values() {
    assert_eq!(real(1.0e300).add(&real(1.0e300)), Some(real(2.0e300)));
}

fn assert_complex_close(value: Option<Value>, re: f64, im: f64) {
    match value {
        Some(Value::Complex(c)) => {
            assert!(
                (c.re - re).abs() < 1e-12 && (c.im - im).abs() < 1e-12,
                "{c}"
            );
        }
        other => panic!("expected a complex, got {other:?}"),
    }
}

#[test]
fn mul_numbers() {
    assert_eq!(real(2.0).mul(&real(3.0)), Some(real(6.0)));
    assert_eq!(real(2.0).mul(&complex(1.0, 4.0)), Some(complex(2.0, 8.0)));
    assert_eq!(complex(1.0, 4.0).mul(&real(2.0)), Some(complex(2.0, 8.0)));
    assert_eq!(
        complex(0.0, 1.0).mul(&complex(0.0, 1.0)),
        Some(complex(-1.0, 0.0))
    );
}

#[test]
fn mul_scalar_and_vector_both_ways() {
    let v = vector(1.0, -2.0, 3.0);
    assert_eq!(real(2.0).mul(&v), Some(vector(2.0, -4.0, 6.0)));
    assert_eq!(v.mul(&real(2.0)), Some(vector(2.0, -4.0, 6.0)));
}

#[test]
fn mul_ambiguous_vector_products_are_undefined() {
    let v = vector(1.0, 2.0, 3.0);
    assert_eq!(v.mul(&v), None);
    assert_eq!(complex(1.0, 1.0).mul(&v), None);
    assert_eq!(v.mul(&complex(1.0, 1.0)), None);
}

#[test]
fn div_numbers_keep_operand_order() {
    assert_eq!(real(6.0).div(&real(3.0)), Some(real(2.0)));
    assert_eq!(real(3.0).div(&real(6.0)), Some(real(0.5)));
    assert_eq!(complex(2.0, 8.0).div(&real(2.0)), Some(complex(1.0, 4.0)));
    assert_complex_close(real(1.0).div(&complex(0.0, 1.0)), 0.0, -1.0);
    assert_complex_close(complex(1.0, 1.0).div(&complex(1.0, -1.0)), 0.0, 1.0);
}

#[test]
fn div_vector_by_scalar_only() {
    let v = vector(2.0, -4.0, 6.0);
    assert_eq!(v.div(&real(2.0)), Some(vector(1.0, -2.0, 3.0)));
    assert_eq!(real(2.0).div(&v), None);
    assert_eq!(v.div(&v), None);
}

#[test]
fn division_by_zero_is_undefined() {
    assert_eq!(real(1.0).div(&real(0.0)), None);
    assert_eq!(real(0.0).div(&real(0.0)), None);
    assert_eq!(complex(1.0, 1.0).div(&complex(0.0, 0.0)), None);
    assert_eq!(complex(1.0, 1.0).div(&real(0.0)), None);
    assert_eq!(vector(1.0, 2.0, 3.0).div(&real(0.0)), None);
}

#[test]
fn pow_reals() {
    assert_eq!(real(2.0).pow(&real(10.0)), Some(real(1024.0)));
    assert_eq!(real(-2.0).pow(&real(3.0)), Some(real(-8.0)));
    assert_eq!(real(4.0).pow(&real(0.5)), Some(real(2.0)));
    assert_eq!(real(0.0).pow(&real(0.0)), Some(real(1.0)));
}

#[test]
fn pow_strict_in_reals() {
    assert_eq!(real(-8.0).pow(&real(0.5)), None);
    assert_eq!(real(-8.0).pow(&real(1.0 / 3.0)), None);
    assert_eq!(real(0.0).pow(&real(-1.0)), None);
}

#[test]
fn pow_complex_principal_branch() {
    assert_complex_close(complex(-8.0, 0.0).pow(&real(1.0 / 3.0)), 1.0, 3f64.sqrt());
    assert_complex_close(real(-1.0).pow(&complex(0.5, 0.0)), 0.0, 1.0);
    assert_complex_close(
        complex(0.0, 1.0).pow(&complex(0.0, 1.0)),
        (-std::f64::consts::FRAC_PI_2).exp(),
        0.0,
    );
    assert_complex_close(complex(0.0, 2.0).pow(&real(2.0)), -4.0, 0.0);
}

fn has_negative_zero(value: &Value) -> bool {
    let parts = match value {
        Value::Real(r) => vec![*r],
        Value::Complex(c) => vec![c.re, c.im],
        Value::Vector(v) => v.to_array().to_vec(),
    };
    parts.iter().any(|x| *x == 0.0 && x.is_sign_negative())
}

#[test]
fn negating_zero_gives_positive_zero() {
    for zero in [real(0.0), complex(0.0, 0.0), vector(0.0, 0.0, 0.0)] {
        let result = zero.neg().unwrap();
        assert!(!has_negative_zero(&result), "{result:?}");
    }
}

#[test]
fn operations_never_produce_negative_zero() {
    let results = [
        real(-1.0).mul(&real(0.0)),
        complex(4.0, 0.0).neg(),
        complex(0.0, 4.0).neg(),
        real(-1.0).mul(&complex(4.0, 0.0)),
        complex(4.0, 0.0).div(&real(-1.0)),
        vector(2.0, 0.0, 0.0).neg(),
        real(-1.0).mul(&vector(2.0, 0.0, 0.0)),
    ];
    for result in results {
        let value = result.unwrap();
        assert!(!has_negative_zero(&value), "{value:?}");
    }
}

#[test]
fn normalization_keeps_nonzero_signs() {
    assert_eq!(real(3.5).neg(), Some(real(-3.5)));
    assert_eq!(real(-1.0).mul(&complex(4.0, 0.0)), Some(complex(-4.0, 0.0)));
    assert_eq!(vector(2.0, 0.0, 0.0).neg(), Some(vector(-2.0, 0.0, 0.0)));
}

#[test]
fn pow_with_vector_is_undefined() {
    let v = vector(1.0, 2.0, 3.0);
    assert_eq!(v.pow(&real(2.0)), None);
    assert_eq!(real(2.0).pow(&v), None);
}
