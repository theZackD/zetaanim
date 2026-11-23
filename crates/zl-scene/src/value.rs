use glam::DVec3;
use num_complex::Complex;

pub(crate) const NEAR_REAL_TOLERANCE: f64 = 1e-10;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Real(f64),
    Complex(Complex<f64>),
    Vector(DVec3),
}

impl Value {
    pub fn neg(&self) -> Option<Value> {
        let res = match self {
            Value::Real(r) => Value::Real(-*r),
            Value::Complex(z) => Value::Complex(-z),
            Value::Vector(v) => Value::Vector(-v),
        };
        res.finite()
    }

    pub fn add(&self, other: &Value) -> Option<Value> {
        let res = match (self, other) {
            (Value::Real(r1), Value::Real(r2)) => Some(Value::Real(r1 + r2)),
            (Value::Real(r), Value::Complex(c)) | (Value::Complex(c), Value::Real(r)) => {
                Some(Value::Complex(r + c))
            }
            (Value::Complex(c1), Value::Complex(c2)) => Some(Value::Complex(c1 + c2)),
            (Value::Vector(v1), Value::Vector(v2)) => Some(Value::Vector(v1 + v2)),
            _ => None,
        };
        res?.finite()
    }
    pub fn sub(&self, other: &Value) -> Option<Value> {
        let res = match (self, other) {
            (Value::Real(r1), Value::Real(r2)) => Some(Value::Real(r1 - r2)),
            (Value::Real(r), Value::Complex(c)) => Some(Value::Complex(r - c)),
            (Value::Complex(c), Value::Real(r)) => Some(Value::Complex(c - r)),
            (Value::Complex(c1), Value::Complex(c2)) => Some(Value::Complex(c1 - c2)),
            (Value::Vector(v1), Value::Vector(v2)) => Some(Value::Vector(v1 - v2)),
            _ => None,
        };
        res?.finite()
    }

    pub fn mul(&self, other: &Value) -> Option<Value> {
        let res = match (self, other) {
            (Value::Real(r1), Value::Real(r2)) => Some(Value::Real(r1 * r2)),
            (Value::Real(r), Value::Complex(c)) | (Value::Complex(c), Value::Real(r)) => {
                Some(Value::Complex(r * c))
            }

            (Value::Complex(c1), Value::Complex(c2)) => Some(Value::Complex(c1 * c2)),
            (Value::Vector(v), Value::Real(r)) | (Value::Real(r), Value::Vector(v)) => {
                Some(Value::Vector(r * v))
            }
            _ => None,
        };
        res?.finite()
    }

    pub fn div(&self, other: &Value) -> Option<Value> {
        let res = match (self, other) {
            (Value::Real(r1), Value::Real(r2)) => Some(Value::Real(r1 / r2)),
            (Value::Real(r), Value::Complex(c)) => Some(Value::Complex(r / c)),
            (Value::Complex(c), Value::Real(r)) => Some(Value::Complex(c / r)),
            (Value::Complex(c1), Value::Complex(c2)) => Some(Value::Complex(c1 / c2)),
            (Value::Vector(v), Value::Real(r)) => Some(Value::Vector(v / r)),
            _ => None,
        };
        res?.finite()
    }

    pub fn pow(&self, other: &Value) -> Option<Value> {
        let res = match (self, other) {
            (Value::Real(r1), Value::Real(r2)) => Some(Value::Real(r1.powf(*r2))),
            (Value::Real(r), Value::Complex(c)) => {
                Some(Value::Complex(Complex::new(*r, 0.0).powc(*c)))
            }
            (Value::Complex(c), Value::Real(r)) => {
                Some(Value::Complex(c.powc(Complex::new(*r, 0.0))))
            }
            (Value::Complex(c1), Value::Complex(c2)) => Some(Value::Complex(c1.powc(*c2))),
            _ => None,
        };
        res?.finite()
    }

    pub(crate) fn finite(self) -> Option<Value> {
        match self {
            Value::Real(r) if r.is_finite() => Some(Value::Real(without_negative_zero(r))),
            Value::Complex(c) if c.is_finite() => Some(Value::Complex(Complex::new(
                without_negative_zero(c.re),
                without_negative_zero(c.im),
            ))),
            Value::Vector(v) if v.is_finite() => Some(Value::Vector(v.map(without_negative_zero))),
            Value::Real(_) | Value::Complex(_) | Value::Vector(_) => None,
        }
    }

    pub(crate) fn as_real(&self) -> Option<f64> {
        match self {
            Value::Real(x) => Some(*x),
            Value::Complex(z) if z.im.abs() <= NEAR_REAL_TOLERANCE * z.re.abs().max(1.0) => {
                Some(z.re)
            }
            Value::Complex(_) | Value::Vector(_) => None,
        }
    }

    pub(crate) fn as_vector(&self) -> Option<DVec3> {
        match self {
            Value::Vector(v) => Some(*v),
            Value::Real(_) | Value::Complex(_) => None,
        }
    }
}

fn without_negative_zero(x: f64) -> f64 {
    if x == 0.0 { 0.0 } else { x }
}

#[cfg(test)]
mod tests;
