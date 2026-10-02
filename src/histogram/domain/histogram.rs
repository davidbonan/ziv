use std::array;

pub const LEVEL_COUNT: usize = 256;
const WHITE: usize = LEVEL_COUNT - 1;
/// Rec.709 luma weights of red, green and blue, in 256ths.
const LUMA_WEIGHTS: [u32; 3] = [54, 183, 19];
/// The long side of the reduced photo a histogram is counted on.
const SAMPLE_LONG_SIDE: u32 = 256;

pub type LevelCounts = [u32; LEVEL_COUNT];
pub type LevelHeights = [f32; LEVEL_COUNT];

/// The size a photo is rendered at to be counted: its shape, reduced.
pub fn sample_size(photo_size: [u32; 2]) -> [u32; 2] {
    let long_side = photo_size[0].max(photo_size[1]);
    if long_side <= SAMPLE_LONG_SIDE {
        return photo_size;
    }
    photo_size.map(|side| (side * SAMPLE_LONG_SIDE).div_ceil(long_side).max(1))
}

/// How many pixels of a developed photo sit at each display level, from black to white.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Histogram {
    luminance: LevelCounts,
    channels: LevelCounts,
}

impl Histogram {
    /// `rgba` holds display-encoded pixels, four bytes each.
    pub fn of_display_pixels(rgba: &[u8]) -> Self {
        let mut luminance = [0; LEVEL_COUNT];
        let mut by_channel = [[0; LEVEL_COUNT]; 3];
        for pixel in rgba.as_chunks::<4>().0 {
            let mut weighted = 128;
            for channel in 0..3 {
                weighted += LUMA_WEIGHTS[channel] * u32::from(pixel[channel]);
                by_channel[channel][usize::from(pixel[channel])] += 1;
            }
            luminance[(weighted >> 8) as usize] += 1;
        }
        let largest_channel = |level| by_channel.map(|counts: LevelCounts| counts[level]);
        Self {
            luminance,
            channels: array::from_fn(|level| largest_channel(level).into_iter().max().unwrap_or(0)),
        }
    }

    pub fn luminance(&self) -> &LevelCounts {
        &self.luminance
    }

    /// For each level, the largest of its red, green and blue counts.
    pub fn channels(&self) -> &LevelCounts {
        &self.channels
    }

    fn tallest_unclipped(&self) -> u32 {
        let unclipped = |counts: &LevelCounts| counts[1..WHITE].iter().copied().max().unwrap_or(0);
        unclipped(&self.luminance).max(unclipped(&self.channels))
    }

    /// From 0 to 1, relative to the tallest level that is not clipped; a
    /// clipped level taller than that is 1.
    fn heights(&self, counts: &LevelCounts) -> LevelHeights {
        let full = self.tallest_unclipped().max(1) as f32;
        counts.map(|count| (count as f32 / full).min(1.0))
    }

    pub fn luminance_heights(&self) -> LevelHeights {
        self.heights(&self.luminance)
    }

    pub fn channel_heights(&self) -> LevelHeights {
        self.heights(&self.channels)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLACK: [u8; 4] = [0, 0, 0, 255];
    const WHITE_PIXEL: [u8; 4] = [255, 255, 255, 255];
    const GREY: [u8; 4] = [100, 100, 100, 255];
    const RED: [u8; 4] = [200, 0, 0, 255];

    fn histogram_of(pixels: &[[u8; 4]]) -> Histogram {
        Histogram::of_display_pixels(pixels.as_flattened())
    }

    #[test]
    fn a_grey_pixel_counts_at_its_level_in_both_shapes() {
        let histogram = histogram_of(&[GREY, GREY, BLACK]);

        assert_eq!(histogram.luminance()[100], 2);
        assert_eq!(histogram.channels()[100], 2);
        assert_eq!(histogram.luminance()[0], 1);
        assert_eq!(histogram.luminance().iter().sum::<u32>(), 3);
    }

    #[test]
    fn a_colored_pixel_counts_at_its_luma_and_at_the_level_of_each_channel() {
        let histogram = histogram_of(&[RED]);

        assert_eq!(histogram.luminance()[42], 1);
        assert_eq!(histogram.channels()[200], 1);
        assert_eq!(histogram.channels()[0], 1);
    }

    #[test]
    fn heights_are_relative_to_the_tallest_level_that_is_not_clipped() {
        let histogram = histogram_of(&[GREY, GREY, [50, 50, 50, 255], BLACK, BLACK, BLACK]);

        let heights = histogram.luminance_heights();

        assert_eq!(heights[100], 1.0);
        assert_eq!(heights[50], 0.5);
        assert_eq!(heights[0], 1.0);
        assert_eq!(heights[200], 0.0);
    }

    #[test]
    fn a_photo_of_clipped_pixels_only_fills_its_edges() {
        let heights = histogram_of(&[BLACK, WHITE_PIXEL, WHITE_PIXEL]).luminance_heights();

        assert_eq!((heights[0], heights[WHITE], heights[128]), (1.0, 1.0, 0.0));
    }

    #[test]
    fn a_large_photo_is_sampled_at_its_shape_and_a_small_one_as_it_is() {
        assert_eq!(sample_size([7008, 4672]), [256, 171]);
        assert_eq!(sample_size([4672, 7008]), [171, 256]);
        assert_eq!(sample_size([64, 32]), [64, 32]);
    }
}
