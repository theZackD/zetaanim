use crate::{ObjectId, Property, Value};
use zl_math::Easing;

const MAX_SECONDS: f64 = 86_400.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Animation {
    pub object: ObjectId,
    pub property: Property,
    pub to: Value,
    pub duration: f64,
    pub delay: f64,
    pub easing: Easing,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub animations: Vec<Animation>,
    pub pause: f64,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Timeline {
    pub steps: Vec<Step>,
}

impl Step {
    pub fn duration(&self) -> f64 {
        let mut longest: f64 = 0.0;

        for animation in &self.animations {
            longest =
                longest.max(valid_seconds(animation.delay) + valid_seconds(animation.duration))
        }

        longest + valid_seconds(self.pause)
    }
}

impl Timeline {
    pub fn duration(&self) -> f64 {
        let mut total = 0.0;

        for step in &self.steps {
            total += step.duration();
        }

        total
    }
}

fn valid_seconds(seconds: f64) -> f64 {
    if seconds > 0.0 && seconds.is_finite() {
        seconds.min(MAX_SECONDS)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests;
