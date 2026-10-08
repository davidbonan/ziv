use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::time::Instant;

use rawler::dng::convert::{ConvertParams, convert_raw_file};
use ziv::develop::domain::development::Development;
use ziv::engine::infrastructure::display_stage::DisplayRequest;
use ziv::photo::domain::photo_kind::PhotoKind;
use ziv::photo::domain::thumbnail::thumbnail_size;
use ziv::photo::infrastructure::raw_file::{
    decode_raw_embedded_picture, decode_raw_file, decode_raw_thumbnail,
};

use crate::gpu::headless_engine;

/// A RAW kept out of the repository: present only on a machine where it was
/// put into `tests/fixtures/local/`.
fn local_raw(name: &str) -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/local")
        .join(name);
    if !path.exists() {
        eprintln!("skipped: {} is not on this machine", path.display());
        return None;
    }
    Some(path)
}

/// The author's own RAW, not redistributable.
fn local_sony_a7m4_raw() -> Option<PathBuf> {
    local_raw("DSC07070.ARW")
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

#[test]
fn leica_q2_raw_calibrated_under_d50_and_not_d65_is_decoded_in_daylight() {
    let Some(path) = local_raw("raws/L1000750.DNG") else {
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

#[test]
fn embedded_picture_of_a_raw_is_read_without_developing_its_sensor_data() {
    let Some(path) = local_sony_a7m4_raw() else {
        return;
    };

    let start = Instant::now();
    let preview = decode_raw_embedded_picture(&path).unwrap().unwrap();
    eprintln!("embedded picture decoded in {:?}", start.elapsed());

    assert_eq!(preview.kind, PhotoKind::StandardImage);
    assert_eq!(
        (preview.image.width(), preview.image.height()),
        (1616, 1080)
    );
    assert_ne!(preview.shooting_data, Default::default());
}

#[test]
fn a_raw_embedding_no_picture_gives_none() {
    let Some(path) = local_sony_a7m4_raw() else {
        return;
    };
    let folder = tempfile::tempdir().unwrap();
    let without_picture = folder.path().join("bare.dng");
    let bare = ConvertParams {
        embedded: false,
        preview: false,
        thumbnail: false,
        ..ConvertParams::default()
    };
    let mut file = BufWriter::new(File::create(&without_picture).unwrap());
    convert_raw_file(&path, &mut file, &bare).unwrap();
    drop(file);

    assert_eq!(decode_raw_embedded_picture(&without_picture).unwrap(), None);
}
