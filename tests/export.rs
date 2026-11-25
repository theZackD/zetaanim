use zl_export::{frame_count, frame_time};
use zl_scene::{
    Animation, Construction, Easing, Expr, Object, ObjectStyle, ObjectValue, Origin, Property,
    Scene, Step, Value,
};

fn wait(seconds: f64) -> Step {
    Step {
        animations: Vec::new(),
        pause: seconds,
    }
}

#[test]
fn last_frame_of_a_video_shows_the_final_state_exactly() {
    let mut scene = Scene::new();
    let a = scene.add(Object {
        name: "a".to_string(),
        origin: Origin::Script,
        construction: Construction::Number {
            value: Expr::Constant(Value::Real(0.0)),
        },
        style: ObjectStyle::default(),
    });
    scene.add_step(wait(2.2));
    scene.add_step(wait(0.2));
    scene.add_step(wait(0.4));
    scene.add_step(Step {
        animations: vec![Animation {
            object: a,
            property: Property::Value,
            to: Value::Real(10.0),
            duration: 0.2,
            delay: 0.0,
            easing: Easing::Linear,
        }],
        pause: 0.0,
    });
    let duration = scene.duration();
    assert_ne!(duration, 3.0);
    let last = frame_count(duration, 60) - 1;
    let evaluated = scene.evaluate(frame_time(last, 60, duration));
    assert_eq!(
        evaluated.values.get(&a),
        Some(&ObjectValue::Number(Value::Real(10.0)))
    );
}
