use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use super::adjustments::Adjustments;
use super::color_grading::ColorGrading;
use super::color_mixer::ColorMixer;
use super::framing::Framing;
use super::mask::{CoverageSource, Mask, MaskShape};
use super::preset::{AppliedPreset, Preset};
use super::tone_curve::ToneCurves;
use super::zone::ZoneMask;

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

    pub fn has_room_for(&self, preset: Preset) -> bool {
        self.masks.len() + preset.mask_count() <= MOST_MASKS
    }

    /// The applied presets of the photo, in the order they were applied.
    pub fn applied_presets(&self) -> Vec<AppliedPreset> {
        let mut applied: Vec<AppliedPreset> = Vec::new();
        for of_mask in self.masks.iter().filter_map(|mask| mask.applied_preset) {
            if applied.iter().all(|listed| listed.id != of_mask.id) {
                applied.push(of_mask);
            }
        }
        applied
    }

    /// Masks the zones of `detected` that `preset` retouches, with the
    /// adjustments it sets on each, as one applied preset. Its id.
    pub fn apply_preset(&mut self, preset: Preset, detected: Vec<ZoneMask>) -> u32 {
        let earlier_ids = self.applied_presets().into_iter().map(|applied| applied.id);
        let id = earlier_ids.max().map_or(0, |last| last + 1);
        let zone_adjustments = preset.zone_adjustments();
        for zone_mask in detected {
            let of_zone = zone_adjustments
                .iter()
                .find(|(zone, _)| *zone == zone_mask.zone);
            if let Some((_, adjustments)) = of_zone {
                self.masks.push(Mask {
                    adjustments: *adjustments,
                    applied_preset: Some(AppliedPreset::of(preset, id)),
                    ..Mask::of(MaskShape::Zone(zone_mask))
                });
            }
        }
        id
    }

    pub fn applied_preset(&self, id: u32) -> Option<AppliedPreset> {
        self.applied_presets()
            .into_iter()
            .find(|applied| applied.id == id)
    }

    /// Removes the applied preset of `id`: every mask it is made of.
    pub fn remove_applied_preset(&mut self, id: u32) {
        let is_of_it = |mask: &Mask| mask.applied_preset.is_some_and(|applied| applied.id == id);
        self.masks.retain(|mask| !is_of_it(mask));
    }

    /// Doses every mask of the applied preset of `id`.
    pub fn set_applied_preset_intensity(&mut self, id: u32, intensity: f32) {
        let of_the_applied_preset = self
            .masks
            .iter_mut()
            .filter_map(|mask| mask.applied_preset.as_mut())
            .filter(|applied| applied.id == id);
        for applied in of_the_applied_preset {
            applied.intensity = intensity;
        }
    }

    /// The applied preset of `id` named after its preset and its rank among
    /// the applied presets of that preset; `None` when the photo has none of that id.
    pub fn applied_preset_name(&self, id: u32) -> Option<String> {
        let applied = self.applied_presets();
        let rank = applied.iter().position(|applied| applied.id == id)?;
        let preset = applied[rank].preset;
        let earlier_of_the_preset = applied[..rank]
            .iter()
            .filter(|earlier| earlier.preset == preset);
        Some(format!(
            "{} {}",
            preset.name(),
            earlier_of_the_preset.count() + 1
        ))
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
    use crate::develop::domain::mask::{MaskKind, MaskShape};
    use crate::develop::domain::zone::Zone;

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
        let sky = Mask::of(MaskShape::Zone(detected(Zone::Sky)));
        let edit = Edit {
            masks: vec![linear_gradient(), sky.clone(), sky],
            ..Edit::default()
        };

        assert_eq!(edit.mask_names(), ["Linear gradient 1", "Sky 1", "Sky 2"]);
    }

    fn detected(zone: Zone) -> ZoneMask {
        ZoneMask {
            zone,
            coverage: Arc::new(CoverageImage::new([1, 1], vec![255]).unwrap()),
        }
    }

    #[test]
    fn applied_preset_masks_its_zone_with_the_adjustments_of_the_preset() {
        let mut edit = Edit::default();

        let id = edit.apply_preset(Preset::EnhancedSky, vec![detected(Zone::Sky)]);

        let [sky] = &edit.masks[..] else {
            panic!("{} masks", edit.masks.len());
        };
        assert_eq!(sky.shape.kind(), MaskKind::Zone(Zone::Sky));
        assert_eq!(sky.adjustments.highlights, -40.0);
        let applied = AppliedPreset::of(Preset::EnhancedSky, id);
        assert_eq!(sky.applied_preset, Some(applied));
    }

    #[test]
    fn subject_pop_masks_subject_and_background_each_with_its_adjustments() {
        let mut edit = Edit::default();

        let id = edit.apply_preset(
            Preset::SubjectPop,
            vec![detected(Zone::Subject), detected(Zone::Background)],
        );

        let exposures: Vec<f32> = edit
            .masks
            .iter()
            .map(|mask| mask.adjustments.exposure)
            .collect();
        assert_eq!(exposures, [0.2, -0.3]);
        assert_eq!(edit.mask_names(), ["Subject 1", "Background 1"]);
        assert!(
            edit.masks
                .iter()
                .all(|mask| mask.applied_preset.map(|applied| applied.id) == Some(id))
        );
    }

    #[test]
    fn preset_applied_twice_is_two_applied_presets_named_by_rank() {
        let mut edit = Edit::default();

        let first = edit.apply_preset(Preset::EnhancedSky, vec![detected(Zone::Sky)]);
        let second = edit.apply_preset(Preset::EnhancedSky, vec![detected(Zone::Sky)]);

        assert_ne!(first, second);
        let names = [first, second].map(|id| edit.applied_preset_name(id).unwrap());
        assert_eq!(names, ["Enhanced sky 1", "Enhanced sky 2"]);
    }

    #[test]
    fn removed_applied_preset_takes_its_masks_and_leaves_the_others() {
        let mut edit = Edit {
            masks: vec![linear_gradient()],
            ..Edit::default()
        };
        let subject_pop = edit.apply_preset(
            Preset::SubjectPop,
            vec![detected(Zone::Subject), detected(Zone::Background)],
        );
        let sky = edit.apply_preset(Preset::EnhancedSky, vec![detected(Zone::Sky)]);

        edit.remove_applied_preset(subject_pop);

        assert_eq!(edit.mask_names(), ["Linear gradient 1", "Sky 1"]);
        assert_eq!(edit.applied_preset(subject_pop), None);
        assert!(edit.applied_preset(sky).is_some());
    }

    #[test]
    fn applied_preset_leaves_with_the_last_of_its_masks() {
        let mut edit = Edit::default();
        let id = edit.apply_preset(
            Preset::SubjectPop,
            vec![detected(Zone::Subject), detected(Zone::Background)],
        );

        edit.masks.remove(0);
        assert_eq!(
            edit.applied_preset_name(id).as_deref(),
            Some("Subject pop 1")
        );

        edit.masks.remove(0);
        assert_eq!(edit.applied_preset(id), None);
    }

    #[test]
    fn intensity_of_an_applied_preset_is_set_on_it_and_not_on_another() {
        let mut edit = Edit::default();
        let first = edit.apply_preset(Preset::EnhancedSky, vec![detected(Zone::Sky)]);
        let second = edit.apply_preset(Preset::EnhancedSky, vec![detected(Zone::Sky)]);

        edit.set_applied_preset_intensity(second, 40.0);

        let intensities = [first, second].map(|id| edit.applied_preset(id).unwrap().intensity);
        assert_eq!(intensities, [FULL_INTENSITY, 40.0]);
        assert_eq!(edit.masks[1].effect_share(), 0.4);
    }

    #[test]
    fn zone_a_preset_does_not_retouch_is_not_masked() {
        let mut edit = Edit::default();

        edit.apply_preset(Preset::EnhancedSky, vec![detected(Zone::Hair)]);

        assert!(edit.masks.is_empty());
    }

    #[test]
    fn preset_needs_room_for_every_mask_it_makes() {
        let mut edit = Edit {
            masks: vec![linear_gradient(); MOST_MASKS - 1],
            ..Edit::default()
        };
        assert!(edit.has_room_for(Preset::EnhancedSky));
        assert!(!edit.has_room_for(Preset::SubjectPop));

        edit.masks.push(linear_gradient());
        assert!(!edit.has_room_for(Preset::EnhancedSky));
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
