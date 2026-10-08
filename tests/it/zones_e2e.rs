use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use ziv::develop::domain::coverage_image::CoverageImage;
use ziv::develop::domain::preset::Preset;
use ziv::develop::domain::zone::{PersonPart, Zone, ZoneMask, ZoneTool};
use ziv::models::application::model_store::ModelStore;
use ziv::models::domain::model::Model;
use ziv::models::infrastructure::model_downloads::ModelDownloads;
use ziv::models::infrastructure::models_folder::models_folder;
use ziv::models::infrastructure::onnx_runner::OnnxRunner;
use ziv::photo::infrastructure::file_decoder::FileDecoder;
use ziv::zones::application::person_zones::PartsAsked;
use ziv::zones::application::zone_detector::ZoneDetector;
use ziv::zones::domain::zone_models::{
    BODY_PARTS_MODEL, FACE_PARTS_MODEL, PERSONS_MODEL, SKY_MODEL, SUBJECT_MODEL,
};
use ziv::zones::infrastructure::engine_photo_view::EnginePhotoView;

use crate::gpu::headless_engine;

/// A photo of the author's, not redistributable: present only on a machine
/// where it was put into `tests/fixtures/local/`.
fn local_photo(name: &str) -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/local")
        .join(name);
    if !path.exists() {
        eprintln!("skipped: {} is not on this machine", path.display());
        return None;
    }
    Some(path)
}

/// Tests never download: a detector only when the models are already here.
fn local_detector(models: &[Model]) -> Option<ZoneDetector<ModelDownloads, OnnxRunner>> {
    let folder = models_folder()?;
    let missing = models
        .iter()
        .find(|model| !folder.join(model.file_name()).is_file());
    if let Some(model) = missing {
        eprintln!("skipped: the {} model is not on this machine", model.name);
        return None;
    }
    let store = ModelStore::new(folder, ModelDownloads);
    Some(ZoneDetector::new(store, OnnxRunner::default()))
}

fn viewed(photo: &Path) -> EnginePhotoView {
    let engine = Arc::new(headless_engine());
    let decoded = FileDecoder.decode(photo).unwrap();
    EnginePhotoView {
        source: Arc::new(engine.upload(&decoded.image)),
        engine,
        kind: decoded.kind,
    }
}

fn save_for_inspection(coverage: &CoverageImage, name: &str) {
    let Ok(folder) = std::env::var("ZIV_ZONES_OUT") else {
        return;
    };
    let [width, height] = coverage.size();
    image::GrayImage::from_raw(width, height, coverage.values().to_vec())
        .unwrap()
        .save(PathBuf::from(folder).join(name))
        .unwrap();
}

#[test]
fn subject_of_the_castle_photo_is_the_castle_and_not_its_sky() {
    let (Some(photo), Some(detector)) = (
        local_photo("DSC07070.ARW"),
        local_detector(&[SUBJECT_MODEL]),
    ) else {
        return;
    };
    let view = viewed(&photo);

    let started = Instant::now();
    let masks = detector
        .zone_masks(ZoneTool::Subject, &view, &mut |_| {})
        .unwrap();
    eprintln!("subject detected in {:?}", started.elapsed());
    let coverage = &masks[0].coverage;
    save_for_inspection(coverage, "subject_castle.png");

    assert_eq!(coverage.size(), [2048, 1366]);
    for on_the_castle in [[0.5, 0.8], [0.75, 0.15], [0.25, 0.3]] {
        assert!(coverage.coverage_at_share(on_the_castle) > 0.95);
    }
    for around_it in [[0.05, 0.05], [0.05, 0.9], [0.95, 0.95]] {
        assert!(coverage.coverage_at_share(around_it) < 0.05);
    }
}

#[test]
fn sky_of_the_landscape_is_above_the_mountains() {
    let (Some(photo), Some(detector)) =
        (local_photo("landscape.jpg"), local_detector(&[SKY_MODEL]))
    else {
        return;
    };

    let masks = detector
        .zone_masks(ZoneTool::Sky, &viewed(&photo), &mut |_| {})
        .unwrap();

    let coverage = &masks[0].coverage;
    save_for_inspection(coverage, "sky_landscape.png");
    assert!(coverage.coverage_at_share([0.5, 0.03]) > 0.95);
    for below_the_sky in [[0.5, 0.4], [0.5, 0.8]] {
        assert!(coverage.coverage_at_share(below_the_sky) < 0.05);
    }
}

#[test]
fn photo_taken_indoors_has_no_sky() {
    let (Some(photo), Some(detector)) = (
        local_photo("group_of_four.jpg"),
        local_detector(&[SKY_MODEL]),
    ) else {
        return;
    };

    let masks = detector.zone_masks(ZoneTool::Sky, &viewed(&photo), &mut |_| {});

    assert_eq!(masks, Ok(Vec::new()));
}

/// The masks of `parts` on every person of the photo, and how many persons that is.
fn parts_of_everyone(photo: &str, parts: &[PersonPart]) -> Option<(usize, Vec<ZoneMask>)> {
    let models = [PERSONS_MODEL, BODY_PARTS_MODEL, FACE_PARTS_MODEL];
    let (Some(photo), Some(detector)) = (local_photo(photo), local_detector(&models)) else {
        return None;
    };
    let view = viewed(&photo);
    let persons = detector.persons(&view, &mut |_| {}).unwrap();
    let asked = PartsAsked {
        persons: &persons,
        parts,
    };
    let started = Instant::now();
    let masks = detector.person_parts(&view, &asked, &mut |_| {}).unwrap();
    eprintln!(
        "parts of {} persons in {:?}",
        persons.len(),
        started.elapsed()
    );
    for mask in &masks {
        save_for_inspection(&mask.coverage, &format!("{:?}.png", mask.zone));
    }
    Some((persons.len(), masks))
}

/// The middle of what the mask covers, in shares of the photo.
fn middle_of(mask: &ZoneMask) -> [f32; 2] {
    let [width, height] = mask.coverage.size();
    let covered = (0..width * height).filter(|texel| mask.coverage.values()[*texel as usize] > 127);
    let (count, across, down) = covered.fold((0, 0, 0), |(count, across, down), texel| {
        (count + 1, across + texel % width, down + texel / width)
    });
    [
        across as f32 / count as f32 / width as f32,
        down as f32 / count as f32 / height as f32,
    ]
}

fn covered(mask: &ZoneMask, share: [f32; 2]) -> f32 {
    mask.coverage.coverage_at_share(share)
}

#[test]
fn parts_of_four_persons_sitting_together() {
    let parts = [PersonPart::Skin, PersonPart::Hair, PersonPart::Clothes];
    let Some((persons, masks)) = parts_of_everyone("group_of_four.jpg", &parts) else {
        return;
    };
    let on_the_table = [0.6, 0.8];
    let [skin, hair, clothes] = &masks[..] else {
        panic!("{} masks", masks.len());
    };

    assert_eq!(persons, 4);
    assert!(covered(skin, [0.497, 0.352]) > 0.5, "a cheek");
    assert!(
        covered(hair, [0.117, 0.42]) > 0.5,
        "long hair seen from behind"
    );
    assert!(covered(clothes, [0.484, 0.493]) > 0.9, "a T-shirt");
    for mask in &masks {
        assert!(covered(mask, on_the_table) < 0.1, "{:?}", mask.zone);
    }
}

/// What `preset` masks in `photo`, and nothing where the models or the photo are absent.
fn preset_masks(preset: Preset, photo: &str) -> Option<Vec<ZoneMask>> {
    let models = [PERSONS_MODEL, BODY_PARTS_MODEL, FACE_PARTS_MODEL];
    let (Some(photo), Some(detector)) = (local_photo(photo), local_detector(&models)) else {
        return None;
    };
    let masks = detector.preset_zone_masks(preset, &viewed(&photo), &mut |_| {});
    Some(masks.unwrap())
}

/// Whether `mask` covers something in the columns between two shares of the photo's width.
fn covers_between(mask: &ZoneMask, from: f32, to: f32) -> bool {
    let [width, height] = mask.coverage.size();
    let columns = (from * width as f32) as u32..(to * width as f32) as u32;
    (0..height)
        .flat_map(|row| columns.clone().map(move |column| row * width + column))
        .any(|texel| mask.coverage.values()[texel as usize] > 127)
}

#[test]
fn bright_eyes_of_a_wedding_group_is_one_mask_covering_eyes_from_end_to_end() {
    let ten_persons_in_a_row = "raws-people/Matrimonio.CR3";
    let Some(masks) = preset_masks(Preset::BrightEyes, ten_persons_in_a_row) else {
        return;
    };

    let [eyes] = &masks[..] else {
        panic!("{} masks", masks.len());
    };
    assert_eq!(eyes.zone, Zone::Eyes);
    assert!(covers_between(eyes, 0.15, 0.25), "the man at the left end");
    assert!(covers_between(eyes, 0.45, 0.52), "the bride");
    assert!(covers_between(eyes, 0.8, 0.9), "the man at the right end");
    assert!(!covers_between(eyes, 0.0, 0.1), "the wall");
}

#[test]
fn bright_eyes_of_a_portrait_covers_its_eyes() {
    let Some(masks) = preset_masks(Preset::BrightEyes, "portrait.jpg") else {
        return;
    };

    let between_the_eyes = [0.468, 0.304];
    assert!(
        is_near(middle_of(&masks[0]), between_the_eyes),
        "{:?}",
        middle_of(&masks[0])
    );
}

fn is_near(place: [f32; 2], expected: [f32; 2]) -> bool {
    (place[0] - expected[0]).abs() < 0.02 && (place[1] - expected[1]).abs() < 0.02
}

#[test]
fn parts_of_a_portrait() {
    let Some((persons, masks)) = parts_of_everyone("portrait.jpg", &PersonPart::ALL) else {
        return;
    };
    let (between_the_eyes, the_mouth, the_forehead) =
        ([0.468, 0.304], [0.462, 0.375], [0.468, 0.258]);
    let [skin, hair, eyes, lips, clothes] = &masks[..] else {
        panic!("{} masks", masks.len());
    };

    assert_eq!(persons, 1);
    assert!(
        is_near(middle_of(eyes), between_the_eyes),
        "{:?}",
        middle_of(eyes)
    );
    assert!(is_near(middle_of(lips), the_mouth), "{:?}", middle_of(lips));
    assert!(covered(skin, the_forehead) > 0.9 && covered(skin, the_mouth) < 0.2);
    assert!(covered(hair, [0.6, 0.45]) > 0.5 && covered(hair, the_forehead) < 0.1);
    assert!(covered(clothes, [0.47, 0.6]) > 0.9 && covered(clothes, the_forehead) < 0.1);
}

/// The teeth of everyone in `photo`, and nothing where the models or the photo are absent.
fn teeth_of_everyone(photo: &str) -> Option<Vec<ZoneMask>> {
    let models = [PERSONS_MODEL, BODY_PARTS_MODEL, FACE_PARTS_MODEL];
    let (Some(photo), Some(detector)) = (local_photo(photo), local_detector(&models)) else {
        return None;
    };
    let masks = detector.teeth_of_everyone(&viewed(&photo), &mut |_| {});
    Some(masks.unwrap())
}

#[test]
fn teeth_of_a_smiling_portrait_are_covered_and_its_lips_and_skin_are_left() {
    let Some(masks) = teeth_of_everyone("portrait.jpg") else {
        return;
    };
    let (a_front_tooth, the_lower_lip, a_cheek) = ([0.452, 0.369], [0.457, 0.388], [0.40, 0.36]);

    let [teeth] = &masks[..] else {
        panic!("{} masks", masks.len());
    };
    assert_eq!(teeth.zone, Zone::Teeth);
    assert!(covered(teeth, a_front_tooth) > 0.8, "a front tooth");
    assert!(covered(teeth, the_lower_lip) < 0.2, "the lower lip");
    assert!(covered(teeth, a_cheek) < 0.05, "a cheek");
}

#[test]
fn whiter_teeth_masks_the_teeth_of_a_smiling_portrait() {
    let Some(masks) = preset_masks(Preset::WhiterTeeth, "portrait.jpg") else {
        return;
    };

    let the_mouth = [0.462, 0.375];
    assert_eq!(masks[0].zone, Zone::Teeth);
    assert!(
        is_near(middle_of(&masks[0]), the_mouth),
        "{:?}",
        middle_of(&masks[0])
    );
}

#[test]
fn closed_mouth_shows_no_teeth() {
    let Some(masks) = teeth_of_everyone("raws-people/20200927_19.30.38_DSC_2839.nef") else {
        return;
    };

    assert_eq!(masks, Vec::new());
}
