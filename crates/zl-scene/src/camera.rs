use crate::{CANVAS_SHORT_SIDE, Canvas};
use glam::{DVec2, DVec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub center: DVec2,
    pub extent: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Camera {
            center: DVec2::ZERO,
            extent: 8.0,
        }
    }
}

impl Camera {
    pub fn scale(&self) -> Option<f64> {
        let res = CANVAS_SHORT_SIDE / self.extent;
        if res > 0.0 && res.is_finite() {
            Some(res)
        } else {
            None
        }
    }

    pub fn to_canvas(&self, point: DVec3, canvas: &Canvas) -> Option<DVec2> {
        let s = self.scale()?;
        let x = canvas.format.width() as f64 / 2.0 + (point.x - self.center.x) * s;
        let y = canvas.format.height() as f64 / 2.0 - (point.y - self.center.y) * s;

        if x.is_finite() && y.is_finite() {
            Some(DVec2 { x, y })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests;
