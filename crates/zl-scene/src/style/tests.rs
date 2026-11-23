use super::*;

#[test]
fn default_style_follows_the_canvas_foreground() {
    let style = ObjectStyle::default();
    assert_eq!(style.color, None);
    assert_eq!(style.width, 4.0);
    assert_eq!(style.line, LineStyle::Solid);
    assert_eq!(style.fill, None);
    assert_eq!(style.point_radius, 8.0);
    assert_eq!(LineStyle::default(), LineStyle::Solid);
}
