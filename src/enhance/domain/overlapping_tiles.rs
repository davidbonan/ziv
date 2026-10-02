pub const TILE_SIDE: usize = 512;
pub const OVERLAP: usize = 32;
const STRIDE: usize = TILE_SIDE - OVERLAP;

/// How many tiles cover `length` pixels, each overlapping the one before.
pub fn tile_count_along(length: usize) -> usize {
    length.saturating_sub(OVERLAP).div_ceil(STRIDE).max(1)
}

pub fn tile_origin(tile: usize) -> usize {
    tile * STRIDE
}

/// The pixel a tile reads at `position`, the photo mirrored beyond its end.
pub fn mirrored(position: usize, length: usize) -> usize {
    let along = position % (2 * length);
    if along < length {
        along
    } else {
        2 * length - 1 - along
    }
}

/// Where a pixel is read among the tiles along one direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TilePlace {
    pub tile: usize,
    pub offset: usize,
    /// How much of the tile before still shows there; 0 outside the overlap.
    pub share_of_tile_before: f32,
}

impl TilePlace {
    pub fn at(position: usize, tile_count: usize) -> Self {
        let tile = (position / STRIDE).min(tile_count - 1);
        let offset = position - tile_origin(tile);
        let is_in_overlap = tile > 0 && offset < OVERLAP;
        let share_of_tile_before = if is_in_overlap {
            1.0 - (offset as f32 + 0.5) / OVERLAP as f32
        } else {
            0.0
        };
        Self {
            tile,
            offset,
            share_of_tile_before,
        }
    }

    /// The value there, `value_in(tile, offset)` being what each tile holds.
    pub fn blended(&self, value_in: impl Fn(usize, usize) -> f32) -> f32 {
        let value = value_in(self.tile, self.offset);
        if self.share_of_tile_before == 0.0 {
            return value;
        }
        let before = value_in(self.tile - 1, self.offset + STRIDE);
        before * self.share_of_tile_before + value * (1.0 - self.share_of_tile_before)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiles_cover_the_length_with_no_more_than_needed() {
        let counts = [1, 512, 513, 992, 993, 7008].map(tile_count_along);

        assert_eq!(counts, [1, 1, 2, 2, 3, 15]);
    }

    #[test]
    fn photo_is_mirrored_beyond_its_end() {
        let read = [0, 2, 3, 4, 5, 6, 7].map(|position| mirrored(position, 3));

        assert_eq!(read, [0, 2, 2, 1, 0, 0, 1]);
    }

    #[test]
    fn overlap_goes_from_the_tile_before_to_the_next_without_a_step() {
        let value_in = |tile, _| tile as f32;

        let across: Vec<f32> = (479..=512)
            .map(|position| TilePlace::at(position, 2).blended(value_in))
            .collect();

        assert_eq!((across[0], across[33]), (0.0, 1.0));
        let largest_step = across
            .windows(2)
            .map(|pair| pair[1] - pair[0])
            .fold(0.0, f32::max);
        assert!(
            largest_step <= 1.0 / OVERLAP as f32 + 1e-6,
            "{largest_step}"
        );
    }

    #[test]
    fn last_tile_holds_everything_up_to_the_end() {
        let place = TilePlace::at(991, 2);

        assert_eq!((place.tile, place.offset), (1, 511));
    }
}
