use super::*;

#[test]
fn every_format_has_a_short_side_of_1080() {
    for format in all_formats() {
        let short_side = format.width().min(format.height());
        assert_eq!(f64::from(short_side), CANVAS_SHORT_SIDE, "{format:?}");
    }
}

#[test]
fn format_sizes() {
    let sizes: Vec<(u32, u32)> = all_formats()
        .iter()
        .map(|format| (format.width(), format.height()))
        .collect();
    assert_eq!(
        sizes,
        vec![(1920, 1080), (1080, 1920), (1440, 1080), (1080, 1080)]
    );
}

#[test]
fn default_canvas_is_landscape_dark_on_white() {
    let canvas = Canvas::default();
    assert_eq!(canvas.format, CanvasFormat::Landscape);
    assert_eq!(
        canvas.background,
        Color {
            r: 255,
            g: 255,
            b: 255,
            a: 255
        }
    );
    assert_eq!(
        canvas.foreground,
        Color {
            r: 30,
            g: 30,
            b: 30,
            a: 255
        }
    );
}
