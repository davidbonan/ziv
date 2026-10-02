use std::path::PathBuf;
use std::time::Instant;

use ziv::develop::domain::development::Development;
use ziv::engine::infrastructure::display_stage::DisplayRequest;
use ziv::photo::domain::photo_kind::PhotoKind;
use ziv::photo::domain::thumbnail::thumbnail_size;
use ziv::photo::infrastructure::raw_file::{decode_raw_file, decode_raw_thumbnail};

use crate::gpu::headless_engine;

/// The author's own RAW, not redistributable: present only on a machine where
/// it was linked into `tests/fixtures/local/`.
fn local_sony_a7m4_raw() -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/local/DSC07070.ARW");
    if !path.exists() {
        eprintln!("skipped: {} is not on this machine", path.display());
        return None;
    }
    Some(path)
}

#[test]
fn sony_a7m4_raw_decodes_to_the_camera_default_crop() {
    let Some(path) = local_sony_a7m4_raw() else {
        return;
    };

    let start = Instant::now();
    let image = decode_raw_file(&path).unwrap().image;
    eprintln!("decoded in {:?}", start.elapsed());

    assert_eq!((image.width(), image.height()), (7008, 4672));
    assert!(
        image
            .pixels()
            .iter()
            .flatten()
            .all(|channel| channel.is_finite())
    );
    let brightest = image.pixels().iter().flatten().copied().fold(0.0, f32::max);
    assert!(
        brightest > 0.1,
        "image is black: brightest value {brightest}"
    );
}

#[test]
fn a_file_that_is_not_a_raw_reports_why() {
    let not_a_raw = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/patches.png");

    let error = decode_raw_file(&not_a_raw).unwrap_err();

    assert!(!error.to_string().is_empty());
}

fn mean_brightness(rgba: &[u8]) -> f32 {
    let (pixels, _) = rgba.as_chunks::<4>();
    let sum: f32 = pixels
        .iter()
        .map(|[red, green, blue, _]| f32::from(*red) + f32::from(*green) + f32::from(*blue))
        .sum();
    sum / (3.0 * 255.0 * pixels.len() as f32)
}

#[test]
fn untouched_raw_is_about_as_bright_as_its_camera_preview() {
    let Some(path) = local_sony_a7m4_raw() else {
        return;
    };
    let engine = headless_engine();
    let decoded = decode_raw_file(&path).unwrap();
    let image = &decoded.image;
    let request = DisplayRequest {
        development: Development {
            kind: decoded.kind,
            ..Development::default()
        },
        ..DisplayRequest::whole_source(thumbnail_size(image.width(), image.height()))
    };

    let output = engine.render_display(&engine.upload(image), &request);
    let rendered = mean_brightness(&engine.read_display_pixels(&output).unwrap().rgba);
    let preview = mean_brightness(&decode_raw_thumbnail(&path).unwrap().rgba);

    assert!(
        (rendered / preview - 1.0).abs() < 0.15,
        "rendered {rendered}, camera preview {preview}"
    );
}

#[test]
fn sony_a7m4_raw_tells_the_daylight_it_was_balanced_for() {
    let Some(path) = local_sony_a7m4_raw() else {
        return;
    };

    let PhotoKind::Raw { as_shot } = decode_raw_file(&path).unwrap().kind else {
        panic!("a RAW file decodes to a RAW photo");
    };

    assert!(
        (4000.0..7500.0).contains(&as_shot.temperature) && as_shot.tint.abs() < 60.0,
        "{as_shot:?}"
    );
}
