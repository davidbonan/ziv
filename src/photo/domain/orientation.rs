/// How the stored pixels must be turned to be upright; same eight cases as EXIF.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Orientation {
    #[default]
    Upright,
    FlippedHorizontally,
    Rotated180,
    FlippedVertically,
    Transposed,
    Rotated90Clockwise,
    Transversed,
    Rotated270Clockwise,
}

impl Orientation {
    /// `None` for a value outside the EXIF range 1..=8.
    pub fn from_exif(value: u16) -> Option<Self> {
        Some(match value {
            1 => Self::Upright,
            2 => Self::FlippedHorizontally,
            3 => Self::Rotated180,
            4 => Self::FlippedVertically,
            5 => Self::Transposed,
            6 => Self::Rotated90Clockwise,
            7 => Self::Transversed,
            8 => Self::Rotated270Clockwise,
            _ => return None,
        })
    }

    pub fn swaps_width_and_height(self) -> bool {
        matches!(
            self,
            Self::Transposed
                | Self::Rotated90Clockwise
                | Self::Transversed
                | Self::Rotated270Clockwise
        )
    }

    /// The size, once upright, of a picture stored as `[width, height]`.
    pub fn upright_size(self, [width, height]: [u32; 2]) -> [u32; 2] {
        match self.swaps_width_and_height() {
            true => [height, width],
            false => [width, height],
        }
    }

    /// Stored position of the pixel that ends up at `(x, y)` once upright.
    pub fn stored_position(self, [x, y]: [u32; 2], [width, height]: [u32; 2]) -> [u32; 2] {
        let (last_x, last_y) = (width - 1, height - 1);
        match self {
            Self::Upright => [x, y],
            Self::FlippedHorizontally => [last_x - x, y],
            Self::Rotated180 => [last_x - x, last_y - y],
            Self::FlippedVertically => [x, last_y - y],
            Self::Transposed => [y, x],
            Self::Rotated90Clockwise => [y, last_y - x],
            Self::Transversed => [last_x - y, last_y - x],
            Self::Rotated270Clockwise => [last_x - y, x],
        }
    }
}
