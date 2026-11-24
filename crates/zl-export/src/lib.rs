const FRAME_TOLERANCE: f64 = 1e-6;

pub fn frame_count(duration: f64, fps: u32) -> u64 {
    if !(duration.is_finite() && duration > 0.0) {
        1
    } else {
        ((duration * f64::from(fps) - FRAME_TOLERANCE).ceil() as u64).saturating_add(1)
    }
}

pub fn frame_time(index: u64, fps: u32, duration: f64) -> f64 {
    (index as f64 / f64::from(fps)).min(duration)
}

#[cfg(test)]
mod tests;
