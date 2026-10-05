use std::sync::LazyLock;

use super::matrix3::{self, Matrix3, RgbMatrix, transformed};
use super::working_space::PRIMARIES;

/// Lightness, then the green–red and the blue–yellow axes. Display white is
/// `[1, 0, 0]`; a grey has no extent on the two color axes.
pub type Lab = [f32; 3];

const XYZ_TO_LMS: Matrix3 = [
    [0.8189330101, 0.3618667424, -0.1288597137],
    [0.0329845436, 0.9293118715, 0.0361456387],
    [0.0482003018, 0.2643662691, 0.6338517070],
];
const LMS_TO_LAB: Matrix3 = [
    [0.2104542553, 0.7936177850, -0.0040720468],
    [1.9779984951, -2.4285922050, 0.4505937099],
    [0.0259040371, 0.7827717662, -0.8086757660],
];

/// Oklab of the working space: the matrices on each side of its cube root.
pub struct Oklab {
    pub working_to_lms: RgbMatrix,
    pub lms_to_lab: RgbMatrix,
    pub lab_to_lms: RgbMatrix,
    pub lms_to_working: RgbMatrix,
}

static OKLAB: LazyLock<Oklab> = LazyLock::new(|| {
    // Each row scaled to sum to one: the working white has no color, exactly.
    let working_to_lms = matrix3::multiply(&XYZ_TO_LMS, &PRIMARIES.rgb_to_xyz())
        .map(|row| row.map(|cell| cell / row.iter().sum::<f64>()));
    let inverse = |matrix: &Matrix3| matrix3::inverse(matrix).expect("Oklab matrices invert");
    Oklab {
        working_to_lms: matrix3::to_f32(&working_to_lms),
        lms_to_lab: matrix3::to_f32(&LMS_TO_LAB),
        lab_to_lms: matrix3::to_f32(&inverse(&LMS_TO_LAB)),
        lms_to_working: matrix3::to_f32(&inverse(&working_to_lms)),
    }
});

pub fn oklab() -> &'static Oklab {
    &OKLAB
}

impl Oklab {
    /// Scene-referred: a value brighter than display white or outside the
    /// gamut keeps its place, nothing is clipped.
    pub fn of_working(&self, working: [f32; 3]) -> Lab {
        let lms = transformed(&self.working_to_lms, working);
        transformed(&self.lms_to_lab, lms.map(f32::cbrt))
    }

    pub fn to_working(&self, lab: Lab) -> [f32; 3] {
        let lms = transformed(&self.lab_to_lms, lab);
        transformed(&self.lms_to_working, lms.map(|cone| cone * cone * cone))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::domain::working_space::SrgbInput;

    fn assert_close(actual: [f32; 3], expected: [f32; 3], tolerance: f32) {
        let is_close = actual
            .iter()
            .zip(expected)
            .all(|(actual, expected)| (actual - expected).abs() < tolerance);
        assert!(is_close, "{actual:?} is not {expected:?}");
    }

    #[test]
    fn display_white_is_full_lightness_without_color() {
        assert_close(oklab().of_working([1.0; 3]), [1.0, 0.0, 0.0], 1e-4);
    }

    #[test]
    fn a_grey_has_no_color_whatever_its_light() {
        for light in [0.0, 0.02, 0.18, 3.0] {
            let [_, green_red, blue_yellow] = oklab().of_working([light; 3]);

            assert!(green_red.abs() < 1e-4 && blue_yellow.abs() < 1e-4);
        }
    }

    #[test]
    fn srgb_red_sits_where_oklab_is_published_to_put_it() {
        let red = SrgbInput::default().to_working([1.0, 0.0, 0.0]);

        assert_close(oklab().of_working(red), [0.628, 0.225, 0.126], 1e-3);
    }

    #[test]
    fn a_pixel_comes_back_from_oklab_as_it_went() {
        for working in [[0.1, 0.4, 0.9], [2.5, 1.0, 0.2], [-0.05, 0.3, 0.1]] {
            let back = oklab().to_working(oklab().of_working(working));

            assert_close(back, working, 1e-4);
        }
    }
}
