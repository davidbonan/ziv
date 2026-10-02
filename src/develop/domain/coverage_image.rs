use std::borrow::Cow;
use std::io::Cursor;
use std::sync::OnceLock;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use image::{GrayImage, ImageFormat};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::mask::{PhotoPoint, photo_extent};

/// Mask edges are soft: past this long edge a larger coverage image shows nothing more.
pub const LONGEST_COVERAGE_EDGE: u32 = 2048;

/// The size of the coverage image of a mask on a photo of `photo_size`.
pub fn coverage_image_size(photo_size: [u32; 2]) -> [u32; 2] {
    let long_edge = photo_size[0].max(photo_size[1]);
    if long_edge <= LONGEST_COVERAGE_EDGE {
        return photo_size;
    }
    photo_size.map(|side| (side * LONGEST_COVERAGE_EDGE).div_ceil(long_edge).max(1))
}

/// The coverage of a mask over the whole photo, 0 … 255, row after row.
/// Written in a document as a PNG in base64.
#[derive(Debug)]
pub struct CoverageImage {
    size: [u32; 2],
    values: Vec<u8>,
    written: OnceLock<String>,
}

impl PartialEq for CoverageImage {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size && self.values == other.values
    }
}

impl Eq for CoverageImage {}

impl CoverageImage {
    /// `None` when `values` is not one value a texel of `size`.
    pub fn new(size: [u32; 2], values: Vec<u8>) -> Option<Self> {
        let is_whole = size[0] > 0 && values.len() == size[0] as usize * size[1] as usize;
        is_whole.then(|| Self {
            size,
            values,
            written: OnceLock::new(),
        })
    }

    pub fn size(&self) -> [u32; 2] {
        self.size
    }

    pub fn values(&self) -> &[u8] {
        &self.values
    }

    fn texel(&self, column: f32, row: f32) -> f32 {
        let [width, height] = self.size;
        let column = column.clamp(0.0, (width - 1) as f32) as usize;
        let row = row.clamp(0.0, (height - 1) as f32) as usize;
        f32::from(self.values[row * width as usize + column]) / 255.0
    }

    /// The coverage at `[across, down]`, each 0 … 1 over the image, between texels.
    pub fn coverage_at_share(&self, [across, down]: [f32; 2]) -> f32 {
        let column = across * self.size[0] as f32 - 0.5;
        let row = down * self.size[1] as f32 - 0.5;
        let [left, top] = [column.floor(), row.floor()];
        let [rightwards, downwards] = [column - left, row - top];
        let upper =
            self.texel(left, top) * (1.0 - rightwards) + self.texel(left + 1.0, top) * rightwards;
        let lower = self.texel(left, top + 1.0) * (1.0 - rightwards)
            + self.texel(left + 1.0, top + 1.0) * rightwards;
        upper * (1.0 - downwards) + lower * downwards
    }

    pub fn coverage(&self, [x, y]: PhotoPoint) -> f32 {
        let [right, bottom] = photo_extent(self.size);
        self.coverage_at_share([x / right, y / bottom])
    }

    /// The values of the image stretched to `size`.
    pub fn values_at_size(&self, size: [u32; 2]) -> Cow<'_, [u8]> {
        if size == self.size {
            return Cow::Borrowed(&self.values);
        }
        let [width, height] = size;
        let stretched = (0..height).flat_map(|row| {
            (0..width).map(move |column| {
                let share = [
                    (column as f32 + 0.5) / width as f32,
                    (row as f32 + 0.5) / height as f32,
                ];
                (self.coverage_at_share(share) * 255.0).round() as u8
            })
        });
        Cow::Owned(stretched.collect())
    }

    pub fn stretched_to(&self, size: [u32; 2]) -> Self {
        Self {
            size,
            values: self.values_at_size(size).into_owned(),
            written: OnceLock::new(),
        }
    }

    /// Covers what this image leaves, and leaves what it covers.
    pub fn inverted(&self) -> Self {
        Self {
            size: self.size,
            values: self.values.iter().map(|value| 255 - value).collect(),
            written: OnceLock::new(),
        }
    }

    /// Whether some place of the photo is more covered than not.
    pub fn covers_something(&self) -> bool {
        self.values.iter().any(|value| *value > u8::MAX / 2)
    }

    fn as_png_in_base64(&self) -> String {
        let [width, height] = self.size;
        let mut png = Cursor::new(Vec::new());
        image::write_buffer_with_format(
            &mut png,
            &self.values,
            width,
            height,
            image::ExtendedColorType::L8,
            ImageFormat::Png,
        )
        .expect("a grey image in memory is written as PNG");
        STANDARD.encode(png.into_inner())
    }

    fn of_png_in_base64(text: &str) -> Result<Self, String> {
        let png = STANDARD.decode(text).map_err(|error| error.to_string())?;
        let image: GrayImage = image::load_from_memory_with_format(&png, ImageFormat::Png)
            .map_err(|error| error.to_string())?
            .into_luma8();
        Self::new([image.width(), image.height()], image.into_raw())
            .ok_or_else(|| "an empty coverage image".to_owned())
    }
}

impl Serialize for CoverageImage {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.written.get_or_init(|| self.as_png_in_base64()))
    }
}

impl<'de> Deserialize<'de> for CoverageImage {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::of_png_in_base64(&text).map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn left_half_covered() -> CoverageImage {
        CoverageImage::new([4, 2], vec![255, 255, 0, 0, 255, 255, 0, 0]).unwrap()
    }

    #[test]
    fn coverage_is_the_texel_at_its_centre_and_blends_between_texels() {
        let image = left_half_covered();

        assert_eq!(image.coverage([0.125, 0.125]), 1.0);
        assert_eq!(image.coverage([0.875, 0.375]), 0.0);
        assert!((image.coverage([0.5, 0.25]) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn coverage_past_the_photo_is_that_of_its_edge() {
        let image = left_half_covered();

        assert_eq!(image.coverage([-1.0, -1.0]), 1.0);
        assert_eq!(image.coverage([2.0, 2.0]), 0.0);
    }

    #[test]
    fn image_stretched_to_another_size_covers_the_same_share_of_it() {
        let image = left_half_covered();
        let stretched = image.values_at_size([8, 1]);

        assert_eq!(stretched[..3], [255; 3]);
        assert_eq!(stretched[5..], [0; 3]);
    }

    #[test]
    fn inverted_image_covers_what_was_left() {
        let inverted = left_half_covered().inverted();

        assert_eq!(inverted.values(), [0, 0, 255, 255, 0, 0, 255, 255]);
    }

    #[test]
    fn faint_coverage_covers_nothing() {
        let faint = CoverageImage::new([2, 1], vec![0, 100]).unwrap();

        assert!(!faint.covers_something());
        assert!(left_half_covered().covers_something());
    }

    #[test]
    fn values_that_do_not_fill_the_size_are_not_an_image() {
        assert!(CoverageImage::new([4, 2], vec![0; 7]).is_none());
        assert!(CoverageImage::new([0, 0], Vec::new()).is_none());
    }

    #[test]
    fn coverage_image_is_the_photo_up_to_a_long_edge() {
        assert_eq!(coverage_image_size([640, 480]), [640, 480]);
        assert_eq!(coverage_image_size([7008, 4672]), [2048, 1366]);
        assert_eq!(coverage_image_size([3000, 6000]), [1024, 2048]);
    }
}
