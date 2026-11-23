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
mod tests;
