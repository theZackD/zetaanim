use crate::{ObjectId, Property, Value};
use serde::{Deserialize, Serialize};
use zl_math::{Easing, progression};

const MAX_SECONDS: f64 = 86_400.0;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Animation {
    pub object: ObjectId,
    pub property: Property,
    pub to: Value,
    pub duration: f64,
    pub delay: f64,
    pub easing: Easing,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub animations: Vec<Animation>,
    pub pause: f64,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
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

    pub fn started(&self, time: f64) -> Vec<(&Animation, f64)> {
        let mut step_start = 0.0;
        let mut start_anim: Vec<(f64, &Animation)> = Vec::new();

        for step in &self.steps {
            for animation in &step.animations {
                let start = step_start + valid_seconds(animation.delay);
                if start <= time {
                    start_anim.push((start, animation));
                }
            }
            step_start += step.duration();
        }

        start_anim.sort_by(|a, b| a.0.total_cmp(&b.0));

        let mut res: Vec<(&Animation, f64)> = Vec::new();
        for (start, animation) in start_anim {
            let w =
                animation
                    .easing
                    .apply(progression(time, start, valid_seconds(animation.duration)));
            res.push((animation, w));
        }
        res
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
