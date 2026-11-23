use super::*;
use glam::DVec3;

#[test]
fn every_layer_is_below_the_layer_count() {
    let plain = ObjectStyle::default();
    let filled = ObjectStyle {
        fill: Some(Color {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        }),
        ..ObjectStyle::default()
    };
    let circle = ObjectValue::Circle {
        center: DVec3::ZERO,
        radius: 1.0,
    };
    let cases = [
        (ObjectValue::Number(crate::Value::Real(1.0)), plain, None),
        (circle.clone(), filled, Some(0)),
        (
            ObjectValue::Segment {
                start: DVec3::ZERO,
                end: DVec3::X,
            },
            plain,
            Some(1),
        ),
        (circle, plain, Some(1)),
        (ObjectValue::Point(DVec3::ZERO), plain, Some(2)),
        (ObjectValue::Curve { from: 0.0, to: 1.0 }, plain, Some(1)),
    ];
    for (value, style, expected) in cases {
        let layer = draw_layer(&value, &style);
        assert_eq!(layer, expected, "{value:?}");
        assert!(layer.is_none_or(|layer| layer < LAYER_COUNT), "{value:?}");
    }
}
