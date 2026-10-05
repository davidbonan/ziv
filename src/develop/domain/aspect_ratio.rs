use super::framing::Framing;

/// How far the proportions of a frame may be from a named ratio and still bear its name.
const NAMING_TOLERANCE: f32 = 0.01;

/// A proportion of the frame's long side to its short one that has a name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamedRatio {
    /// The picture's own.
    Original,
    Square,
    FourByFive,
    ThreeByTwo,
    SixteenByNine,
}

impl NamedRatio {
    pub const ALL: [Self; 5] = [
        Self::Original,
        Self::Square,
        Self::FourByFive,
        Self::ThreeByTwo,
        Self::SixteenByNine,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Original => "Original",
            Self::Square => "1 : 1",
            Self::FourByFive => "4 : 5",
            Self::ThreeByTwo => "3 : 2",
            Self::SixteenByNine => "16 : 9",
        }
    }

    /// The long side over the short one, for a frame of `picture`.
    pub fn long_over_short(&self, picture: [u32; 2]) -> f32 {
        match self {
            Self::Original => {
                let [width, height] = picture.map(|side| side as f32);
                width.max(height) / width.min(height)
            }
            Self::Square => 1.0,
            Self::FourByFive => 5.0 / 4.0,
            Self::ThreeByTwo => 3.0 / 2.0,
            Self::SixteenByNine => 16.0 / 9.0,
        }
    }

    /// The name of the proportions the frame of `framing` has, when they have one.
    pub fn of(framing: &Framing, picture: [u32; 2]) -> Option<Self> {
        let of_frame = framing.long_over_short(picture);
        Self::ALL.into_iter().find(|named| {
            (of_frame / named.long_over_short(picture) - 1.0).abs() <= NAMING_TOLERANCE
        })
    }
}

/// What holds the proportions of the frame while it is resized.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RatioLock {
    #[default]
    Free,
    Named(NamedRatio),
    /// The proportions the frame has, which bear no name.
    Custom,
}

impl RatioLock {
    /// The lock holding the frame of `framing` to the proportions it has.
    pub fn on_frame(framing: &Framing, picture: [u32; 2]) -> Self {
        NamedRatio::of(framing, picture).map_or(Self::Custom, Self::Named)
    }

    pub fn is_locked(&self) -> bool {
        *self != Self::Free
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Free => "Free",
            Self::Named(named) => named.name(),
            Self::Custom => "Custom",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::develop::domain::framing::CropFrame;

    const PICTURE: [u32; 2] = [6000, 4000];

    fn framing(size: [f32; 2]) -> Framing {
        Framing {
            frame: CropFrame {
                centre: [0.5; 2],
                size,
            },
            ..Framing::default()
        }
    }

    #[test]
    fn the_whole_picture_has_the_original_ratio_before_any_other_name() {
        assert_eq!(
            RatioLock::on_frame(&Framing::default(), PICTURE),
            RatioLock::Named(NamedRatio::Original)
        );
    }

    #[test]
    fn a_ratio_bears_its_name_whatever_the_orientation_of_the_frame() {
        let portrait_four_by_five = framing([0.4, 0.75]);
        let square = framing([0.5, 0.75]);

        assert_eq!(
            NamedRatio::of(&portrait_four_by_five, PICTURE),
            Some(NamedRatio::FourByFive)
        );
        assert_eq!(NamedRatio::of(&square, PICTURE), Some(NamedRatio::Square));
    }

    #[test]
    fn proportions_without_a_name_lock_as_custom() {
        assert_eq!(
            RatioLock::on_frame(&framing([0.9, 0.3]), PICTURE),
            RatioLock::Custom
        );
    }
}
