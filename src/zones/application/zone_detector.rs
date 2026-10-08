use std::fmt;
use std::ops::RangeInclusive;
use std::sync::Arc;

use crate::develop::domain::coverage_image::{CoverageImage, coverage_image_size};
use crate::develop::domain::zone::{Zone, ZoneMask, ZoneTool};
use crate::models::domain::model::Model;
use crate::models::domain::model_runner::{ModelInput, ModelOutput, ModelRunner};
use crate::models::domain::model_source::ModelSource;
use crate::zones::domain::matte::{coverage_of_logits, sureness_of_shares};
use crate::zones::domain::matte_refinement::{CoarseMatte, Colours, Plane, refined_coverage};
use crate::zones::domain::model_input::normalized_planes;
use crate::zones::domain::photo_view::{PhotoRegion, PhotoView, TexelBlock};
use crate::zones::domain::zone_models::{SKY_MODEL, SUBJECT_MODEL};

use crate::models::application::model_store::{ModelError, ModelStore};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionError {
    Model {
        model: &'static Model,
        error: ModelError,
    },
    Photo(String),
    Inference(String),
}

impl fmt::Display for DetectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Model { model, error } => {
                write!(formatter, "the {} model is missing: {error}", model.name)
            }
            Self::Photo(reason) => write!(formatter, "the photo could not be read ({reason})"),
            Self::Inference(reason) => write!(formatter, "the model failed ({reason})"),
        }
    }
}

/// What a detection is busy with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectionStep {
    Downloading {
        model: &'static Model,
        received: u64,
    },
    Detecting,
}

// An overcast sky is seen with less confidence than a blue one, and is sky all the same.
const SKY_UNSURE: RangeInclusive<f32> = 0.2..=0.6;
const SUBJECT_INPUT: &str = "input_image";
const SUBJECT_SIDE: usize = 1024;
const SKY_INPUT: &str = "input.1";
const SKY_SIDE: usize = 320;

/// A region of the photo squeezed into the square a model takes.
pub(super) struct ModelView<'a> {
    pub photo: &'a dyn PhotoView,
    pub region: PhotoRegion,
    pub input: &'static str,
    pub side: usize,
}

impl<'a> ModelView<'a> {
    fn of_whole(photo: &'a dyn PhotoView, input: &'static str, side: usize) -> Self {
        Self {
            photo,
            region: PhotoRegion::WHOLE,
            input,
            side,
        }
    }

    fn model_input(&self) -> Result<ModelInput, DetectionError> {
        let side = self.side as u32;
        let rgba = self
            .photo
            .pixels(&self.region, [side, side])
            .map_err(DetectionError::Photo)?;
        Ok(ModelInput {
            name: self.input,
            shape: [1, 3, self.side, self.side],
            values: normalized_planes(&rgba),
        })
    }
}

/// Detects zones in a photo with models fetched when first needed.
pub struct ZoneDetector<Source, Runner> {
    store: ModelStore<Source>,
    runner: Runner,
}

impl<Source: ModelSource, Runner: ModelRunner> ZoneDetector<Source, Runner> {
    pub fn new(store: ModelStore<Source>, runner: Runner) -> Self {
        Self { store, runner }
    }

    /// What `model` answers when shown `view`.
    pub(super) fn outputs_seeing(
        &self,
        model: &'static Model,
        view: &ModelView<'_>,
        on_step: &mut dyn FnMut(DetectionStep),
    ) -> Result<Vec<ModelOutput>, DetectionError> {
        let mut on_received = |received| on_step(DetectionStep::Downloading { model, received });
        let file = self
            .store
            .model_file(model, &mut on_received)
            .map_err(|error| DetectionError::Model { model, error })?;
        on_step(DetectionStep::Detecting);
        self.runner
            .outputs(&file, view.model_input()?)
            .map_err(DetectionError::Inference)
    }

    fn subject(
        &self,
        photo: &dyn PhotoView,
        on_step: &mut dyn FnMut(DetectionStep),
    ) -> Result<CoverageImage, DetectionError> {
        let view = ModelView::of_whole(photo, SUBJECT_INPUT, SUBJECT_SIDE);
        let outputs = self.outputs_seeing(&SUBJECT_MODEL, &view, on_step)?;
        let logits = square_map(first_output(outputs)?.values)?;
        let side = logits.size[0] as u32;
        let matte = CoverageImage::new([side, side], coverage_of_logits(&logits.values))
            .ok_or_else(|| unexpected_answer("an empty matte"))?;
        Ok(matte.stretched_to(coverage_image_size(photo.photo_size())))
    }

    fn sky(
        &self,
        photo: &dyn PhotoView,
        on_step: &mut dyn FnMut(DetectionStep),
    ) -> Result<CoverageImage, DetectionError> {
        let view = ModelView::of_whole(photo, SKY_INPUT, SKY_SIDE);
        let outputs = self.outputs_seeing(&SKY_MODEL, &view, on_step)?;
        let shares = square_map(first_output(outputs)?.values)?;
        let sureness = Plane {
            values: sureness_of_shares(&shares.values, &SKY_UNSURE),
            ..shares
        };
        let size = coverage_image_size(photo.photo_size());
        let refined = refined_over(photo, &TexelBlock::whole(size), &sureness)?;
        CoverageImage::new(size, refined).ok_or_else(|| unexpected_answer("an empty photo"))
    }

    fn zone_coverage(
        &self,
        tool: ZoneTool,
        photo: &dyn PhotoView,
        on_step: &mut dyn FnMut(DetectionStep),
    ) -> Result<(Zone, Option<CoverageImage>), DetectionError> {
        let found =
            |coverage: CoverageImage| Some(coverage).filter(CoverageImage::covers_something);
        Ok(match tool {
            ZoneTool::Subject => (Zone::Subject, found(self.subject(photo, on_step)?)),
            ZoneTool::Background => {
                let subject = found(self.subject(photo, on_step)?);
                (Zone::Background, subject.map(|subject| subject.inverted()))
            }
            ZoneTool::Sky => (Zone::Sky, found(self.sky(photo, on_step)?)),
        })
    }

    /// The subject and what it leaves, from one detection; no mask when the
    /// photo has no subject, or nothing but its subject.
    pub(super) fn subject_and_background(
        &self,
        photo: &dyn PhotoView,
        on_step: &mut dyn FnMut(DetectionStep),
    ) -> Result<Vec<ZoneMask>, DetectionError> {
        let subject = self.subject(photo, on_step)?;
        let background = subject.inverted();
        if !subject.covers_something() || !background.covers_something() {
            return Ok(Vec::new());
        }
        let masks = [(Zone::Subject, subject), (Zone::Background, background)];
        Ok(masks
            .into_iter()
            .map(|(zone, coverage)| ZoneMask {
                zone,
                coverage: Arc::new(coverage),
            })
            .collect())
    }

    /// The masks `tool` finds in the photo; none when what it looks for is
    /// not in the photo.
    pub fn zone_masks(
        &self,
        tool: ZoneTool,
        photo: &dyn PhotoView,
        on_step: &mut dyn FnMut(DetectionStep),
    ) -> Result<Vec<ZoneMask>, DetectionError> {
        let (zone, coverage) = self.zone_coverage(tool, photo, on_step)?;
        let mask = coverage.map(|coverage| ZoneMask {
            zone,
            coverage: Arc::new(coverage),
        });
        Ok(mask.into_iter().collect())
    }
}

pub(super) fn unexpected_answer(what: &str) -> DetectionError {
    DetectionError::Inference(format!("unexpected answer: {what}"))
}

pub(super) fn first_output(outputs: Vec<ModelOutput>) -> Result<ModelOutput, DetectionError> {
    let no_output = || unexpected_answer("no output");
    outputs.into_iter().next().ok_or_else(no_output)
}

fn square_map(values: Vec<f32>) -> Result<Plane, DetectionError> {
    let side = values.len().isqrt();
    if side == 0 || side * side != values.len() {
        return Err(unexpected_answer("a matte that is not a square"));
    }
    Ok(Plane {
        size: [side; 2],
        values,
    })
}

/// `matte`, which a model gave for `block` of the photo's coverage image, as
/// the coverage of that block: one value a texel, its edges moved onto the
/// outlines of the photo.
pub(super) fn refined_over(
    photo: &dyn PhotoView,
    block: &TexelBlock,
    matte: &Plane,
) -> Result<Vec<u8>, DetectionError> {
    let region = block.region(coverage_image_size(photo.photo_size()));
    let colours_at = |size: [u32; 2]| {
        let rgba = photo.pixels(&region, size).map_err(DetectionError::Photo)?;
        Ok(Colours::of_rgba(size, &rgba))
    };
    let coarse = CoarseMatte {
        colours: colours_at(matte.size.map(|side| side as u32))?,
        matte: matte.clone(),
    };
    Ok(refined_coverage(&coarse, &colours_at(block.size)?))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Read;
    use std::path::Path;

    use super::*;
    use crate::develop::domain::preset::Preset;
    use crate::models::domain::model::RemoteFile;

    const PHOTO_SIZE: [u32; 2] = [4096, 1024];

    struct GreyPhoto;

    impl PhotoView for GreyPhoto {
        fn photo_size(&self) -> [u32; 2] {
            PHOTO_SIZE
        }

        fn pixels(&self, _: &PhotoRegion, [width, height]: [u32; 2]) -> Result<Vec<u8>, String> {
            Ok(vec![128; (width * height * 4) as usize])
        }
    }

    struct NoNetwork;

    impl ModelSource for NoNetwork {
        fn download(&self, _: &RemoteFile) -> Result<Box<dyn Read>, String> {
            Err("no network".to_owned())
        }
    }

    /// Finds the subject in the left half of what it is shown.
    struct LeftHalfIsSubject;

    impl ModelRunner for LeftHalfIsSubject {
        fn outputs(&self, _: &Path, input: ModelInput) -> Result<Vec<ModelOutput>, String> {
            let [.., height, width] = input.shape;
            let row = (0..width).map(|column| if column < width / 2 { 20.0 } else { -20.0 });
            Ok(vec![ModelOutput {
                shape: vec![1, 1, height, width],
                values: row.cycle().take(width * height).collect(),
            }])
        }
    }

    struct Failing;

    impl ModelRunner for Failing {
        fn outputs(&self, _: &Path, _: ModelInput) -> Result<Vec<ModelOutput>, String> {
            Err("out of memory".to_owned())
        }
    }

    fn folder_holding(model: &Model) -> tempfile::TempDir {
        let folder = tempfile::tempdir().unwrap();
        fs::write(folder.path().join(model.file_name()), b"weights").unwrap();
        folder
    }

    /// Finds no subject at all.
    struct NothingIsSubject;

    impl ModelRunner for NothingIsSubject {
        fn outputs(&self, _: &Path, input: ModelInput) -> Result<Vec<ModelOutput>, String> {
            let [.., height, width] = input.shape;
            Ok(vec![ModelOutput {
                shape: vec![1, 1, height, width],
                values: vec![-20.0; width * height],
            }])
        }
    }

    fn detector_running<Runner: ModelRunner>(
        models: &tempfile::TempDir,
        runner: Runner,
    ) -> ZoneDetector<NoNetwork, Runner> {
        ZoneDetector::new(ModelStore::new(models.path().to_owned(), NoNetwork), runner)
    }

    #[test]
    fn subject_mask_is_the_matte_of_the_model_over_the_photo() {
        let models = folder_holding(&SUBJECT_MODEL);
        let detector = detector_running(&models, LeftHalfIsSubject);
        let mut steps = Vec::new();

        let masks = detector
            .zone_masks(ZoneTool::Subject, &GreyPhoto, &mut |step| steps.push(step))
            .unwrap();

        let [ZoneMask { zone, coverage }] = &masks[..] else {
            panic!("{} masks", masks.len());
        };
        assert_eq!(*zone, Zone::Subject);
        assert_eq!(coverage.size(), [2048, 512]);
        assert_eq!(coverage.coverage_at_share([0.25, 0.5]), 1.0);
        assert_eq!(coverage.coverage_at_share([0.75, 0.5]), 0.0);
        assert_eq!(steps, [DetectionStep::Detecting]);
    }

    #[test]
    fn background_mask_covers_what_the_subject_leaves() {
        let models = folder_holding(&SUBJECT_MODEL);
        let detector = detector_running(&models, LeftHalfIsSubject);

        let masks = detector
            .zone_masks(ZoneTool::Background, &GreyPhoto, &mut |_| {})
            .unwrap();

        assert_eq!(masks[0].zone, Zone::Background);
        assert_eq!(masks[0].coverage.coverage_at_share([0.25, 0.5]), 0.0);
        assert_eq!(masks[0].coverage.coverage_at_share([0.75, 0.5]), 1.0);
    }

    #[test]
    fn sky_mask_covers_what_the_model_is_sure_of() {
        let models = folder_holding(&SKY_MODEL);
        let detector = detector_running(&models, LeftHalfIsSubject);

        let masks = detector
            .zone_masks(ZoneTool::Sky, &GreyPhoto, &mut |_| {})
            .unwrap();

        assert_eq!(masks[0].zone, Zone::Sky);
        assert_eq!(masks[0].coverage.size(), [2048, 512]);
        assert!(masks[0].coverage.coverage_at_share([0.1, 0.5]) > 0.95);
        assert!(masks[0].coverage.coverage_at_share([0.9, 0.5]) < 0.05);
    }

    #[test]
    fn photo_without_subject_gives_no_mask() {
        let models = folder_holding(&SUBJECT_MODEL);
        let detector = detector_running(&models, NothingIsSubject);

        for tool in [ZoneTool::Subject, ZoneTool::Background] {
            let masks = detector.zone_masks(tool, &GreyPhoto, &mut |_| {});

            assert_eq!(masks, Ok(Vec::new()));
        }
    }

    #[test]
    fn subject_pop_masks_the_subject_and_what_it_leaves_from_one_detection() {
        let models = folder_holding(&SUBJECT_MODEL);
        let detector = detector_running(&models, LeftHalfIsSubject);
        let mut steps = Vec::new();

        let masks = detector
            .preset_zone_masks(Preset::SubjectPop, &GreyPhoto, &mut |step| steps.push(step))
            .unwrap();

        let zones: Vec<Zone> = masks.iter().map(|mask| mask.zone).collect();
        assert_eq!(zones, [Zone::Subject, Zone::Background]);
        let on_the_left: Vec<f32> = masks
            .iter()
            .map(|mask| mask.coverage.coverage_at_share([0.25, 0.5]))
            .collect();
        assert_eq!(on_the_left, [1.0, 0.0]);
        assert_eq!(steps, [DetectionStep::Detecting]);
    }

    #[test]
    fn subject_pop_on_a_photo_without_subject_masks_nothing() {
        let models = folder_holding(&SUBJECT_MODEL);
        let detector = detector_running(&models, NothingIsSubject);

        let masks = detector.preset_zone_masks(Preset::SubjectPop, &GreyPhoto, &mut |_| {});

        assert_eq!(masks, Ok(Vec::new()));
    }

    #[test]
    fn missing_model_that_cannot_be_downloaded_is_reported() {
        let models = tempfile::tempdir().unwrap();
        let detector = detector_running(&models, LeftHalfIsSubject);

        let error = detector
            .zone_masks(ZoneTool::Subject, &GreyPhoto, &mut |_| {})
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "the subject model is missing: it could not be downloaded (no network)"
        );
    }

    #[test]
    fn failing_model_is_reported() {
        let models = folder_holding(&SUBJECT_MODEL);
        let detector = detector_running(&models, Failing);

        let error = detector
            .zone_masks(ZoneTool::Subject, &GreyPhoto, &mut |_| {})
            .unwrap_err();

        assert_eq!(error, DetectionError::Inference("out of memory".to_owned()));
    }
}
