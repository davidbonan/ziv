use std::path::PathBuf;

use ziv::color::domain::working_space::DisplayTransform;
use ziv::photo::domain::working_image::WorkingImage;
use ziv::photo::infrastructure::standard_image_file::decode_standard_image_file;

const BLOCK: u32 = 16;
const RED: [u8; 3] = [255, 0, 0];
const GREEN: [u8; 3] = [0, 255, 0];
const BLUE: [u8; 3] = [0, 0, 255];
const WHITE: [u8; 3] = [255, 255, 255];
const BLACK: [u8; 3] = [0, 0, 0];
const GREY: [u8; 3] = [128, 128, 128];

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn displayed_patches(image: &WorkingImage) -> Vec<Vec<[u8; 3]>> {
    let display = DisplayTransform::default();
    (0..image.height() / BLOCK)
        .map(|row| {
            (0..image.width() / BLOCK)
                .map(|column| {
                    let center = image.pixel(column * BLOCK + BLOCK / 2, row * BLOCK + BLOCK / 2);
                    display
                        .to_display(center)
                        .map(|channel| (channel * 255.0).round() as u8)
                })
                .collect()
        })
        .collect()
}

fn assert_patches_within(actual: &[Vec<[u8; 3]>], expected: &[Vec<[u8; 3]>], tolerance: u8) {
    let actual_flat: Vec<u8> = actual.iter().flatten().flatten().copied().collect();
    let expected_flat: Vec<u8> = expected.iter().flatten().flatten().copied().collect();
    assert_eq!(actual_flat.len(), expected_flat.len(), "{actual:?}");
    assert!(
        actual_flat
            .iter()
            .zip(&expected_flat)
            .all(|(actual, expected)| actual.abs_diff(*expected) <= tolerance),
        "{actual:?} differs from {expected:?}"
    );
}

fn stored_patches() -> Vec<Vec<[u8; 3]>> {
    vec![vec![RED, GREEN, BLUE], vec![WHITE, BLACK, GREY]]
}

#[test]
fn png_displays_its_srgb_colors_unchanged() {
    let image = decode_standard_image_file(&fixture("patches.png")).unwrap();

    assert_eq!((image.width(), image.height()), (48, 32));
    assert_patches_within(&displayed_patches(&image), &stored_patches(), 0);
}

#[test]
fn tiff_displays_its_srgb_colors_unchanged() {
    let image = decode_standard_image_file(&fixture("patches.tiff")).unwrap();

    assert_patches_within(&displayed_patches(&image), &stored_patches(), 0);
}

#[test]
fn jpeg_displays_its_srgb_colors_within_compression_error() {
    let image = decode_standard_image_file(&fixture("patches.jpg")).unwrap();

    assert_patches_within(&displayed_patches(&image), &stored_patches(), 2);
}

#[test]
fn exif_orientation_is_applied() {
    let image = decode_standard_image_file(&fixture("patches_rotated_90_cw.jpg")).unwrap();

    let upright = vec![vec![WHITE, RED], vec![BLACK, GREEN], vec![GREY, BLUE]];
    assert_eq!((image.width(), image.height()), (32, 48));
    assert_patches_within(&displayed_patches(&image), &upright, 2);
}

#[test]
fn a_file_that_is_not_an_image_reports_why() {
    let not_an_image = fixture("../it/main.rs");

    let error = decode_standard_image_file(&not_an_image).unwrap_err();

    assert!(!error.to_string().is_empty());
}
