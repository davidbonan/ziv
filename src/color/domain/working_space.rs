use super::matrix3;
use super::primaries::{Primaries, REC709, REC2020};
use super::srgb_transfer;

pub const PRIMARIES: Primaries = REC2020;

/// Scene light an average subject reflects, display white being 1.
pub const MIDDLE_GREY: f32 = 0.18;

pub fn luminance([red, green, blue]: [f32; 3]) -> f32 {
    let [for_red, for_green, for_blue] = luminance_weights();
    for_red * red + for_green * green + for_blue * blue
}

pub fn luminance_weights() -> [f32; 3] {
    matrix3::to_f32(&PRIMARIES.rgb_to_xyz())[1]
}

type RgbMatrix = [[f32; 3]; 3];

fn transform(matrix: &RgbMatrix, [red, green, blue]: [f32; 3]) -> [f32; 3] {
    matrix.map(|row| row[0] * red + row[1] * green + row[2] * blue)
}

/// Encoded sRGB → linear working space.
pub struct SrgbInput {
    rec709_to_working: RgbMatrix,
}

impl Default for SrgbInput {
    fn default() -> Self {
        Self {
            rec709_to_working: matrix3::to_f32(&REC709.rgb_to(&PRIMARIES)),
        }
    }
}

impl SrgbInput {
    pub fn to_working(&self, encoded: [f32; 3]) -> [f32; 3] {
        transform(&self.rec709_to_working, encoded.map(srgb_transfer::decode))
    }
}

/// Linear working space → encoded sRGB, clipped to the display range.
pub struct DisplayTransform {
    working_to_rec709: RgbMatrix,
}

impl Default for DisplayTransform {
    fn default() -> Self {
        Self {
            working_to_rec709: matrix3::to_f32(&PRIMARIES.rgb_to(&REC709)),
        }
    }
}

impl DisplayTransform {
    pub fn working_to_rec709(&self) -> &RgbMatrix {
        &self.working_to_rec709
    }

    pub fn to_display_linear(&self, working: [f32; 3]) -> [f32; 3] {
        transform(&self.working_to_rec709, working)
    }

    pub fn encoded(&self, display_linear: [f32; 3]) -> [f32; 3] {
        display_linear.map(|linear| srgb_transfer::encode(linear.clamp(0.0, 1.0)))
    }

    pub fn to_display(&self, working: [f32; 3]) -> [f32; 3] {
        self.encoded(self.to_display_linear(working))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_8_bit(encoded: [f32; 3]) -> [u8; 3] {
        encoded.map(|channel| (channel * 255.0).round() as u8)
    }

    #[test]
    fn srgb_colors_survive_the_working_space_round_trip() {
        let input = SrgbInput::default();
        let display = DisplayTransform::default();

        for code in [
            [0, 0, 0],
            [255, 255, 255],
            [255, 0, 0],
            [0, 255, 0],
            [0, 0, 255],
            [12, 128, 200],
        ] {
            let encoded = code.map(|channel: u8| f32::from(channel) / 255.0);

            assert_eq!(
                to_8_bit(display.to_display(input.to_working(encoded))),
                code
            );
        }
    }

    #[test]
    fn values_brighter_than_display_white_clip_to_white() {
        let display = DisplayTransform::default();

        assert_eq!(
            to_8_bit(display.to_display([4.0, 4.0, 4.0])),
            [255, 255, 255]
        );
    }

    #[test]
    fn colors_outside_the_display_gamut_clip_per_channel() {
        let display = DisplayTransform::default();

        assert_eq!(to_8_bit(display.to_display([0.0, 1.0, 0.0])), [0, 255, 0]);
    }
}
