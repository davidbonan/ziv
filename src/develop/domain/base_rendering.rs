use crate::color::domain::working_space::MIDDLE_GREY;
use crate::photo::domain::photo_kind::PhotoKind;

const CAMERA_LIKE_CONTRAST: f32 = 1.4;
const CAMERA_LIKE_MIDDLE_GREY: f32 = 0.42;

/// What a photo gets before its edit so that it looks finished when untouched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseRendering {
    None,
    CameraLike,
}

/// `x^contrast · (1 + shoulder) / (x^contrast + shoulder)`: 0 stays 0, 1 stays 1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilmCurve {
    pub contrast: f32,
    pub shoulder: f32,
}

impl FilmCurve {
    fn sending_middle_grey_to(display_grey: f32, contrast: f32) -> Self {
        let grey = MIDDLE_GREY.powf(contrast);
        Self {
            contrast,
            shoulder: grey * (1.0 - display_grey) / (display_grey - grey),
        }
    }

    fn light_rendered_as(&self, rendered: f32) -> f32 {
        (self.shoulder * rendered / (1.0 + self.shoulder - rendered)).powf(1.0 / self.contrast)
    }

    pub fn applied(&self, linear: f32) -> f32 {
        let steepened = linear.max(0.0).powf(self.contrast);
        steepened * (1.0 + self.shoulder) / (steepened + self.shoulder)
    }
}

impl BaseRendering {
    pub fn of(kind: &PhotoKind) -> Self {
        match kind {
            PhotoKind::Raw { .. } => Self::CameraLike,
            PhotoKind::StandardImage => Self::None,
        }
    }

    pub fn tone_curve(&self) -> Option<FilmCurve> {
        match self {
            Self::None => None,
            Self::CameraLike => Some(FilmCurve::sending_middle_grey_to(
                CAMERA_LIKE_MIDDLE_GREY,
                CAMERA_LIKE_CONTRAST,
            )),
        }
    }

    /// The light of the photo that ends up shown as middle grey.
    pub fn middle_grey(&self) -> f32 {
        match self.tone_curve() {
            Some(curve) => curve.light_rendered_as(MIDDLE_GREY),
            None => MIDDLE_GREY,
        }
    }

    pub fn rendered(&self, display_linear: [f32; 3]) -> [f32; 3] {
        match self.tone_curve() {
            Some(curve) => display_linear.map(|channel| curve.applied(channel)),
            None => display_linear,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::domain::illuminant::Illuminant;

    fn camera_like() -> FilmCurve {
        BaseRendering::CameraLike.tone_curve().unwrap()
    }

    #[test]
    fn standard_images_get_no_base_rendering() {
        let pixel = [0.1, 0.4, 1.7];

        assert_eq!(
            BaseRendering::of(&PhotoKind::StandardImage).rendered(pixel),
            pixel
        );
    }

    #[test]
    fn raw_photos_get_the_camera_like_rendering() {
        let raw = PhotoKind::Raw {
            as_shot: Illuminant {
                temperature: 5200.0,
                tint: 0.0,
            },
        };

        assert_eq!(BaseRendering::of(&raw), BaseRendering::CameraLike);
    }

    #[test]
    fn camera_like_curve_keeps_black_and_white_and_lifts_middle_grey() {
        let curve = camera_like();

        assert_eq!(curve.applied(0.0), 0.0);
        assert!((curve.applied(1.0) - 1.0).abs() < 1e-6);
        assert!((curve.applied(MIDDLE_GREY) - CAMERA_LIKE_MIDDLE_GREY).abs() < 1e-5);
    }

    #[test]
    fn camera_like_curve_never_darkens_as_light_grows() {
        let curve = camera_like();
        let steps: Vec<f32> = (0..=200)
            .map(|step| curve.applied(step as f32 / 100.0))
            .collect();

        assert!(steps.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn camera_like_curve_rolls_highlights_off() {
        let curve = camera_like();
        let slope_near = |light: f32| (curve.applied(light + 0.01) - curve.applied(light)) / 0.01;

        assert!(slope_near(0.9) < slope_near(MIDDLE_GREY) / 3.0);
    }

    #[test]
    fn middle_grey_of_a_photo_is_the_light_shown_as_middle_grey() {
        let camera_like = BaseRendering::CameraLike;

        let shown = camera_like.rendered([camera_like.middle_grey(); 3])[0];

        assert!((shown - MIDDLE_GREY).abs() < 1e-5);
        assert_eq!(BaseRendering::None.middle_grey(), MIDDLE_GREY);
    }

    #[test]
    fn light_below_black_renders_black() {
        assert_eq!(camera_like().applied(-0.2), 0.0);
    }
}
