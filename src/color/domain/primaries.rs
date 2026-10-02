use super::matrix3::{self, Matrix3};

/// CIE 1931 xy chromaticities of an RGB color space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Primaries {
    pub red: [f64; 2],
    pub green: [f64; 2],
    pub blue: [f64; 2],
    pub white: [f64; 2],
}

const D65: [f64; 2] = [0.3127, 0.3290];

pub const REC709: Primaries = Primaries {
    red: [0.640, 0.330],
    green: [0.300, 0.600],
    blue: [0.150, 0.060],
    white: D65,
};

pub const REC2020: Primaries = Primaries {
    red: [0.708, 0.292],
    green: [0.170, 0.797],
    blue: [0.131, 0.046],
    white: D65,
};

fn xyz_of_unit_luminance([x, y]: [f64; 2]) -> [f64; 3] {
    [x / y, 1.0, (1.0 - x - y) / y]
}

impl Primaries {
    pub fn rgb_to_xyz(&self) -> Matrix3 {
        let [red, green, blue] = [self.red, self.green, self.blue].map(xyz_of_unit_luminance);
        let unscaled = [
            [red[0], green[0], blue[0]],
            [red[1], green[1], blue[1]],
            [red[2], green[2], blue[2]],
        ];
        let luminances = matrix3::transform(
            &matrix3::inverse(&unscaled).expect("primaries are not collinear"),
            xyz_of_unit_luminance(self.white),
        );
        unscaled.map(|row| {
            [
                row[0] * luminances[0],
                row[1] * luminances[1],
                row[2] * luminances[2],
            ]
        })
    }

    pub fn xyz_to_rgb(&self) -> Matrix3 {
        matrix3::inverse(&self.rgb_to_xyz()).expect("primaries are not collinear")
    }

    /// Valid between primaries sharing a white point: no chromatic adaptation.
    pub fn rgb_to(&self, target: &Primaries) -> Matrix3 {
        matrix3::multiply(&target.xyz_to_rgb(), &self.rgb_to_xyz())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: &Matrix3, expected: &Matrix3, tolerance: f64) {
        for (actual_row, expected_row) in actual.iter().zip(expected) {
            for (actual_cell, expected_cell) in actual_row.iter().zip(expected_row) {
                assert!(
                    (actual_cell - expected_cell).abs() < tolerance,
                    "{actual:?} differs from {expected:?}"
                );
            }
        }
    }

    #[test]
    fn rec709_to_xyz_matches_the_published_srgb_matrix() {
        let published = [
            [0.4124, 0.3576, 0.1805],
            [0.2126, 0.7152, 0.0722],
            [0.0193, 0.1192, 0.9505],
        ];

        assert_close(&REC709.rgb_to_xyz(), &published, 5e-4);
    }

    #[test]
    fn rec709_to_rec2020_matches_itu_r_bt2087() {
        let bt2087 = [
            [0.6274, 0.3293, 0.0433],
            [0.0691, 0.9195, 0.0114],
            [0.0164, 0.0880, 0.8956],
        ];

        assert_close(&REC709.rgb_to(&REC2020), &bt2087, 5e-5);
    }

    #[test]
    fn white_stays_white_across_primaries() {
        let white = matrix3::transform(&REC2020.rgb_to(&REC709), [1.0, 1.0, 1.0]);

        assert!(white.iter().all(|channel| (channel - 1.0).abs() < 1e-12));
    }

    #[test]
    fn saturated_rec2020_green_is_outside_rec709() {
        let [red, _, blue] = matrix3::transform(&REC2020.rgb_to(&REC709), [0.0, 1.0, 0.0]);

        assert!(red < 0.0 && blue < 0.0);
    }
}
