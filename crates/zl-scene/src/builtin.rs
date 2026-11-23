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
mod tests;
