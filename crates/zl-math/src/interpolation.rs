pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

pub fn progression(t: f64, start: f64, duration: f64) -> f64 {
    if duration <= 0.0 {
        if t >= start { 1.0 } else { 0.0 }
    } else {
        let elapsed = t - start;
        (elapsed / duration).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp_bounds_and_midpoint() {
        assert_eq!(lerp(2.0, 6.0, 0.0), 2.0);
        assert_eq!(lerp(2.0, 6.0, 1.0), 6.0);
        assert_eq!(lerp(2.0, 6.0, 0.5), 4.0);
        assert_eq!(lerp(6.0, 2.0, 0.25), 5.0);
    }

    #[test]
    fn progression_at_different_points() {
        assert_eq!(progression(0.0, 4.0, 10.0), 0.0);
        assert_eq!(progression(0.0, 4.0, 0.0), 0.0);
        assert_eq!(progression(5.0, 4.0, 0.0), 1.0);
        assert_eq!(progression(6.5, 1.0, 11.0), 0.5);
        assert_eq!(progression(3.0, 2.0, 4.0), 0.25);
        assert_eq!(progression(7.0, 2.0, 4.0), 1.0);
    }

    #[test]
    fn progression_exact_boundaries() {
        assert_eq!(progression(2.0, 2.0, 4.0), 0.0);
        assert_eq!(progression(6.0, 2.0, 4.0), 1.0);
        assert_eq!(progression(1.999, 2.0, 4.0), 0.0);
        assert_eq!(progression(100.0, 2.0, 4.0), 1.0);
    }

    #[test]
    fn progression_instant_animation() {
        assert_eq!(progression(3.999, 4.0, 0.0), 0.0);
        assert_eq!(progression(4.0, 4.0, 0.0), 1.0);
        assert_eq!(progression(1000.0, 4.0, 0.0), 1.0);
    }

    #[test]
    fn progression_stays_in_unit_interval_and_never_decreases() {
        let mut previous = 0.0;
        for i in 0..=1000 {
            let t = i as f64 / 100.0;
            let p = progression(t, 2.5, 3.0);
            assert!((0.0..=1.0).contains(&p), "progression({t}) = {p}");
            assert!(p >= previous, "decreases at t = {t}");
            previous = p;
        }
    }

    #[test]
    fn progression_is_linear_inside_the_animation() {
        for k in 0..=10 {
            let fraction = k as f64 / 10.0;
            let t = 2.0 + fraction * 4.0;
            assert!((progression(t, 2.0, 4.0) - fraction).abs() < 1e-12);
        }
    }

    #[test]
    fn progression_with_start_at_zero_and_late_start() {
        assert_eq!(progression(0.5, 0.0, 1.0), 0.5);
        assert_eq!(progression(3600.5, 3600.0, 1.0), 0.5);
    }

    #[test]
    fn progression_with_tiny_duration() {
        assert_eq!(progression(2.0, 2.0, 1e-9), 0.0);
        assert_eq!(progression(2.0 + 1e-6, 2.0, 1e-9), 1.0);
    }

    #[test]
    fn progression_negative_duration_is_instant() {
        assert_eq!(progression(1.0, 2.0, -4.0), 0.0);
        assert_eq!(progression(2.0, 2.0, -4.0), 1.0);
        assert_eq!(progression(10.0, 2.0, -4.0), 1.0);
    }
}
