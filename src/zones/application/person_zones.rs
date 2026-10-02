use std::ops::RangeInclusive;
use std::sync::Arc;

use crate::develop::domain::coverage_image::coverage_image_size;
use crate::develop::domain::zone::{PersonPart, ZoneMask};
use crate::models::domain::model::Model;
use crate::models::domain::model_runner::ModelRunner;
use crate::models::domain::model_source::ModelSource;
use crate::zones::domain::class_shares::ClassShares;
use crate::zones::domain::coverage_canvas::CoverageCanvas;
use crate::zones::domain::matte::sureness_of_shares;
use crate::zones::domain::matte_refinement::Plane;
use crate::zones::domain::person_parts::{
    BODY_FACE_CLASS, FACE_CLASSES_NOT_SKIN, body_classes, face_classes, face_surroundings,
    needs_face,
};
use crate::zones::domain::persons::persons_found;
use crate::zones::domain::photo_view::{PhotoRegion, PhotoView, TexelBlock};
use crate::zones::domain::zone_models::{BODY_PARTS_MODEL, FACE_PARTS_MODEL, PERSONS_MODEL};

use super::zone_detector::{
    DetectionError, DetectionStep, ModelView, ZoneDetector, first_output, refined_over,
    unexpected_answer,
};

const PIXELS_INPUT: &str = "pixel_values";
const MODEL_SIDE: usize = 512;
// Blond hair is a little likely to be skin: a part is where the model leans towards it.
const PART_UNSURE: RangeInclusive<f32> = 0.3..=0.7;
// A person is looked at with a little of what surrounds them.
const PERSON_MARGIN: f32 = 1.1;

/// The parts to mask, on the persons to mask them on.
pub struct PartsAsked<'a> {
    pub persons: &'a [PhotoRegion],
    pub parts: &'a [PersonPart],
}

/// What each asked part covers so far, and what of the faces is not skin.
struct PartCoverages {
    size: [u32; 2],
    of_parts: Vec<(PersonPart, CoverageCanvas)>,
    not_skin: CoverageCanvas,
}

impl PartCoverages {
    fn of(parts: &[PersonPart], size: [u32; 2]) -> Self {
        let nothing = || CoverageCanvas::covering_nothing(size);
        Self {
            size,
            of_parts: parts.iter().map(|part| (*part, nothing())).collect(),
            not_skin: nothing(),
        }
    }

    fn needs_faces(&self) -> bool {
        self.of_parts.iter().any(|(part, _)| needs_face(*part))
    }

    fn into_masks(self) -> Vec<ZoneMask> {
        let not_skin = self.not_skin;
        let masks = self.of_parts.into_iter().filter_map(|(part, mut canvas)| {
            if part == PersonPart::Skin {
                canvas.uncover(&not_skin);
            }
            Some(ZoneMask {
                zone: part.zone(),
                coverage: Arc::new(canvas.into_image()?),
            })
        });
        masks.collect()
    }
}

/// What a model that tells classes apart found in a block of the photo.
struct ClassesSeen<'a> {
    photo: &'a dyn PhotoView,
    block: TexelBlock,
    shares: ClassShares,
}

impl ClassesSeen<'_> {
    // Covers `canvas` with `classes`; nothing to do when the model has none of them.
    fn cover(&self, canvas: &mut CoverageCanvas, classes: &[usize]) -> Result<(), DetectionError> {
        if classes.is_empty() {
            return Ok(());
        }
        let shares = self.shares.share_of(classes);
        let sureness = Plane {
            values: sureness_of_shares(&shares.values, &PART_UNSURE),
            ..shares
        };
        let refined = refined_over(self.photo, &self.block, &sureness)?;
        canvas.cover(&self.block, &refined);
        Ok(())
    }
}

impl<Source: ModelSource, Runner: ModelRunner> ZoneDetector<Source, Runner> {
    /// The persons of the photo, from left to right.
    pub fn persons(
        &self,
        photo: &dyn PhotoView,
        on_step: &mut dyn FnMut(DetectionStep),
    ) -> Result<Vec<PhotoRegion>, DetectionError> {
        let view = ModelView {
            photo,
            region: PhotoRegion::WHOLE,
            input: PIXELS_INPUT,
            side: MODEL_SIDE,
        };
        let outputs = self.outputs_seeing(&PERSONS_MODEL, &view, on_step)?;
        let [boxes, scores] = &outputs[..] else {
            return Err(unexpected_answer("not boxes and scores"));
        };
        Ok(persons_found(boxes, scores))
    }

    fn classes_seen<'a>(
        &self,
        model: &'static Model,
        (photo, region): (&'a dyn PhotoView, &PhotoRegion),
        on_step: &mut dyn FnMut(DetectionStep),
    ) -> Result<ClassesSeen<'a>, DetectionError> {
        let image_size = coverage_image_size(photo.photo_size());
        let block = region.texel_block(image_size);
        let view = ModelView {
            photo,
            region: block.region(image_size),
            input: PIXELS_INPUT,
            side: MODEL_SIDE,
        };
        let logits = first_output(self.outputs_seeing(model, &view, on_step)?)?;
        let shares = ClassShares::of_logits(&logits)
            .ok_or_else(|| unexpected_answer("not a square map a class"))?;
        Ok(ClassesSeen {
            photo,
            block,
            shares,
        })
    }

    fn cover_face_parts(
        &self,
        body: &ClassesSeen<'_>,
        coverages: &mut PartCoverages,
        on_step: &mut dyn FnMut(DetectionStep),
    ) -> Result<(), DetectionError> {
        let Some(face_in_body) = body.shares.extent_of(BODY_FACE_CLASS) else {
            return Ok(());
        };
        let photo = body.photo;
        let face = body.block.region(coverages.size).part(&face_in_body);
        let seen = face_surroundings(&face, photo.photo_size());
        let face = self.classes_seen(&FACE_PARTS_MODEL, (photo, &seen), on_step)?;
        for (part, canvas) in &mut coverages.of_parts {
            face.cover(canvas, face_classes(*part))?;
        }
        face.cover(&mut coverages.not_skin, &FACE_CLASSES_NOT_SKIN)
    }

    /// One mask a part asked, covering it on every person asked; a part
    /// found on none of them has no mask.
    pub fn person_parts(
        &self,
        photo: &dyn PhotoView,
        asked: &PartsAsked<'_>,
        on_step: &mut dyn FnMut(DetectionStep),
    ) -> Result<Vec<ZoneMask>, DetectionError> {
        let size = coverage_image_size(photo.photo_size());
        let mut coverages = PartCoverages::of(asked.parts, size);
        for person in asked.persons {
            let seen = person.grown(PERSON_MARGIN);
            let body = self.classes_seen(&BODY_PARTS_MODEL, (photo, &seen), on_step)?;
            for (part, canvas) in &mut coverages.of_parts {
                body.cover(canvas, body_classes(*part))?;
            }
            if coverages.needs_faces() {
                self.cover_face_parts(&body, &mut coverages, on_step)?;
            }
        }
        Ok(coverages.into_masks())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Read;
    use std::path::Path;

    use super::*;
    use crate::develop::domain::zone::Zone;
    use crate::models::application::model_store::ModelStore;
    use crate::models::domain::model::RemoteFile;
    use crate::models::domain::model_runner::{ModelInput, ModelOutput};

    const PHOTO_SIZE: [u32; 2] = [2048, 1024];
    // Wide enough for the refinement to leave the middle of a part alone.
    const MAP_SIDE: usize = 32;
    const BODY_CLASSES: usize = 18;
    const FACE_CLASSES: usize = 19;

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

    // A map where each row is of one class.
    fn rows_of_classes(classes: usize, class_of_row: impl Fn(usize) -> usize) -> ModelOutput {
        let texels = MAP_SIDE * MAP_SIDE;
        let values = (0..classes * texels).map(|index| {
            let (class, row) = (index / texels, index % texels / MAP_SIDE);
            if class_of_row(row) == class {
                8.0
            } else {
                -8.0
            }
        });
        ModelOutput {
            shape: vec![1, classes, MAP_SIDE, MAP_SIDE],
            values: values.collect(),
        }
    }

    /// Two persons side by side; each has hair on top, then a face, then a
    /// shirt; the face has eyes above lips.
    struct TwoPersons {
        have_faces: bool,
    }

    impl ModelRunner for TwoPersons {
        fn outputs(&self, model_file: &Path, _: ModelInput) -> Result<Vec<ModelOutput>, String> {
            let name = model_file.file_name().unwrap().to_string_lossy();
            if name.starts_with(PERSONS_MODEL.name) {
                let boxes = vec![0.75, 0.5, 0.2, 0.8, 0.25, 0.5, 0.2, 0.8];
                return Ok(vec![
                    ModelOutput {
                        shape: vec![1, 2, 4],
                        values: boxes,
                    },
                    ModelOutput {
                        shape: vec![1, 2, 2],
                        values: vec![-5.0, 5.0, -5.0, 5.0],
                    },
                ]);
            }
            if name.starts_with(BODY_PARTS_MODEL.name) {
                let have_faces = self.have_faces;
                return Ok(vec![rows_of_classes(BODY_CLASSES, |row| match row {
                    0..8 => 2,
                    8..16 if have_faces => BODY_FACE_CLASS,
                    _ => 4,
                })]);
            }
            Ok(vec![rows_of_classes(FACE_CLASSES, |row| match row {
                2..14 => 4,
                22..30 => 11,
                _ => 1,
            })])
        }
    }

    fn detector(
        models: &tempfile::TempDir,
        have_faces: bool,
    ) -> ZoneDetector<NoNetwork, TwoPersons> {
        for model in [PERSONS_MODEL, BODY_PARTS_MODEL, FACE_PARTS_MODEL] {
            fs::write(models.path().join(model.file_name()), b"weights").unwrap();
        }
        let store = ModelStore::new(models.path().to_owned(), NoNetwork);
        ZoneDetector::new(store, TwoPersons { have_faces })
    }

    fn zones_of(masks: &[ZoneMask]) -> Vec<Zone> {
        masks.iter().map(|mask| mask.zone).collect()
    }

    #[test]
    fn persons_are_found_from_left_to_right() {
        let models = tempfile::tempdir().unwrap();

        let persons = detector(&models, true)
            .persons(&GreyPhoto, &mut |_| {})
            .unwrap();

        let centres: Vec<f32> = persons.iter().map(|person| person.centre()[0]).collect();
        assert_eq!(centres, [0.25, 0.75]);
    }

    #[test]
    fn each_part_asked_is_one_mask_over_the_persons_asked() {
        let models = tempfile::tempdir().unwrap();
        let detector = detector(&models, true);
        let persons = detector.persons(&GreyPhoto, &mut |_| {}).unwrap();
        let asked = PartsAsked {
            persons: &persons[..1],
            parts: &PersonPart::ALL,
        };

        let masks = detector
            .person_parts(&GreyPhoto, &asked, &mut |_| {})
            .unwrap();

        let all_zones = PersonPart::ALL.map(PersonPart::zone);
        assert_eq!(zones_of(&masks), all_zones);
        let [skin, hair, eyes, _, clothes] = &masks[..] else {
            panic!("{} masks", masks.len());
        };
        let at = |mask: &ZoneMask, share: [f32; 2]| mask.coverage.coverage_at_share(share);
        assert!(at(hair, [0.25, 0.1]) > 0.9 && at(hair, [0.25, 0.8]) < 0.1);
        assert!(at(clothes, [0.25, 0.8]) > 0.9 && at(clothes, [0.25, 0.1]) < 0.1);
        assert!(at(skin, [0.25, 0.45]) > 0.5 && at(skin, [0.25, 0.8]) < 0.1);
        assert_eq!(at(hair, [0.75, 0.1]), 0.0);
        assert!(eyes.coverage.covers_something() && at(eyes, [0.25, 0.8]) == 0.0);
    }

    #[test]
    fn skin_leaves_out_the_eyes() {
        let models = tempfile::tempdir().unwrap();
        let detector = detector(&models, true);
        let persons = detector.persons(&GreyPhoto, &mut |_| {}).unwrap();
        let asked = PartsAsked {
            persons: &persons[..1],
            parts: &[PersonPart::Skin, PersonPart::Eyes],
        };

        let masks = detector
            .person_parts(&GreyPhoto, &asked, &mut |_| {})
            .unwrap();

        let both_cover = masks[0]
            .coverage
            .values()
            .iter()
            .zip(masks[1].coverage.values())
            .any(|(skin, eyes)| *skin > 200 && *eyes > 200);
        assert!(!both_cover);
    }

    #[test]
    fn part_found_on_nobody_has_no_mask() {
        let models = tempfile::tempdir().unwrap();
        let detector = detector(&models, false);
        let persons = detector.persons(&GreyPhoto, &mut |_| {}).unwrap();
        let asked = PartsAsked {
            persons: &persons,
            parts: &[PersonPart::Eyes, PersonPart::Hair, PersonPart::Lips],
        };

        let masks = detector
            .person_parts(&GreyPhoto, &asked, &mut |_| {})
            .unwrap();

        assert_eq!(zones_of(&masks), [Zone::Hair]);
    }
}
