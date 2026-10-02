use rayon::prelude::*;

use crate::photo::domain::working_image::WorkingImage;

use super::model_encoding::ModelEncoding;

const CHANNELS: usize = 3;
const NO_DIFFERENCE: u8 = 128;
const LEVELS_EACH_WAY: f32 = 127.0;
// The largest difference kept, in the model's encoding: noise is far below.
const DIFFERENCE_REACH: f32 = 0.5;

pub fn difference_level(difference: f32) -> u8 {
    let levels = (difference / DIFFERENCE_REACH * LEVELS_EACH_WAY).round();
    (f32::from(NO_DIFFERENCE) + levels.clamp(-LEVELS_EACH_WAY, LEVELS_EACH_WAY)) as u8
}

fn difference_of(level: u8) -> f32 {
    (f32::from(level) - f32::from(NO_DIFFERENCE)) / LEVELS_EACH_WAY * DIFFERENCE_REACH
}

/// What enhancing changed in a photo: for each channel of each pixel, the
/// difference from the original in the model's encoding, as a level.
#[derive(Debug, Clone, PartialEq)]
pub struct Enhancement {
    size: [u32; 2],
    encoding: ModelEncoding,
    difference_levels: Vec<u8>,
}

impl Enhancement {
    /// `None` when the levels are not those of a photo of `size`.
    pub fn new(
        size: [u32; 2],
        encoding: ModelEncoding,
        difference_levels: Vec<u8>,
    ) -> Option<Self> {
        let expected = size[0] as usize * size[1] as usize * CHANNELS;
        (difference_levels.len() == expected).then_some(Self {
            size,
            encoding,
            difference_levels,
        })
    }

    pub fn size(&self) -> [u32; 2] {
        self.size
    }

    pub fn encoding(&self) -> ModelEncoding {
        self.encoding
    }

    /// Red, green, blue of each pixel, row by row.
    pub fn difference_levels(&self) -> &[u8] {
        &self.difference_levels
    }

    /// The enhanced photo; `None` when `original` is not the photo enhanced.
    pub fn enhanced(&self, original: &WorkingImage) -> Option<WorkingImage> {
        if [original.width(), original.height()] != self.size {
            return None;
        }
        let levels = self.difference_levels.as_chunks::<CHANNELS>().0;
        let pixels = original
            .pixels()
            .par_iter()
            .zip(levels)
            .map(|(pixel, levels)| {
                std::array::from_fn(|channel| {
                    let encoded = self.encoding.encoded(pixel[channel]);
                    self.encoding
                        .decoded(encoded + difference_of(levels[channel]))
                })
            });
        let [width, height] = self.size;
        Some(WorkingImage::new(width, height, pixels.collect()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn original() -> WorkingImage {
        WorkingImage::new(2, 1, vec![[0.25, 4.0, -0.02], [0.0, 1.0, 0.5]])
    }

    #[test]
    fn photo_without_difference_is_the_original() {
        let original = original();
        let unchanged =
            Enhancement::new([2, 1], ModelEncoding::of(&original), vec![NO_DIFFERENCE; 6]);

        let enhanced = unchanged.unwrap().enhanced(&original).unwrap();

        for (enhanced, original) in enhanced
            .pixels()
            .iter()
            .flatten()
            .zip(original.pixels().iter().flatten())
        {
            assert!(
                (enhanced - original).abs() < 1e-5,
                "{original} became {enhanced}"
            );
        }
    }

    #[test]
    fn difference_moves_the_photo_in_the_model_s_encoding() {
        let grey = WorkingImage::new(1, 1, vec![[0.25; 3]]);
        let encoding = ModelEncoding::with_ceiling(1.0);
        let levels = vec![
            difference_level(0.1),
            difference_level(-0.1),
            difference_level(9.0),
        ];

        let enhanced = Enhancement::new([1, 1], encoding, levels)
            .unwrap()
            .enhanced(&grey)
            .unwrap();

        let expected = [0.1, -0.1, DIFFERENCE_REACH]
            .map(|difference| encoding.decoded(encoding.encoded(0.25) + difference));
        for (enhanced, expected) in enhanced.pixel(0, 0).iter().zip(expected) {
            assert!(
                (enhanced - expected).abs() < 2e-3,
                "{enhanced} for {expected}"
            );
        }
    }

    #[test]
    fn enhancement_of_another_photo_size_is_refused() {
        let encoding = ModelEncoding::with_ceiling(1.0);

        assert_eq!(
            Enhancement::new([2, 1], encoding, vec![NO_DIFFERENCE; 5]),
            None
        );
        let of_one_pixel = Enhancement::new([1, 1], encoding, vec![NO_DIFFERENCE; 3]).unwrap();
        assert_eq!(of_one_pixel.enhanced(&original()), None);
    }
}
