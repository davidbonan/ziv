use std::path::PathBuf;
use std::time::Instant;

use ziv::develop::domain::adjustments::Adjustments;
use ziv::develop::domain::development::Development;
use ziv::develop::domain::edit::Edit;
use ziv::histogram::domain::histogram::Histogram;
use ziv::histogram::infrastructure::developed_histogram::{DevelopedHistogram, histogram_of};
use ziv::photo::infrastructure::raw_file::decode_raw_file;

use crate::gpu::headless_engine;
use crate::synthetic::{HEIGHT, WIDTH, ramp_and_patches};

const COUNTS_TIMED: u32 = 20;

fn exposed(exposure: f32) -> Development {
    Development {
        edit: Edit::from(Adjustments {
            exposure,
            ..Adjustments::default()
        }),
        ..Development::default()
    }
}

fn mean_level(histogram: &Histogram) -> f32 {
    let counts = histogram.luminance();
    let weighted: f32 = (0..counts.len())
        .map(|level| level as f32 * counts[level] as f32)
        .sum();
    weighted / counts.iter().sum::<u32>() as f32
}

#[test]
fn the_histogram_of_a_photo_is_that_of_its_developed_pixels() {
    let engine = headless_engine();
    let source = engine.upload(&ramp_and_patches());

    let histogram = histogram_of(&engine, &source, &exposed(0.5)).unwrap();

    let pixels = engine
        .render_pixels(&source, &exposed(0.5), [WIDTH, HEIGHT])
        .unwrap();
    assert_eq!(histogram, Histogram::of_display_pixels(&pixels.rgba));
    assert_eq!(histogram.luminance().iter().sum::<u32>(), WIDTH * HEIGHT);
}

#[test]
fn the_histogram_moves_with_the_edit_and_is_counted_once_per_development() {
    let engine = headless_engine();
    let source = engine.upload(&ramp_and_patches());
    let mut developed = DevelopedHistogram::default();

    assert!(developed.follow(&engine, &source, &exposed(-2.0)));
    let darker = mean_level(developed.latest().unwrap());
    assert!(!developed.follow(&engine, &source, &exposed(-2.0)));
    assert!(developed.follow(&engine, &source, &exposed(1.0)));

    assert!(mean_level(developed.latest().unwrap()) > darker + 40.0);
}

#[test]
fn the_cost_of_one_histogram_of_a_large_raw_is_measured() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/local/DSC07070.ARW");
    if !path.exists() {
        eprintln!("skipped: {} is not on this machine", path.display());
        return;
    }
    let engine = headless_engine();
    let source = engine.upload(&decode_raw_file(&path).unwrap().image);
    histogram_of(&engine, &source, &exposed(0.0)).unwrap();

    let start = Instant::now();
    for count in 0..COUNTS_TIMED {
        histogram_of(&engine, &source, &exposed(count as f32 / 10.0)).unwrap();
    }
    let each = start.elapsed() / COUNTS_TIMED;

    eprintln!("one histogram in {each:?}");
}
