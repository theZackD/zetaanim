use glam::DVec3;
use num_complex::Complex;

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Real(f64),
    Complex(Complex<f64>),
    Vector(DVec3),
}
