use std::f64::consts::PI;

const BACK_C1: f64 = 1.70158;
const BACK_C2: f64 = BACK_C1 * 1.525;
const BACK_C3: f64 = BACK_C1 + 1.0;
const ELASTIC_C4: f64 = 2.0 * PI / 3.0;
const ELASTIC_C5: f64 = 2.0 * PI / 4.5;
const BOUNCE_N1: f64 = 7.5625;
const BOUNCE_D1: f64 = 2.75;

pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

pub fn linear(t: f64) -> f64 {
    t
}

pub fn smooth(t: f64) -> f64 {
    t * t * t * (t * (6.0 * t - 15.0) + 10.0)
}

pub fn ease_in_sine(t: f64) -> f64 {
    1.0 - (PI * t / 2.0).cos()
}

pub fn ease_out_sine(t: f64) -> f64 {
    (PI * t / 2.0).sin()
}

pub fn ease_in_out_sine(t: f64) -> f64 {
    (1.0 - (PI * t).cos()) / 2.0
}

pub fn ease_in_quad(t: f64) -> f64 {
    t * t
}

pub fn ease_out_quad(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(2)
}

pub fn ease_in_out_quad(t: f64) -> f64 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (2.0 - 2.0 * t).powi(2) / 2.0
    }
}

pub fn ease_in_cubic(t: f64) -> f64 {
    t * t * t
}

pub fn ease_out_cubic(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(3)
}

pub fn ease_in_out_cubic(t: f64) -> f64 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (2.0 - 2.0 * t).powi(3) / 2.0
    }
}

pub fn ease_in_expo(t: f64) -> f64 {
    if t <= 0.0 {
        0.0
    } else {
        2.0_f64.powf(10.0 * t - 10.0)
    }
}

pub fn ease_out_expo(t: f64) -> f64 {
    if t >= 1.0 {
        1.0
    } else {
        1.0 - 2.0_f64.powf(-10.0 * t)
    }
}

pub fn ease_in_out_expo(t: f64) -> f64 {
    if t <= 0.0 {
        0.0
    } else if t >= 1.0 {
        1.0
    } else if t < 0.5 {
        2.0_f64.powf(20.0 * t - 10.0) / 2.0
    } else {
        (2.0 - 2.0_f64.powf(-20.0 * t + 10.0)) / 2.0
    }
}

pub fn ease_in_back(t: f64) -> f64 {
    BACK_C3 * t * t * t - BACK_C1 * t * t
}

pub fn ease_out_back(t: f64) -> f64 {
    let u = t - 1.0;
    1.0 + BACK_C3 * u * u * u + BACK_C1 * u * u
}

pub fn ease_in_out_back(t: f64) -> f64 {
    if t < 0.5 {
        let u = 2.0 * t;
        u * u * ((BACK_C2 + 1.0) * u - BACK_C2) / 2.0
    } else {
        let u = 2.0 * t - 2.0;
        (u * u * ((BACK_C2 + 1.0) * u + BACK_C2) + 2.0) / 2.0
    }
}

pub fn ease_in_elastic(t: f64) -> f64 {
    if t <= 0.0 {
        0.0
    } else if t >= 1.0 {
        1.0
    } else {
        -(2.0_f64.powf(10.0 * t - 10.0)) * ((10.0 * t - 10.75) * ELASTIC_C4).sin()
    }
}

pub fn ease_out_elastic(t: f64) -> f64 {
    if t <= 0.0 {
        0.0
    } else if t >= 1.0 {
        1.0
    } else {
        2.0_f64.powf(-10.0 * t) * ((10.0 * t - 0.75) * ELASTIC_C4).sin() + 1.0
    }
}

pub fn ease_in_out_elastic(t: f64) -> f64 {
    if t <= 0.0 {
        0.0
    } else if t >= 1.0 {
        1.0
    } else if t < 0.5 {
        -(2.0_f64.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * ELASTIC_C5).sin()) / 2.0
    } else {
        2.0_f64.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * ELASTIC_C5).sin() / 2.0 + 1.0
    }
}

pub fn ease_out_bounce(t: f64) -> f64 {
    if t < 1.0 / BOUNCE_D1 {
        BOUNCE_N1 * t * t
    } else if t < 2.0 / BOUNCE_D1 {
        let u = t - 1.5 / BOUNCE_D1;
        BOUNCE_N1 * u * u + 0.75
    } else if t < 2.5 / BOUNCE_D1 {
        let u = t - 2.25 / BOUNCE_D1;
        BOUNCE_N1 * u * u + 0.9375
    } else {
        let u = t - 2.625 / BOUNCE_D1;
        BOUNCE_N1 * u * u + 0.984375
    }
}

pub fn ease_in_bounce(t: f64) -> f64 {
    1.0 - ease_out_bounce(1.0 - t)
}

pub fn ease_in_out_bounce(t: f64) -> f64 {
    if t < 0.5 {
        (1.0 - ease_out_bounce(1.0 - 2.0 * t)) / 2.0
    } else {
        (1.0 + ease_out_bounce(2.0 * t - 1.0)) / 2.0
    }
}

pub fn there_and_back(t: f64) -> f64 {
    if t < 0.5 {
        smooth(2.0 * t)
    } else {
        smooth(2.0 - 2.0 * t)
    }
}

pub fn there_and_back_with_pause(t: f64, pause_ratio: f64) -> f64 {
    let a = 2.0 / (1.0 - pause_ratio);
    if t < 0.5 - pause_ratio / 2.0 {
        smooth(a * t)
    } else if t < 0.5 + pause_ratio / 2.0 {
        1.0
    } else {
        smooth(a - a * t)
    }
}

pub fn rush_into(t: f64) -> f64 {
    2.0 * smooth(t / 2.0)
}

pub fn rush_from(t: f64) -> f64 {
    2.0 * smooth(t / 2.0 + 0.5) - 1.0
}

pub fn double_smooth(t: f64) -> f64 {
    if t < 0.5 {
        smooth(2.0 * t) / 2.0
    } else {
        (1.0 + smooth(2.0 * t - 1.0)) / 2.0
    }
}

pub fn wiggle(t: f64, wiggles: f64) -> f64 {
    there_and_back(t) * (wiggles * PI * t).sin()
}

pub fn lingering(t: f64) -> f64 {
    (t / 0.8).min(1.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    Linear,
    Smooth,
    InSine,
    OutSine,
    InOutSine,
    InQuad,
    OutQuad,
    InOutQuad,
    InCubic,
    OutCubic,
    InOutCubic,
    InExpo,
    OutExpo,
    InOutExpo,
    InBack,
    OutBack,
    InOutBack,
    InElastic,
    OutElastic,
    InOutElastic,
    InBounce,
    OutBounce,
    InOutBounce,
    ThereAndBack,
    ThereAndBackPause { pause_ratio: f64 },
    RushInto,
    RushFrom,
    DoubleSmooth,
    Wiggle { wiggles: f64 },
    Lingering,
}

impl Easing {
    pub fn apply(&self, t: f64) -> f64 {
        match self {
            Easing::Smooth => smooth(t),
            Easing::Linear => linear(t),
            Easing::InSine => ease_in_sine(t),
            Easing::OutSine => ease_out_sine(t),
            Easing::InOutSine => ease_in_out_sine(t),
            Easing::InQuad => ease_in_quad(t),
            Easing::OutQuad => ease_out_quad(t),
            Easing::InOutQuad => ease_in_out_quad(t),
            Easing::InCubic => ease_in_cubic(t),
            Easing::OutCubic => ease_out_cubic(t),
            Easing::InOutCubic => ease_in_out_cubic(t),
            Easing::InExpo => ease_in_expo(t),
            Easing::OutExpo => ease_out_expo(t),
            Easing::InOutExpo => ease_in_out_expo(t),
            Easing::InBack => ease_in_back(t),
            Easing::OutBack => ease_out_back(t),
            Easing::InOutBack => ease_in_out_back(t),
            Easing::InElastic => ease_in_elastic(t),
            Easing::OutElastic => ease_out_elastic(t),
            Easing::InOutElastic => ease_in_out_elastic(t),
            Easing::InBounce => ease_in_bounce(t),
            Easing::OutBounce => ease_out_bounce(t),
            Easing::InOutBounce => ease_in_out_bounce(t),
            Easing::ThereAndBack => there_and_back(t),
            Easing::ThereAndBackPause { pause_ratio } => there_and_back_with_pause(t, *pause_ratio),
            Easing::RushInto => rush_into(t),
            Easing::RushFrom => rush_from(t),
            Easing::DoubleSmooth => double_smooth(t),
            Easing::Wiggle { wiggles } => wiggle(t, *wiggles),
            Easing::Lingering => lingering(t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Curve = fn(f64) -> f64;

    const EPS: f64 = 1e-12;

    const STANDARD: [Curve; 23] = [
        linear,
        smooth,
        ease_in_sine,
        ease_out_sine,
        ease_in_out_sine,
        ease_in_quad,
        ease_out_quad,
        ease_in_out_quad,
        ease_in_cubic,
        ease_out_cubic,
        ease_in_out_cubic,
        ease_in_expo,
        ease_out_expo,
        ease_in_out_expo,
        ease_in_back,
        ease_out_back,
        ease_in_out_back,
        ease_in_elastic,
        ease_out_elastic,
        ease_in_out_elastic,
        ease_in_bounce,
        ease_out_bounce,
        ease_in_out_bounce,
    ];

    const IN_OUT: [Curve; 9] = [
        smooth,
        ease_in_out_sine,
        ease_in_out_quad,
        ease_in_out_cubic,
        ease_in_out_expo,
        ease_in_out_back,
        ease_in_out_elastic,
        ease_in_out_bounce,
        double_smooth,
    ];

    const IN_OUT_PAIRS: [(Curve, Curve); 7] = [
        (ease_in_sine, ease_out_sine),
        (ease_in_quad, ease_out_quad),
        (ease_in_cubic, ease_out_cubic),
        (ease_in_expo, ease_out_expo),
        (ease_in_back, ease_out_back),
        (ease_in_elastic, ease_out_elastic),
        (ease_in_bounce, ease_out_bounce),
    ];

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPS
    }

    fn samples() -> impl Iterator<Item = f64> {
        (0..=100).map(|i| i as f64 / 100.0)
    }

    #[test]
    fn lerp_bounds_and_midpoint() {
        assert_eq!(lerp(2.0, 6.0, 0.0), 2.0);
        assert_eq!(lerp(2.0, 6.0, 1.0), 6.0);
        assert_eq!(lerp(2.0, 6.0, 0.5), 4.0);
        assert_eq!(lerp(6.0, 2.0, 0.25), 5.0);
    }

    #[test]
    fn standard_bounds() {
        for (i, f) in STANDARD.iter().enumerate() {
            assert!(approx_eq(f(0.0), 0.0), "curve {i}: f(0) = {}", f(0.0));
            assert!(approx_eq(f(1.0), 1.0), "curve {i}: f(1) = {}", f(1.0));
        }
    }

    #[test]
    fn standard_finite_values() {
        for (i, f) in STANDARD.iter().enumerate() {
            for t in samples() {
                assert!(f(t).is_finite(), "curve {i}: f({t}) is not finite");
            }
        }
    }

    #[test]
    fn in_out_symmetric() {
        for (i, f) in IN_OUT.iter().enumerate() {
            assert!(approx_eq(f(0.5), 0.5), "curve {i}: f(0.5) = {}", f(0.5));
            for t in samples() {
                assert!(
                    approx_eq(f(t) + f(1.0 - t), 1.0),
                    "curve {i}: f({t}) + f(1 - {t}) != 1"
                );
            }
        }
    }

    #[test]
    fn out_is_flipped_in() {
        for (i, (f_in, f_out)) in IN_OUT_PAIRS.iter().enumerate() {
            for t in samples() {
                assert!(
                    approx_eq(f_out(t), 1.0 - f_in(1.0 - t)),
                    "pair {i}: out({t}) != 1 - in(1 - {t})"
                );
            }
        }
    }

    #[test]
    fn smooth_zero_derivatives_at_bounds() {
        let h = 1e-4;
        assert!((smooth(h) - smooth(0.0)) / h < 1e-6);
        assert!((smooth(1.0) - smooth(1.0 - h)) / h < 1e-6);
    }

    #[test]
    fn known_values() {
        assert!(approx_eq(ease_in_quad(0.5), 0.25));
        assert!(approx_eq(ease_in_cubic(0.5), 0.125));
        assert!(approx_eq(ease_out_cubic(0.5), 0.875));
        assert!(approx_eq(ease_in_sine(1.0 / 3.0), 1.0 - (PI / 6.0).cos()));
        assert!(approx_eq(ease_in_expo(0.5), 2.0_f64.powi(-5)));
    }

    #[test]
    fn back_overshoots() {
        assert!(ease_in_back(0.2) < 0.0);
        assert!(ease_out_back(0.8) > 1.0);
    }

    #[test]
    fn bounce_continuous() {
        for k in [1.0, 2.0, 2.5] {
            let x = k / BOUNCE_D1;
            assert!(
                (ease_out_bounce(x - 1e-9) - ease_out_bounce(x)).abs() < 1e-6,
                "jump at {x}"
            );
        }
    }

    #[test]
    fn there_and_back_round_trip() {
        assert_eq!(there_and_back(0.0), 0.0);
        assert!(approx_eq(there_and_back(0.5), 1.0));
        assert!(approx_eq(there_and_back(1.0), 0.0));
        assert!(approx_eq(there_and_back(0.2), there_and_back(0.8)));
    }

    #[test]
    fn there_and_back_with_pause_plateau() {
        let p = 1.0 / 3.0;
        assert_eq!(there_and_back_with_pause(0.0, p), 0.0);
        assert!(approx_eq(there_and_back_with_pause(1.0 / 3.0, p), 1.0));
        assert_eq!(there_and_back_with_pause(0.5, p), 1.0);
        assert!(approx_eq(there_and_back_with_pause(1.0, p), 0.0));
    }

    #[test]
    fn rush_bounds() {
        assert_eq!(rush_into(0.0), 0.0);
        assert!(approx_eq(rush_into(1.0), 1.0));
        assert!(approx_eq(rush_from(0.0), 0.0));
        assert!(approx_eq(rush_from(1.0), 1.0));
        assert!(rush_into(0.5) < 0.5);
        assert!(rush_from(0.5) > 0.5);
    }

    #[test]
    fn double_smooth_bounds() {
        assert_eq!(double_smooth(0.0), 0.0);
        assert!(approx_eq(double_smooth(1.0), 1.0));
    }

    #[test]
    fn wiggle_returns_to_zero() {
        assert_eq!(wiggle(0.0, 2.0), 0.0);
        assert!(approx_eq(wiggle(1.0, 2.0), 0.0));
    }

    #[test]
    fn lingering_reaches_one_at_80_percent() {
        assert_eq!(lingering(0.0), 0.0);
        assert!(approx_eq(lingering(0.4), 0.5));
        assert_eq!(lingering(0.8), 1.0);
        assert_eq!(lingering(1.0), 1.0);
    }

    #[test]
    fn easing_apply_wiggle() {
        assert_eq!(Easing::Wiggle { wiggles: 3.6 }.apply(0.4), wiggle(0.4, 3.6))
    }

    #[test]
    fn easing_apply_with_pause() {
        for t in samples() {
            assert_eq!(
                Easing::ThereAndBackPause { pause_ratio: 0.27 }.apply(t),
                there_and_back_with_pause(t, 0.27)
            );
        }
    }

    #[test]
    fn easing_apply_matches_free_functions() {
        let pairs: [(Easing, Curve); 28] = [
            (Easing::Linear, linear),
            (Easing::Smooth, smooth),
            (Easing::InSine, ease_in_sine),
            (Easing::OutSine, ease_out_sine),
            (Easing::InOutSine, ease_in_out_sine),
            (Easing::InQuad, ease_in_quad),
            (Easing::OutQuad, ease_out_quad),
            (Easing::InOutQuad, ease_in_out_quad),
            (Easing::InCubic, ease_in_cubic),
            (Easing::OutCubic, ease_out_cubic),
            (Easing::InOutCubic, ease_in_out_cubic),
            (Easing::InExpo, ease_in_expo),
            (Easing::OutExpo, ease_out_expo),
            (Easing::InOutExpo, ease_in_out_expo),
            (Easing::InBack, ease_in_back),
            (Easing::OutBack, ease_out_back),
            (Easing::InOutBack, ease_in_out_back),
            (Easing::InElastic, ease_in_elastic),
            (Easing::OutElastic, ease_out_elastic),
            (Easing::InOutElastic, ease_in_out_elastic),
            (Easing::InBounce, ease_in_bounce),
            (Easing::OutBounce, ease_out_bounce),
            (Easing::InOutBounce, ease_in_out_bounce),
            (Easing::ThereAndBack, there_and_back),
            (Easing::RushInto, rush_into),
            (Easing::RushFrom, rush_from),
            (Easing::DoubleSmooth, double_smooth),
            (Easing::Lingering, lingering),
        ];
        for (easing, f) in pairs {
            for t in samples() {
                assert_eq!(easing.apply(t), f(t), "{easing:?} at t = {t}");
            }
        }
    }
}
