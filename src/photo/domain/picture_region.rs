/// The part of a picture a render shows, as a parallelogram: where its
/// top-left corner is, and what its top and left sides span. In shares of the
/// picture's width and height from its top-left.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PictureRegion {
    pub origin: [f32; 2],
    pub across: [f32; 2],
    pub down: [f32; 2],
}

impl PictureRegion {
    pub const WHOLE: Self = Self::upright([0.0; 2], [1.0; 2]);

    pub const fn upright(min: [f32; 2], size: [f32; 2]) -> Self {
        Self {
            origin: min,
            across: [size[0], 0.0],
            down: [0.0, size[1]],
        }
    }

    /// Where `position`, in shares of the region's sides, is in the picture.
    pub fn at(&self, position: [f32; 2]) -> [f32; 2] {
        [0, 1].map(|axis| {
            self.origin[axis] + position[0] * self.across[axis] + position[1] * self.down[axis]
        })
    }

    /// The part of the region starting at `min` and spanning `size`, both in
    /// shares of its sides.
    pub fn part(&self, min: [f32; 2], size: [f32; 2]) -> Self {
        Self {
            origin: self.at(min),
            across: self.across.map(|share| share * size[0]),
            down: self.down.map(|share| share * size[1]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_part_of_an_upright_region_is_placed_in_the_picture() {
        let right_half = PictureRegion::upright([0.5, 0.0], [0.5, 1.0]);

        let lower_quarter = right_half.part([0.0, 0.75], [1.0, 0.25]);

        assert_eq!(
            lower_quarter,
            PictureRegion::upright([0.5, 0.75], [0.5, 0.25])
        );
    }
}
