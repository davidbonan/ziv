use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::color::domain::illuminant::Illuminant;
use crate::color::domain::matrix3;
use crate::color::domain::working_space::PRIMARIES;
use crate::photo::domain::photo_kind::PhotoKind;

pub const KELVIN_RANGE: RangeInclusive<f32> = 2000.0..=50000.0;
pub const KELVIN_TINT_RANGE: RangeInclusive<f32> = -150.0..=150.0;
pub const RELATIVE_RANGE: RangeInclusive<f32> = -100.0..=100.0;

const MIRED: f32 = 1.0e6;
const MIRED_PER_RELATIVE_UNIT: f32 = 1.0;
/// What a standard image is taken to have been balanced for.
const RELATIVE_REFERENCE: Illuminant = Illuminant {
    temperature: 6500.0,
    tint: 0.0,
};

/// White balance as the sliders show it. On a RAW: the kelvin and tint of the
/// light. On a standard image: an offset from how the file already looks.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WhiteBalance {
    pub temperature: f32,
    pub tint: f32,
}

fn within(range: &RangeInclusive<f32>, value: f32) -> f32 {
    value.round().clamp(*range.start(), *range.end())
}

impl WhiteBalance {
    /// The white balance the photo has before any edit.
    pub fn as_shot(kind: &PhotoKind) -> Self {
        match kind {
            PhotoKind::Raw { as_shot } => Self {
                temperature: within(&KELVIN_RANGE, as_shot.temperature),
                tint: within(&KELVIN_TINT_RANGE, as_shot.tint),
            },
            PhotoKind::StandardImage => Self {
                temperature: 0.0,
                tint: 0.0,
            },
        }
    }

    fn illuminant(&self, kind: &PhotoKind) -> Illuminant {
        match kind {
            PhotoKind::Raw { .. } => Illuminant {
                temperature: self.temperature,
                tint: self.tint,
            },
            PhotoKind::StandardImage => {
                let reference = MIRED / RELATIVE_REFERENCE.temperature;
                Illuminant {
                    temperature: MIRED / (reference - self.temperature * MIRED_PER_RELATIVE_UNIT),
                    tint: RELATIVE_REFERENCE.tint + self.tint,
                }
            }
        }
    }

    /// The working-space transform that shows the photo as lit by this white
    /// balance instead of its as-shot one.
    pub fn adaptation(&self, kind: &PhotoKind) -> [[f32; 3]; 3] {
        let as_shot = Self::as_shot(kind).illuminant(kind);
        let in_xyz = self.illuminant(kind).adaptation_to(&as_shot);
        let to_xyz = PRIMARIES.rgb_to_xyz();
        let from_xyz = PRIMARIES.xyz_to_rgb();
        matrix3::to_f32(&matrix3::multiply(
            &from_xyz,
            &matrix3::multiply(&in_xyz, &to_xyz),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RAW: PhotoKind = PhotoKind::Raw {
        as_shot: Illuminant {
            temperature: 5234.6,
            tint: 11.7,
        },
    };
    const GREY: [f32; 3] = [0.4; 3];

    fn balanced(kind: &PhotoKind, white_balance: WhiteBalance) -> [f32; 3] {
        white_balance
            .adaptation(kind)
            .map(|row| row[0] * GREY[0] + row[1] * GREY[1] + row[2] * GREY[2])
    }

    fn assert_neutral([red, green, blue]: [f32; 3]) {
        assert!((red - green).abs() < 1e-5 && (blue - green).abs() < 1e-5);
    }

    #[test]
    fn as_shot_of_a_raw_is_its_light_in_whole_slider_units() {
        let as_shot = WhiteBalance::as_shot(&RAW);

        assert_eq!((as_shot.temperature, as_shot.tint), (5235.0, 12.0));
    }

    #[test]
    fn grey_stays_neutral_at_the_as_shot_white_balance() {
        for kind in [RAW, PhotoKind::StandardImage] {
            assert_neutral(balanced(&kind, WhiteBalance::as_shot(&kind)));
        }
    }

    #[test]
    fn raising_the_temperature_warms_and_lowering_it_cools() {
        for (kind, step) in [(RAW, 2000.0), (PhotoKind::StandardImage, 40.0)] {
            let as_shot = WhiteBalance::as_shot(&kind);
            let with_temperature = |temperature| WhiteBalance {
                temperature,
                ..as_shot
            };

            let [red, _, blue] = balanced(&kind, with_temperature(as_shot.temperature + step));
            assert!(red > blue, "{kind:?} raised");
            let [red, _, blue] = balanced(&kind, with_temperature(as_shot.temperature - step));
            assert!(red < blue, "{kind:?} lowered");
        }
    }

    #[test]
    fn raising_the_tint_goes_to_magenta_and_lowering_it_to_green() {
        for kind in [RAW, PhotoKind::StandardImage] {
            let as_shot = WhiteBalance::as_shot(&kind);
            let with_tint = |tint| WhiteBalance { tint, ..as_shot };

            let [red, green, blue] = balanced(&kind, with_tint(as_shot.tint + 50.0));
            assert!(green < red && green < blue, "{kind:?} raised");
            let [red, green, blue] = balanced(&kind, with_tint(as_shot.tint - 50.0));
            assert!(green > red && green > blue, "{kind:?} lowered");
        }
    }
}
