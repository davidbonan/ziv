/// A part of the photo, in shares of its width and height from its top-left.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhotoRegion {
    pub min: [f32; 2],
    pub size: [f32; 2],
}

impl PhotoRegion {
    pub const WHOLE: Self = Self {
        min: [0.0; 2],
        size: [1.0; 2],
    };

    pub fn centre(&self) -> [f32; 2] {
        [0, 1].map(|axis| self.min[axis] + self.size[axis] / 2.0)
    }

    /// The region of `size` around `centre`, cut where it leaves the photo.
    pub fn around(centre: [f32; 2], size: [f32; 2]) -> Self {
        let min = [0, 1].map(|axis| (centre[axis] - size[axis] / 2.0).clamp(0.0, 1.0));
        let max = [0, 1].map(|axis| (centre[axis] + size[axis] / 2.0).clamp(0.0, 1.0));
        Self {
            min,
            size: [0, 1].map(|axis| max[axis] - min[axis]),
        }
    }

    /// `factor` times as wide and high around the same centre, cut where it
    /// leaves the photo.
    pub fn grown(&self, factor: f32) -> Self {
        Self::around(self.centre(), self.size.map(|side| side * factor))
    }

    /// `inner`, given in shares of this region, in shares of the photo.
    pub fn part(&self, inner: &Self) -> Self {
        Self {
            min: [0, 1].map(|axis| self.min[axis] + inner.min[axis] * self.size[axis]),
            size: [0, 1].map(|axis| inner.size[axis] * self.size[axis]),
        }
    }

    /// The smallest block of texels holding the region in an image of
    /// `image_size` laid over the photo; at least one texel.
    pub fn texel_block(&self, image_size: [u32; 2]) -> TexelBlock {
        let first = |axis: usize| {
            let texel = (self.min[axis] * image_size[axis] as f32).floor() as u32;
            texel.min(image_size[axis] - 1)
        };
        let end = |axis: usize| {
            let far_edge = (self.min[axis] + self.size[axis]) * image_size[axis] as f32;
            (far_edge.ceil() as u32).clamp(first(axis) + 1, image_size[axis])
        };
        TexelBlock {
            origin: [first(0), first(1)],
            size: [end(0) - first(0), end(1) - first(1)],
        }
    }
}

/// A rectangle of texels of an image laid over the photo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TexelBlock {
    pub origin: [u32; 2],
    pub size: [u32; 2],
}

impl TexelBlock {
    pub fn whole(image_size: [u32; 2]) -> Self {
        Self {
            origin: [0; 2],
            size: image_size,
        }
    }

    /// What the block covers of the photo an image of `image_size` is laid over.
    pub fn region(&self, image_size: [u32; 2]) -> PhotoRegion {
        let share =
            |texels: [u32; 2]| [0, 1].map(|axis| texels[axis] as f32 / image_size[axis] as f32);
        PhotoRegion {
            min: share(self.origin),
            size: share(self.size),
        }
    }
}

/// The photo as a model looks at it: developed without its edit.
pub trait PhotoView {
    fn photo_size(&self) -> [u32; 2];

    /// `region` stretched to `size`: display-encoded RGBA, row after row.
    fn pixels(&self, region: &PhotoRegion, size: [u32; 2]) -> Result<Vec<u8>, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grown_region_keeps_its_centre_and_stays_in_the_photo() {
        let region = PhotoRegion {
            min: [0.4, 0.0],
            size: [0.2, 0.5],
        };

        let grown = region.grown(2.0);

        assert_eq!(grown.min, [0.3, 0.0]);
        assert!((grown.size[0] - 0.4).abs() < 1e-6 && (grown.size[1] - 0.75).abs() < 1e-6);
    }

    #[test]
    fn texel_block_holds_the_region_and_covers_it_back() {
        let region = PhotoRegion {
            min: [0.26, 0.5],
            size: [0.2, 0.5],
        };

        let block = region.texel_block([10, 4]);

        let expected = TexelBlock {
            origin: [2, 2],
            size: [3, 2],
        };
        assert_eq!(block, expected);
        assert_eq!(block.region([10, 4]).min, [0.2, 0.5]);
    }

    #[test]
    fn part_of_a_region_is_placed_in_the_photo() {
        let region = PhotoRegion {
            min: [0.5, 0.5],
            size: [0.4, 0.2],
        };
        let lower_half = PhotoRegion {
            min: [0.0, 0.5],
            size: [1.0, 0.5],
        };

        let placed = region.part(&lower_half);

        assert_eq!(placed.min, [0.5, 0.6]);
        assert_eq!(placed.size, [0.4, 0.1]);
    }
}
