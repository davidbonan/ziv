use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use super::adjustments::Adjustments;
use super::color_grading::ColorGrading;
use super::color_mixer::ColorMixer;
use super::framing::Framing;
use super::mask::{CoverageSource, Mask, MaskShape};
use super::tone_curve::ToneCurves;

pub const MOST_MASKS: usize = 16;

/// Everything the user changed on one photo. The default leaves it unchanged.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Edit {
    #[serde(flatten)]
    pub adjustments: Adjustments,
    pub masks: Vec<Mask>,
    /// How much of the photo's enhancement is mixed in, 0 … 100.
    pub enhancement_intensity: f32,
    pub tone_curves: ToneCurves,
    pub color_mixer: ColorMixer,
    pub color_grading: ColorGrading,
    pub framing: Framing,
}

pub const FULL_INTENSITY: f32 = 100.0;
pub const INTENSITY_RANGE: RangeInclusive<f32> = 0.0..=FULL_INTENSITY;

impl Edit {
    /// The share of the enhancement in the photo, 0 … 1.
    pub fn enhancement_share(&self) -> f32 {
        (self.enhancement_intensity / FULL_INTENSITY).clamp(0.0, 1.0)
    }

    pub fn visible_masks(&self) -> impl Iterator<Item = &Mask> {
        self.masks.iter().filter(|mask| !mask.is_hidden)
    }

    /// What each mask rendered from a coverage image is made of, in the
    /// order of the masks, hidden ones included.
    pub fn coverage_sources(&self) -> impl Iterator<Item = CoverageSource<'_>> {
        self.masks
            .iter()
            .filter_map(|mask| mask.shape.coverage_source())
    }

    /// The rank of `shape` among `coverage_sources`.
    pub fn coverage_layer_of(&self, shape: &MaskShape) -> Option<usize> {
        self.masks
            .iter()
            .filter(|mask| mask.shape.coverage_source().is_some())
            .position(|mask| mask.shape == *shape)
    }

    pub fn has_room_for_a_mask(&self) -> bool {
        self.masks.len() < MOST_MASKS
    }

    /// Each mask named after its kind and its rank among the masks of that kind.
    pub fn mask_names(&self) -> Vec<String> {
        let kind_of = |mask: &Mask| mask.shape.kind();
        (0..self.masks.len())
            .map(|index| {
                let kind = kind_of(&self.masks[index]);
                let earlier_of_the_kind = self.masks[..index]
                    .iter()
                    .filter(|mask| kind_of(mask) == kind);
                format!("{} {}", kind.name(), earlier_of_the_kind.count() + 1)
            })
            .collect()
    }
}

impl From<Adjustments> for Edit {
    fn from(adjustments: Adjustments) -> Self {
        Self {
            adjustments,
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use crate::develop::domain::coverage_image::CoverageImage;
    use crate::develop::domain::linear_gradient::LinearGradient;
    use crate::develop::domain::mask::MaskShape;
    use crate::develop::domain::zone::{Zone, ZoneMask};

    fn linear_gradient() -> Mask {
        Mask::of(MaskShape::LinearGradient(LinearGradient {
            full: [0.0, 0.0],
            none: [1.0, 0.0],
        }))
    }

    #[test]
    fn masks_are_named_after_their_kind_and_rank() {
        let edit = Edit {
            masks: vec![linear_gradient(), linear_gradient()],
            ..Edit::default()
        };

        assert_eq!(
            edit.mask_names(),
            ["Linear gradient 1", "Linear gradient 2"]
        );
    }

    #[test]
    fn zone_masks_are_named_after_their_zone() {
        let sky = Mask::of(MaskShape::Zone(ZoneMask {
            zone: Zone::Sky,
            coverage: Arc::new(CoverageImage::new([1, 1], vec![255]).unwrap()),
        }));
        let edit = Edit {
            masks: vec![linear_gradient(), sky.clone(), sky],
            ..Edit::default()
        };

        assert_eq!(edit.mask_names(), ["Linear gradient 1", "Sky 1", "Sky 2"]);
    }

    #[test]
    fn sixteen_masks_leave_no_room_for_another() {
        let mut edit = Edit {
            masks: vec![linear_gradient(); MOST_MASKS - 1],
            ..Edit::default()
        };
        assert!(edit.has_room_for_a_mask());

        edit.masks.push(linear_gradient());
        assert!(!edit.has_room_for_a_mask());
    }
}
