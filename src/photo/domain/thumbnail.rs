use std::ops::Range;

use crate::color::domain::working_space::DisplayTransform;

use super::working_image::WorkingImage;

pub const THUMBNAIL_MAX_SIDE: u32 = 320;

/// A small display-encoded sRGB picture standing for a photo, RGBA8 row by row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Thumbnail {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Size fitting in the thumbnail bounds, aspect ratio kept, never enlarged.
pub fn thumbnail_size(width: u32, height: u32) -> [u32; 2] {
    let longest = width.max(height);
    if longest <= THUMBNAIL_MAX_SIDE {
        return [width, height];
    }
    [width, height].map(|side| (side * THUMBNAIL_MAX_SIDE / longest).max(1))
}

fn block(index: u32, count: u32, full: u32) -> Range<u32> {
    index * full / count..(index + 1) * full / count
}

fn average(image: &WorkingImage, columns: Range<u32>, rows: Range<u32>) -> [f32; 3] {
    let count = (columns.len() * rows.len()) as f32;
    rows.flat_map(|y| columns.clone().map(move |x| image.pixel(x, y)))
        .fold([0.0; 3], |sum, pixel| {
            [0, 1, 2].map(|channel| sum[channel] + pixel[channel])
        })
        .map(|channel| channel / count)
}

impl Thumbnail {
    /// Averages the working image down in linear light, then encodes it for display.
    pub fn of(image: &WorkingImage) -> Self {
        let [width, height] = thumbnail_size(image.width(), image.height());
        let display = DisplayTransform::default();
        let rgba = (0..height)
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .flat_map(|(x, y)| {
                let columns = block(x, width, image.width());
                let rows = block(y, height, image.height());
                let [red, green, blue] = display
                    .to_display(average(image, columns, rows))
                    .map(|channel| (channel * 255.0).round() as u8);
                [red, green, blue, u8::MAX]
            })
            .collect();
        Self {
            width,
            height,
            rgba,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn large_photo_is_reduced_to_the_thumbnail_bounds_keeping_its_shape() {
        assert_eq!(thumbnail_size(6400, 3200), [320, 160]);
        assert_eq!(thumbnail_size(3200, 6400), [160, 320]);
    }

    #[test]
    fn small_photo_is_not_enlarged() {
        assert_eq!(thumbnail_size(48, 32), [48, 32]);
    }

    #[test]
    fn thumbnail_averages_in_linear_light() {
        let half_black_half_white = (0..640 * 2)
            .map(|index| if index % 2 == 0 { [0.0; 3] } else { [1.0; 3] })
            .collect();
        let image = WorkingImage::new(640, 2, half_black_half_white);

        let thumbnail = Thumbnail::of(&image);

        let half_white =
            (DisplayTransform::default().to_display([0.5; 3])[0] * 255.0).round() as u8;
        assert_eq!([thumbnail.width, thumbnail.height], [320, 1]);
        assert_eq!(
            thumbnail.rgba[..4],
            [half_white, half_white, half_white, 255]
        );
    }
}
