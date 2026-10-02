use crate::color::domain::illuminant::Illuminant;
use crate::color::domain::matrix3::{self, Matrix3};
use crate::color::domain::working_space;

/// What a RAW file says about its sensor's colors.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraCalibration {
    /// CIE XYZ (D65) → camera RGB.
    pub xyz_to_camera: Matrix3,
    /// Per-channel gains making the scene's white neutral in camera RGB.
    pub white_balance: [f32; 3],
}

/// Camera RGB in sensor range (1.0 = saturated) → linear working space.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraToWorking {
    white_balance: [f32; 3],
    balanced_camera_to_working: [[f32; 3]; 3],
}

fn with_rows_summing_to_one(matrix: Matrix3) -> Matrix3 {
    matrix.map(|row| {
        let sum: f64 = row.iter().sum();
        row.map(|cell| cell / sum)
    })
}

impl CameraCalibration {
    /// The light under which the white balance gains make white neutral.
    /// `None` when the calibration matrix cannot be inverted.
    pub fn as_shot_illuminant(&self) -> Option<Illuminant> {
        let camera_to_xyz = matrix3::inverse(&self.xyz_to_camera)?;
        let camera_white = self.white_balance.map(|gain| 1.0 / f64::from(gain));
        Some(Illuminant::of_white_xyz(matrix3::transform(
            &camera_to_xyz,
            camera_white,
        )))
    }

    /// `None` when the calibration matrix cannot be inverted.
    pub fn to_working(&self) -> Option<CameraToWorking> {
        let working_to_camera =
            matrix3::multiply(&self.xyz_to_camera, &working_space::PRIMARIES.rgb_to_xyz());
        // White-balanced camera white and working white are both (1, 1, 1).
        let working_to_balanced_camera = with_rows_summing_to_one(working_to_camera);
        let balanced_camera_to_working = matrix3::inverse(&working_to_balanced_camera)?;

        let smallest_gain = self.white_balance.into_iter().fold(f32::INFINITY, f32::min);
        Some(CameraToWorking {
            white_balance: self.white_balance.map(|gain| gain / smallest_gain),
            balanced_camera_to_working: matrix3::to_f32(&balanced_camera_to_working),
        })
    }
}

impl CameraToWorking {
    pub fn convert(&self, camera: [f32; 3]) -> [f32; 3] {
        // A saturated sensor holds no color: clipping at the level the least
        // amplified channel saturates keeps blown highlights neutral.
        let balanced =
            [0, 1, 2].map(|channel| (camera[channel] * self.white_balance[channel]).min(1.0));
        self.balanced_camera_to_working
            .map(|row| row[0] * balanced[0] + row[1] * balanced[1] + row[2] * balanced[2])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SONY_A7M4_XYZ_TO_CAMERA: Matrix3 = [
        [0.7374, -0.2389, -0.0551],
        [-0.5435, 1.3162, 0.2519],
        [-0.1006, 0.1795, 0.6552],
    ];
    const DAYLIGHT_GAINS: [f32; 3] = [2.467_773_4, 1.0, 1.521_484_4];

    fn sony_a7m4() -> CameraToWorking {
        CameraCalibration {
            xyz_to_camera: SONY_A7M4_XYZ_TO_CAMERA,
            white_balance: DAYLIGHT_GAINS,
        }
        .to_working()
        .unwrap()
    }

    #[test]
    fn daylight_gains_mean_a_daylight_illuminant() {
        let as_shot = CameraCalibration {
            xyz_to_camera: SONY_A7M4_XYZ_TO_CAMERA,
            white_balance: DAYLIGHT_GAINS,
        }
        .as_shot_illuminant()
        .unwrap();

        assert!(
            (4500.0..6500.0).contains(&as_shot.temperature),
            "{as_shot:?}"
        );
        assert!(as_shot.tint.abs() < 40.0, "{as_shot:?}");
    }

    fn assert_close(actual: [f32; 3], expected: [f32; 3]) {
        for (actual_channel, expected_channel) in actual.iter().zip(expected) {
            assert!(
                (actual_channel - expected_channel).abs() < 1e-4,
                "{actual:?} differs from {expected:?}"
            );
        }
    }

    #[test]
    fn camera_seeing_in_working_primaries_needs_no_conversion() {
        let conversion = CameraCalibration {
            xyz_to_camera: working_space::PRIMARIES.xyz_to_rgb(),
            white_balance: [1.0; 3],
        }
        .to_working()
        .unwrap();

        assert_close(conversion.convert([0.2, 0.5, 0.7]), [0.2, 0.5, 0.7]);
    }

    #[test]
    fn white_balanced_grey_is_neutral_in_the_working_space() {
        let grey_as_the_sensor_sees_it = DAYLIGHT_GAINS.map(|gain| 0.18 / gain);

        assert_close(sony_a7m4().convert(grey_as_the_sensor_sees_it), [0.18; 3]);
    }

    #[test]
    fn saturated_sensor_is_neutral_white() {
        assert_close(sony_a7m4().convert([1.0; 3]), [1.0; 3]);
    }

    #[test]
    fn white_balance_gains_below_one_still_clip_at_sensor_saturation() {
        let conversion = CameraCalibration {
            xyz_to_camera: SONY_A7M4_XYZ_TO_CAMERA,
            white_balance: [1.0, 0.5, 0.8],
        }
        .to_working()
        .unwrap();

        assert_close(conversion.convert([1.0; 3]), [1.0; 3]);
    }

    #[test]
    fn degenerate_calibration_has_no_conversion() {
        let calibration = CameraCalibration {
            xyz_to_camera: [[0.0; 3]; 3],
            white_balance: [1.0; 3],
        };

        assert!(calibration.to_working().is_none());
    }
}
