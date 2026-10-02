use crate::photo::domain::working_image::WorkingImage;

const EXPONENT: f32 = 2.2;

/// How working values are shown to the model: brought under 1 by the
/// photo's ceiling, then spread like a display would. Reversible.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelEncoding {
    ceiling: f32,
}

impl ModelEncoding {
    pub fn with_ceiling(ceiling: f32) -> Self {
        Self {
            ceiling: ceiling.max(1.0),
        }
    }

    pub fn of(image: &WorkingImage) -> Self {
        let largest = image.pixels().iter().flatten().copied().fold(1.0, f32::max);
        Self::with_ceiling(largest)
    }

    pub fn ceiling(&self) -> f32 {
        self.ceiling
    }

    pub fn encoded(&self, working: f32) -> f32 {
        (working.abs() / self.ceiling)
            .powf(1.0 / EXPONENT)
            .copysign(working)
    }

    pub fn decoded(&self, encoded: f32) -> f32 {
        encoded.abs().powf(EXPONENT).copysign(encoded) * self.ceiling
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_come_back_from_the_encoding_above_white_and_below_black() {
        let image = WorkingImage::new(2, 1, vec![[0.18, 4.0, -0.02], [0.0, 1.0, 0.5]]);
        let encoding = ModelEncoding::of(&image);

        assert_eq!(encoding.ceiling(), 4.0);
        assert_eq!(encoding.encoded(4.0), 1.0);
        for working in [0.0, 0.18, 1.0, 4.0, -0.02] {
            let back = encoding.decoded(encoding.encoded(working));
            assert!((back - working).abs() < 1e-5, "{working} came back {back}");
        }
    }
}
