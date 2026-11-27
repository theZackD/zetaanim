use std::f64::consts::PI;

const BACK_C1: f64 = 1.70158;
const BACK_C2: f64 = BACK_C1 * 1.525;
const BACK_C3: f64 = BACK_C1 + 1.0;
const ELASTIC_C4: f64 = 2.0 * PI / 3.0;
const ELASTIC_C5: f64 = 2.0 * PI / 4.5;
const BOUNCE_N1: f64 = 7.5625;
const BOUNCE_D1: f64 = 2.75;

pub fn linear(t: f64) -> f64 {
    t
}

pub fn smooth(t: f64) -> f64 {
    t * t * t * (t * (6.0 * t - 15.0) + 10.0)
}

pub fn ease_in_sine(t: f64) -> f64 {
    if t >= 1.0 {
        1.0
    } else if t <= 0.0 {
        0.0
    } else {
        1.0 - (PI * t / 2.0).cos()
    }
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
    if t >= 1.0 {
        1.0
    } else if t <= 0.0 {
        0.0
    } else {
        BACK_C3 * t * t * t - BACK_C1 * t * t
    }
}

pub fn ease_out_back(t: f64) -> f64 {
    if t <= 0.0 {
        0.0
    } else {
        let u = t - 1.0;
        1.0 + BACK_C3 * u * u * u + BACK_C1 * u * u
    }
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
    let pause_ratio = if (0.0..1.0).contains(&pause_ratio) {
        pause_ratio
    } else {
        0.0
    };
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
    if !(wiggles * PI).is_finite() {
        return 0.0;
    }
    there_and_back(t) * (wiggles * PI * t).sin()
}

pub fn lingering(t: f64) -> f64 {
    (t / 0.8).min(1.0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
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
mod tests;
