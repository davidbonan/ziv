use std::path::PathBuf;

use image::RgbaImage;
use ziv::engine::infrastructure::display_readback::DisplayPixels;

/// GPU float math is not bit-exact across drivers.
const TOLERANCE: u8 = 2;
const UPDATE_SWITCH: &str = "ZIV_UPDATE_GOLDEN";

fn golden_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(format!("{name}.png"))
}

fn rejected_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}.actual.png"))
}

pub fn assert_matches_golden(name: &str, actual: &DisplayPixels) {
    let actual = RgbaImage::from_raw(actual.width, actual.height, actual.rgba.clone())
        .expect("readback size matches its pixels");
    let path = golden_path(name);

    if std::env::var_os(UPDATE_SWITCH).is_some() {
        actual.save(&path).expect("golden is writable");
        return;
    }

    let golden = image::open(&path)
        .unwrap_or_else(|error| {
            panic!(
                "{}: {error} — create it with {UPDATE_SWITCH}=1",
                path.display()
            )
        })
        .into_rgba8();
    let matches = golden.dimensions() == actual.dimensions()
        && golden
            .iter()
            .zip(actual.iter())
            .all(|(golden, actual)| golden.abs_diff(*actual) <= TOLERANCE);
    if !matches {
        let rejected = rejected_path(name);
        actual.save(&rejected).expect("target tmpdir is writable");
        panic!(
            "{name} differs from its golden by more than {TOLERANCE}; actual output: {}",
            rejected.display()
        );
    }
}
