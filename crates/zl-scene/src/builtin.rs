use std::f64::consts::PI;

use glam::DVec3;
use num_complex::Complex;

use crate::{BuiltInFn, NEAR_REAL_TOLERANCE, Value};

const CHOP_TOLERANCE: f64 = 1e-10;
const MAX_FACTORIAL_ARG: f64 = 170.0;

type RealFn = fn(f64) -> f64;
type ComplexFn = fn(Complex<f64>) -> Complex<f64>;

impl BuiltInFn {
    pub fn arity(&self) -> usize {
        match self {
            BuiltInFn::Atan2
            | BuiltInFn::Log
            | BuiltInFn::NthRoot
            | BuiltInFn::Dot
            | BuiltInFn::Cross
            | BuiltInFn::Min
            | BuiltInFn::Max
            | BuiltInFn::Mod
            | BuiltInFn::Binomial => 2,
            BuiltInFn::Sin
            | BuiltInFn::Cos
            | BuiltInFn::Tan
            | BuiltInFn::Sqrt
            | BuiltInFn::Exp
            | BuiltInFn::Ln
            | BuiltInFn::Abs
            | BuiltInFn::Asin
            | BuiltInFn::Acos
            | BuiltInFn::Atan
            | BuiltInFn::Sinh
            | BuiltInFn::Cosh
            | BuiltInFn::Tanh
            | BuiltInFn::Log10
            | BuiltInFn::Cbrt
            | BuiltInFn::Re
            | BuiltInFn::Im
            | BuiltInFn::Arg
            | BuiltInFn::Conj
            | BuiltInFn::ToComplex
            | BuiltInFn::ToPoint
            | BuiltInFn::Norm
            | BuiltInFn::Floor
            | BuiltInFn::Ceil
            | BuiltInFn::Round
            | BuiltInFn::Sign
            | BuiltInFn::Chop
            | BuiltInFn::Factorial => 1,
        }
    }

    pub fn apply(&self, args: &[Value]) -> Option<Value> {
        if args.len() != self.arity() {
            return None;
        }
        let a = &args[0];
        let res = match self {
            BuiltInFn::Sin => real_or_complex(a, f64::sin, Complex::sin),
            BuiltInFn::Cos => real_or_complex(a, f64::cos, Complex::cos),
            BuiltInFn::Tan => real_or_complex(a, f64::tan, Complex::tan),
            BuiltInFn::Sqrt => real_or_complex(a, f64::sqrt, Complex::sqrt),
            BuiltInFn::Exp => real_or_complex(a, f64::exp, Complex::exp),
            BuiltInFn::Ln => real_or_complex(a, f64::ln, Complex::ln),
            BuiltInFn::Asin => real_or_complex(a, f64::asin, Complex::asin),
            BuiltInFn::Acos => real_or_complex(a, f64::acos, Complex::acos),
            BuiltInFn::Atan => real_or_complex(a, f64::atan, Complex::atan),
            BuiltInFn::Sinh => real_or_complex(a, f64::sinh, Complex::sinh),
            BuiltInFn::Cosh => real_or_complex(a, f64::cosh, Complex::cosh),
            BuiltInFn::Tanh => real_or_complex(a, f64::tanh, Complex::tanh),
            BuiltInFn::Log10 => real_or_complex(a, f64::log10, Complex::log10),
            BuiltInFn::Cbrt => real_or_complex(a, f64::cbrt, Complex::cbrt),
            BuiltInFn::Abs => match a {
                Value::Real(x) => Some(Value::Real(x.abs())),
                Value::Complex(z) => Some(Value::Real(z.norm())),
                Value::Vector(_) => None,
            },
            BuiltInFn::Atan2 => two_reals(args, f64::atan2),
            BuiltInFn::Log => log(a, &args[1]),
            BuiltInFn::NthRoot => nth_root(a, &args[1]),
            BuiltInFn::Re => match a {
                Value::Real(x) => Some(Value::Real(*x)),
                Value::Complex(z) => Some(Value::Real(z.re)),
                Value::Vector(_) => None,
            },
            BuiltInFn::Im => match a {
                Value::Real(_) => Some(Value::Real(0.0)),
                Value::Complex(z) => Some(Value::Real(z.im)),
                Value::Vector(_) => None,
            },
            BuiltInFn::Arg => match a {
                Value::Real(x) => Some(Value::Real(if *x < 0.0 { PI } else { 0.0 })),
                Value::Complex(z) => Some(Value::Real(z.arg())),
                Value::Vector(_) => None,
            },
            BuiltInFn::Conj => match a {
                Value::Real(x) => Some(Value::Real(*x)),
                Value::Complex(z) => Some(Value::Complex(z.conj())),
                Value::Vector(_) => None,
            },
            BuiltInFn::ToComplex => match a {
                Value::Real(x) => Some(Value::Complex(Complex::new(*x, 0.0))),
                Value::Complex(z) => Some(Value::Complex(*z)),
                Value::Vector(v)
                    if v.z.abs() <= NEAR_REAL_TOLERANCE * v.x.abs().max(v.y.abs()).max(1.0) =>
                {
                    Some(Value::Complex(Complex::new(v.x, v.y)))
                }
                Value::Vector(_) => None,
            },
            BuiltInFn::ToPoint => match a {
                Value::Real(x) => Some(Value::Vector(DVec3::new(*x, 0.0, 0.0))),
                Value::Complex(z) => Some(Value::Vector(DVec3::new(z.re, z.im, 0.0))),
                Value::Vector(v) => Some(Value::Vector(*v)),
            },
            BuiltInFn::Dot => match (a, &args[1]) {
                (Value::Vector(u), Value::Vector(v)) => Some(Value::Real(u.dot(*v))),
                _ => None,
            },
            BuiltInFn::Cross => match (a, &args[1]) {
                (Value::Vector(u), Value::Vector(v)) => Some(Value::Vector(u.cross(*v))),
                _ => None,
            },
            BuiltInFn::Norm => match a {
                Value::Real(x) => Some(Value::Real(x.abs())),
                Value::Complex(z) => Some(Value::Real(z.norm())),
                Value::Vector(v) => Some(Value::Real(v.length())),
            },
            BuiltInFn::Floor => real_only(a, f64::floor),
            BuiltInFn::Ceil => real_only(a, f64::ceil),
            BuiltInFn::Round => real_only(a, f64::round),
            BuiltInFn::Sign => real_only(a, sign),
            BuiltInFn::Min => two_reals(args, f64::min),
            BuiltInFn::Max => two_reals(args, f64::max),
            BuiltInFn::Chop => Some(match a {
                Value::Real(x) => Value::Real(chop(*x)),
                Value::Complex(z) => Value::Complex(Complex::new(chop(z.re), chop(z.im))),
                Value::Vector(v) => Value::Vector(DVec3::new(chop(v.x), chop(v.y), chop(v.z))),
            }),
            BuiltInFn::Factorial => factorial(a.as_real()?).map(Value::Real),
            BuiltInFn::Mod => two_reals(args, f64::rem_euclid),
            BuiltInFn::Binomial => binomial(a.as_real()?, args[1].as_real()?).map(Value::Real),
        };
        res?.finite()
    }
}

fn real_or_complex(value: &Value, on_real: RealFn, on_complex: ComplexFn) -> Option<Value> {
    match value {
        Value::Real(x) => Some(Value::Real(on_real(*x))),
        Value::Complex(z) => Some(Value::Complex(on_complex(*z))),
        Value::Vector(_) => None,
    }
}

fn real_only(value: &Value, f: RealFn) -> Option<Value> {
    Some(Value::Real(f(value.as_real()?)))
}

fn two_reals(args: &[Value], f: fn(f64, f64) -> f64) -> Option<Value> {
    Some(Value::Real(f(args[0].as_real()?, args[1].as_real()?)))
}

fn as_complex(value: &Value) -> Option<Complex<f64>> {
    match value {
        Value::Real(x) => Some(Complex::new(*x, 0.0)),
        Value::Complex(z) => Some(*z),
        Value::Vector(_) => None,
    }
}

fn log(x: &Value, base: &Value) -> Option<Value> {
    match (x, base) {
        (Value::Real(x), Value::Real(b)) => Some(Value::Real(x.ln() / b.ln())),
        _ => Some(Value::Complex(as_complex(x)?.ln() / as_complex(base)?.ln())),
    }
}

fn nth_root(x: &Value, n: &Value) -> Option<Value> {
    let n = &n.as_real()?;
    if n.fract() != 0.0 || *n < 1.0 {
        return None;
    }
    match x {
        Value::Real(x) if *x < 0.0 => {
            if *n % 2.0 == 1.0 {
                Some(Value::Real(-(-x).powf(1.0 / n)))
            } else {
                None
            }
        }
        Value::Real(x) => Some(Value::Real(x.powf(1.0 / n))),
        Value::Complex(z) => Some(Value::Complex(z.powf(1.0 / n))),
        Value::Vector(_) => None,
    }
}

fn sign(x: f64) -> f64 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

fn chop(x: f64) -> f64 {
    if x.abs() < CHOP_TOLERANCE { 0.0 } else { x }
}

fn factorial(n: f64) -> Option<f64> {
    if n.fract() != 0.0 || n < 0.0 || n > MAX_FACTORIAL_ARG {
        return None;
    }
    let mut product = 1.0;
    let mut k = 2.0;
    while k <= n {
        product *= k;
        k += 1.0;
    }
    Some(product)
}

fn binomial(n: f64, k: f64) -> Option<f64> {
    if n.fract() != 0.0 || k.fract() != 0.0 || n < 0.0 || k < 0.0 {
        return None;
    }
    if k > n {
        return Some(0.0);
    }
    let k = k.min(n - k);
    let mut result = 1.0;
    let mut i = 1.0;
    while i <= k {
        result = result * (n - k + i) / i;
        if !result.is_finite() {
            return None;
        }
        i += 1.0;
    }
    Some(result.round())
}

#[cfg(test)]
mod tests {
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
}
