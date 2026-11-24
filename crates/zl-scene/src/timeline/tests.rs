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

fn moving(to: f64, duration: f64, delay: f64) -> Animation {
    Animation {
        to: Value::Real(to),
        ..animation(duration, delay)
    }
}

fn timeline(steps: Vec<Step>) -> Timeline {
    Timeline { steps }
}

#[test]
fn started_animations_follow_the_traced_example() {
    let two_pi = 2.0 * std::f64::consts::PI;
    let timeline = timeline(vec![
        step(vec![moving(two_pi, 6.0, 0.0)], 1.0),
        step(vec![moving(0.0, 2.0, 0.0)], 0.0),
    ]);
    let first = &timeline.steps[0].animations[0];
    let second = &timeline.steps[1].animations[0];
    assert_eq!(timeline.started(3.0), vec![(first, 0.5)]);
    assert_eq!(timeline.started(6.5), vec![(first, 1.0)]);
    assert_eq!(timeline.started(7.5), vec![(first, 1.0), (second, 0.25)]);
}

#[test]
fn nothing_has_started_before_the_first_start() {
    let timeline = timeline(vec![step(vec![moving(5.0, 1.0, 2.0)], 0.0)]);
    let delayed = &timeline.steps[0].animations[0];
    for time in [-1.0, 0.0, 1.9, f64::NAN] {
        assert!(timeline.started(time).is_empty(), "time {time}");
    }
    assert_eq!(timeline.started(2.0), vec![(delayed, 0.0)]);
}

#[test]
fn waiting_step_delays_the_next_steps() {
    let timeline = timeline(vec![
        step(Vec::new(), 3.0),
        step(vec![moving(5.0, 1.0, 0.0)], 0.0),
    ]);
    let after_wait = &timeline.steps[1].animations[0];
    assert!(timeline.started(2.9).is_empty());
    assert_eq!(timeline.started(3.5), vec![(after_wait, 0.5)]);
}

#[test]
fn started_animations_are_ordered_by_start_then_by_timeline_order() {
    let timeline = timeline(vec![
        step(
            vec![
                moving(1.0, 1.0, 2.0),
                moving(2.0, 1.0, 0.0),
                moving(3.0, 1.0, 2.0),
                moving(4.0, 1.0, 0.0),
                moving(5.0, 0.0, 3.0),
            ],
            0.0,
        ),
        step(vec![moving(6.0, 1.0, 0.0)], 0.0),
    ]);
    let first = &timeline.steps[0].animations;
    let second = &timeline.steps[1].animations;
    let order: Vec<&Animation> = timeline
        .started(10.0)
        .into_iter()
        .map(|(animation, _)| animation)
        .collect();
    assert_eq!(
        order,
        vec![
            &first[1], &first[3], &first[0], &first[2], &first[4], &second[0]
        ]
    );
}

#[test]
fn invalid_durations_jump_and_invalid_delays_start_with_the_step() {
    let timeline = timeline(vec![
        step(Vec::new(), 1.0),
        step(
            vec![
                moving(5.0, f64::NAN, 0.0),
                moving(6.0, -2.0, 0.0),
                moving(7.0, 4.0, f64::NAN),
                moving(8.0, 4.0, -3.0),
            ],
            0.0,
        ),
    ]);
    assert!(timeline.started(0.5).is_empty());
    let targets_and_weights: Vec<(Value, f64)> = timeline
        .started(1.0)
        .into_iter()
        .map(|(animation, weight)| (animation.to.clone(), weight))
        .collect();
    assert_eq!(
        targets_and_weights,
        vec![
            (Value::Real(5.0), 1.0),
            (Value::Real(6.0), 1.0),
            (Value::Real(7.0), 0.0),
            (Value::Real(8.0), 0.0),
        ]
    );
}

#[test]
fn finished_animations_stay_with_their_final_weight() {
    let timeline = timeline(vec![
        step(vec![moving(5.0, 1.0, 0.0)], 0.0),
        step(
            vec![Animation {
                easing: Easing::ThereAndBack,
                ..moving(6.0, 1.0, 0.0)
            }],
            0.0,
        ),
    ]);
    let linear = &timeline.steps[0].animations[0];
    let round_trip = &timeline.steps[1].animations[0];
    assert_eq!(
        timeline.started(100.0),
        vec![(linear, 1.0), (round_trip, 0.0)]
    );
}

#[test]
fn easing_shapes_the_weight() {
    let timeline = timeline(vec![step(
        vec![Animation {
            easing: Easing::InQuad,
            ..moving(5.0, 2.0, 0.0)
        }],
        0.0,
    )]);
    let eased = &timeline.steps[0].animations[0];
    assert_eq!(timeline.started(1.0), vec![(eased, 0.25)]);
}
