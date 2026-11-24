use super::*;

fn animation(duration: f64, delay: f64) -> Animation {
    Animation {
        object: ObjectId(0),
        property: Property::Value,
        to: Value::Real(1.0),
        duration,
        delay,
        easing: Easing::Linear,
    }
}

fn step(animations: Vec<Animation>, pause: f64) -> Step {
    Step { animations, pause }
}

#[test]
fn valid_seconds_keeps_positive_finite_values_up_to_a_day() {
    assert_eq!(valid_seconds(6.0), 6.0);
    assert_eq!(valid_seconds(0.25), 0.25);
    assert_eq!(valid_seconds(MAX_SECONDS), MAX_SECONDS);
    assert_eq!(valid_seconds(MAX_SECONDS + 0.5), MAX_SECONDS);
    assert_eq!(valid_seconds(1e308), MAX_SECONDS);
}

#[test]
fn invalid_seconds_become_zero() {
    for seconds in [0.0, -0.0, -2.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let valid = valid_seconds(seconds);
        assert_eq!(valid, 0.0, "{seconds}");
        assert!(valid.is_sign_positive(), "{seconds}");
    }
}

#[test]
fn step_lasts_as_long_as_its_longest_animation_plus_its_pause() {
    assert_eq!(step(vec![animation(6.0, 0.0)], 1.0).duration(), 7.0);
    let longest_wins = step(
        vec![
            animation(2.0, 0.0),
            animation(5.0, 0.0),
            animation(3.0, 0.0),
        ],
        0.0,
    );
    assert_eq!(longest_wins.duration(), 5.0);
    let delay_counts = step(vec![animation(4.0, 0.0), animation(1.0, 3.5)], 0.0);
    assert_eq!(delay_counts.duration(), 4.5);
}

#[test]
fn waiting_step_lasts_its_pause() {
    assert_eq!(step(Vec::new(), 1.0).duration(), 1.0);
    assert_eq!(step(Vec::new(), 0.0).duration(), 0.0);
}

#[test]
fn invalid_durations_delays_and_pauses_count_as_zero() {
    let invalid = step(
        vec![
            animation(f64::NAN, -1.0),
            animation(f64::INFINITY, f64::NAN),
        ],
        f64::NEG_INFINITY,
    );
    assert_eq!(invalid.duration(), 0.0);
    assert_eq!(
        step(vec![animation(2.0, f64::INFINITY)], f64::NAN).duration(),
        2.0
    );
}

#[test]
fn huge_durations_delays_and_pauses_are_capped_at_a_day() {
    let huge = step(vec![animation(1e308, 1e308)], 1e308);
    assert_eq!(huge.duration(), 3.0 * MAX_SECONDS);
}

#[test]
fn timeline_lasts_the_sum_of_its_steps() {
    assert_eq!(Timeline::default().duration(), 0.0);
    let timeline = Timeline {
        steps: vec![
            step(vec![animation(6.0, 0.0)], 1.0),
            step(Vec::new(), 1.0),
            step(vec![animation(4.0, 0.0), animation(1.0, 3.5)], 0.0),
        ],
    };
    assert_eq!(timeline.duration(), 12.5);
}

#[test]
fn timeline_duration_stays_finite_with_huge_values() {
    let timeline = Timeline {
        steps: vec![step(vec![animation(1e308, 1e308)], 1e308); 1000],
    };
    assert_eq!(timeline.duration(), 1000.0 * 3.0 * MAX_SECONDS);
}
