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
mod tests {
    use super::*;
    use crate::CanvasFormat;

    fn canvas(format: CanvasFormat) -> Canvas {
        Canvas {
            format,
            ..Canvas::default()
        }
    }

    fn camera(center: (f64, f64), extent: f64) -> Camera {
        Camera {
            center: DVec2::new(center.0, center.1),
            extent,
        }
    }

    #[test]
    fn default_camera_shows_eight_units_on_the_short_side() {
        let camera = Camera::default();
        assert_eq!(camera.center, DVec2::ZERO);
        assert_eq!(camera.extent, 8.0);
        assert_eq!(camera.scale(), Some(135.0));
    }

    #[test]
    fn smaller_extent_zooms_in() {
        assert_eq!(camera((0.0, 0.0), 4.0).scale(), Some(270.0));
        assert!(camera((0.0, 0.0), 1e300).scale().is_some_and(|s| s > 0.0));
    }

    #[test]
    fn scale_is_undefined_for_invalid_extents() {
        for extent in [
            0.0,
            -0.0,
            -1.0,
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            1e-320,
        ] {
            assert_eq!(
                camera((0.0, 0.0), extent).scale(),
                None,
                "extent = {extent:e}"
            );
        }
    }

    #[test]
    fn origin_goes_to_the_center_of_every_format() {
        let camera = Camera::default();
        for format in crate::canvas::all_formats() {
            let center = DVec2::new(
                f64::from(format.width()) / 2.0,
                f64::from(format.height()) / 2.0,
            );
            assert_eq!(
                camera.to_canvas(DVec3::ZERO, &canvas(format)),
                Some(center),
                "{format:?}"
            );
        }
    }

    #[test]
    fn y_axis_points_up_on_the_plane_and_down_in_the_image() {
        let landscape = Canvas::default();
        let camera = Camera::default();
        assert_eq!(
            camera.to_canvas(DVec3::new(1.0, 1.0, 0.0), &landscape),
            Some(DVec2::new(1095.0, 405.0))
        );
        assert_eq!(
            camera.to_canvas(DVec3::new(-1.0, -1.0, 0.0), &landscape),
            Some(DVec2::new(825.0, 675.0))
        );
    }

    #[test]
    fn camera_center_goes_to_the_canvas_center() {
        let landscape = Canvas::default();
        let camera = camera((2.0, 1.0), 8.0);
        assert_eq!(
            camera.to_canvas(DVec3::new(2.0, 1.0, 0.0), &landscape),
            Some(DVec2::new(960.0, 540.0))
        );
        assert_eq!(
            camera.to_canvas(DVec3::new(3.0, 1.0, 0.0), &landscape),
            Some(DVec2::new(1095.0, 540.0))
        );
    }

    #[test]
    fn z_is_ignored() {
        let landscape = Canvas::default();
        let camera = Camera::default();
        assert_eq!(
            camera.to_canvas(DVec3::new(1.0, 1.0, 5.0), &landscape),
            camera.to_canvas(DVec3::new(1.0, 1.0, 0.0), &landscape)
        );
    }

    #[test]
    fn portrait_canvas_also_shows_four_units_on_each_side() {
        let portrait = canvas(CanvasFormat::Portrait);
        let camera = Camera::default();
        assert_eq!(
            camera.to_canvas(DVec3::new(4.0, 0.0, 0.0), &portrait),
            Some(DVec2::new(1080.0, 960.0))
        );
        assert_eq!(
            camera.to_canvas(DVec3::new(-4.0, 0.0, 0.0), &portrait),
            Some(DVec2::new(0.0, 960.0))
        );
    }

    #[test]
    fn invalid_camera_or_unreachable_point_has_no_position() {
        let landscape = Canvas::default();
        assert_eq!(
            camera((0.0, 0.0), 0.0).to_canvas(DVec3::ZERO, &landscape),
            None
        );
        assert_eq!(
            Camera::default().to_canvas(DVec3::new(1e308, 0.0, 0.0), &landscape),
            None
        );
        assert_eq!(
            Camera::default().to_canvas(DVec3::new(0.0, -1e308, 0.0), &landscape),
            None
        );
    }
}
