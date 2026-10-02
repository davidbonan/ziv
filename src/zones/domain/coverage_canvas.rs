use crate::develop::domain::coverage_image::CoverageImage;

use super::photo_view::TexelBlock;

/// A coverage image being put together block by block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageCanvas {
    size: [u32; 2],
    values: Vec<u8>,
}

impl CoverageCanvas {
    pub fn covering_nothing(size: [u32; 2]) -> Self {
        Self {
            size,
            values: vec![0; size[0] as usize * size[1] as usize],
        }
    }

    /// Covers `block` with `coverage`, one value a texel of the block, where
    /// that is more than what is covered already.
    pub fn cover(&mut self, block: &TexelBlock, coverage: &[u8]) {
        let width = block.size[0] as usize;
        for (row, covered) in coverage.chunks_exact(width).enumerate() {
            let start =
                (block.origin[1] as usize + row) * self.size[0] as usize + block.origin[0] as usize;
            for (value, covered) in self.values[start..start + width].iter_mut().zip(covered) {
                *value = (*value).max(*covered);
            }
        }
    }

    /// Uncovers what `other` covers, in proportion.
    pub fn uncover(&mut self, other: &Self) {
        for (value, uncovered) in self.values.iter_mut().zip(&other.values) {
            let kept = u16::from(u8::MAX - uncovered);
            *value = (u16::from(*value) * kept / u16::from(u8::MAX)) as u8;
        }
    }

    /// `None` when nothing is more covered than not.
    pub fn into_image(self) -> Option<CoverageImage> {
        CoverageImage::new(self.size, self.values).filter(CoverageImage::covers_something)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLOCK: TexelBlock = TexelBlock {
        origin: [1, 0],
        size: [2, 2],
    };

    #[test]
    fn blocks_cover_their_place_and_the_most_covering_wins() {
        let mut canvas = CoverageCanvas::covering_nothing([4, 2]);

        canvas.cover(&BLOCK, &[200, 100, 50, 0]);
        canvas.cover(&BLOCK, &[100, 255, 0, 0]);

        let image = canvas.into_image().unwrap();
        assert_eq!(image.values(), [0, 200, 255, 0, 0, 50, 0, 0]);
    }

    #[test]
    fn uncovering_removes_what_the_other_covers() {
        let mut canvas = CoverageCanvas::covering_nothing([4, 2]);
        canvas.cover(&BLOCK, &[200, 200, 200, 200]);
        let mut other = CoverageCanvas::covering_nothing([4, 2]);
        other.cover(&BLOCK, &[255, 0, 0, 0]);

        canvas.uncover(&other);

        let image = canvas.into_image().unwrap();
        assert_eq!(image.values(), [0, 0, 200, 0, 0, 200, 200, 0]);
    }

    #[test]
    fn canvas_that_covers_nothing_is_no_image() {
        let mut canvas = CoverageCanvas::covering_nothing([4, 2]);
        canvas.cover(&BLOCK, &[100, 0, 0, 0]);

        assert_eq!(canvas.into_image(), None);
    }
}
