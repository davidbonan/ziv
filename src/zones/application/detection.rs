use crate::develop::domain::zone::{DetectionTool, PersonPart, ZoneMask, ZoneTool};
use crate::models::domain::model_runner::ModelRunner;
use crate::models::domain::model_source::ModelSource;
use crate::zones::domain::photo_view::{PhotoRegion, PhotoView};

use super::person_zones::PartsAsked;
use super::zone_detector::{DetectionError, DetectionStep, ZoneDetector};

/// What a detection is asked to find.
#[derive(Debug, Clone, PartialEq)]
pub enum DetectionAsked {
    Zone(ZoneTool),
    Persons,
    PersonParts {
        persons: Vec<PhotoRegion>,
        parts: Vec<PersonPart>,
    },
}

impl From<DetectionTool> for DetectionAsked {
    fn from(tool: DetectionTool) -> Self {
        match tool {
            DetectionTool::Zone(zone) => Self::Zone(zone),
            DetectionTool::People => Self::Persons,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Detected {
    Masks(Vec<ZoneMask>),
    /// From left to right.
    Persons(Vec<PhotoRegion>),
}

impl DetectionAsked {
    /// The parts asked that have no mask among `masks`.
    pub fn parts_missing_from(&self, masks: &[ZoneMask]) -> Vec<PersonPart> {
        let Self::PersonParts { parts, .. } = self else {
            return Vec::new();
        };
        let is_missing = |part: &&PersonPart| !masks.iter().any(|mask| mask.zone == part.zone());
        parts.iter().filter(is_missing).copied().collect()
    }
}

impl<Source: ModelSource, Runner: ModelRunner> ZoneDetector<Source, Runner> {
    pub fn detected(
        &self,
        asked: &DetectionAsked,
        photo: &dyn PhotoView,
        on_step: &mut dyn FnMut(DetectionStep),
    ) -> Result<Detected, DetectionError> {
        match asked {
            DetectionAsked::Zone(tool) => {
                self.zone_masks(*tool, photo, on_step).map(Detected::Masks)
            }
            DetectionAsked::Persons => self.persons(photo, on_step).map(Detected::Persons),
            DetectionAsked::PersonParts { persons, parts } => {
                let asked = PartsAsked { persons, parts };
                self.person_parts(photo, &asked, on_step)
                    .map(Detected::Masks)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::develop::domain::coverage_image::CoverageImage;
    use crate::develop::domain::zone::Zone;

    #[test]
    fn parts_without_a_mask_are_missing() {
        let asked = DetectionAsked::PersonParts {
            persons: vec![PhotoRegion::WHOLE],
            parts: vec![PersonPart::Hair, PersonPart::Eyes, PersonPart::Lips],
        };
        let hair = ZoneMask {
            zone: Zone::Hair,
            coverage: Arc::new(CoverageImage::new([1, 1], vec![255]).unwrap()),
        };

        let missing = asked.parts_missing_from(&[hair]);

        assert_eq!(missing, [PersonPart::Eyes, PersonPart::Lips]);
    }
}
