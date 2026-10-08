use crate::develop::domain::preset::Preset;
use crate::develop::domain::zone::{PersonPart, ZoneMask, ZoneTool};
use crate::models::domain::model_runner::ModelRunner;
use crate::models::domain::model_source::ModelSource;
use crate::zones::domain::photo_view::PhotoView;

use super::zone_detector::{DetectionError, DetectionStep, ZoneDetector};

impl<Source: ModelSource, Runner: ModelRunner> ZoneDetector<Source, Runner> {
    /// One mask per zone `preset` retouches; none when one of them is not in the photo.
    pub fn preset_zone_masks(
        &self,
        preset: Preset,
        photo: &dyn PhotoView,
        on_step: &mut dyn FnMut(DetectionStep),
    ) -> Result<Vec<ZoneMask>, DetectionError> {
        match preset {
            Preset::WhiterTeeth => self.teeth_of_everyone(photo, on_step),
            Preset::EnhancedSky | Preset::DramaticSky | Preset::GoldenSky => {
                self.zone_masks(ZoneTool::Sky, photo, on_step)
            }
            Preset::BrightEyes => self.part_of_everyone(PersonPart::Eyes, photo, on_step),
            Preset::SkinGlow => self.part_of_everyone(PersonPart::Skin, photo, on_step),
            Preset::LipColor => self.part_of_everyone(PersonPart::Lips, photo, on_step),
            Preset::HairShine => self.part_of_everyone(PersonPart::Hair, photo, on_step),
            Preset::SubjectPop => self.subject_and_background(photo, on_step),
        }
    }
}
