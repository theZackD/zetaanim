use super::*;

#[test]
fn frame_count_covers_the_start_and_the_end() {
    assert_eq!(frame_count(6.0, 60), 361);
    assert_eq!(frame_count(1.0, 24), 25);
    assert_eq!(frame_count(0.5, 30), 16);
}

#[test]
fn rounding_errors_do_not_add_a_frame() {
    assert_eq!(frame_count(0.1 + 0.2, 60), 19);
    assert_eq!(frame_count(1.1 + 2.2, 60), 199);
}

#[test]
fn partial_last_interval_still_gets_its_frame() {
    assert_eq!(frame_count(1.0 / 60.0 + 0.001, 60), 3);
}

#[test]
fn invalid_or_empty_durations_give_a_single_frame() {
    for duration in [0.0, -0.0, -2.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(frame_count(duration, 60), 1, "duration {duration}");
    }
    assert_eq!(frame_count(6.0, 0), 1);
}

#[test]
fn huge_duration_does_not_overflow() {
    assert_eq!(frame_count(1e300, 60), u64::MAX);
}

#[test]
fn frame_time_divides_and_stops_at_the_end() {
    assert_eq!(frame_time(0, 60, 6.0), 0.0);
    assert_eq!(frame_time(30, 60, 6.0), 0.5);
    assert_eq!(frame_time(360, 60, 6.0), 6.0);
    assert_eq!(frame_time(18, 60, 0.1 + 0.2), 0.3);
    let duration = 0.26;
    let last = frame_count(duration, 60) - 1;
    assert_eq!(last, 16);
    assert_eq!(frame_time(last, 60, duration), duration);
}

#[test]
fn frame_times_do_not_drift_over_long_videos() {
    let mut accumulated = 0.0;
    for _ in 0..36_000 {
        accumulated += 1.0 / 60.0;
    }
    assert_ne!(accumulated, 600.0);
    assert_eq!(frame_time(36_000, 60, 1000.0), 600.0);
}
