pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

pub fn linear(t: f64) -> f64 {
    t
}

pub fn smooth(t: f64) -> f64 {
    3.0 * t * t - 2.0 * t * t * t
}

pub fn ease_in(t: f64) -> f64 {
    t * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works_lerp() {
        let a = lerp(0.0, 1.0, 0.0);
        let b = lerp(0.0, 1.0, 1.0);
        let c = lerp(0.0, 1.0, 0.5);
        assert_eq!(a, 0.0);
        assert_eq!(b, 1.0);
        assert_eq!(c, 0.5);
        assert_eq!(lerp(6.0, 2.0, 0.25), 5.0);
    }
    #[test]
    fn works_linear() {
        let a = linear(0.0);
        let b = linear(1.0);
        let c = linear(0.5);

        assert_eq!(a, 0.0);
        assert_eq!(b, 1.0);
        assert_eq!(c, 0.5);
    }
    #[test]
    fn works_smooth() {
        let a = smooth(0.0);
        let b = smooth(1.0);
        let c = smooth(0.5);

        assert_eq!(a, 0.0);
        assert_eq!(b, 1.0);
        assert_eq!(c, 0.5);
    }
    #[test]
    fn works_ease_in() {
        let a = ease_in(0.0);
        let b = ease_in(1.0);
        let c = ease_in(0.5);

        assert_eq!(a, 0.0);
        assert_eq!(b, 1.0);
        assert_eq!(c, 0.25);
    }
}
