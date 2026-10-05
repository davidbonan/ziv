use std::path::PathBuf;
use std::time::Instant;

use ziv::develop::domain::adjustments::Adjustments;
use ziv::develop::domain::development::Development;
use ziv::develop::domain::edit::Edit;
use ziv::develop::domain::framing::{CropFrame, Framing};
use ziv::histogram::domain::histogram::Histogram;
use ziv::histogram::infrastructure::developed_histogram::{
    CountedPhoto, DevelopedHistogram, histogram_of,
};
use ziv::photo::infrastructure::raw_file::decode_raw_file;

use crate::gpu::{headless_engine, whole_source_request};
use crate::synthetic::{HEIGHT, WIDTH, ramp_and_patches};

const COUNTS_TIMED: u32 = 20;

fn exposed(exposure: f32) -> CountedPhoto {
    CountedPhoto {
        development: Development {
            edit: Edit::from(Adjustments {
                exposure,
                ..Adjustments::default()
            }),
            ..Development::default()
        },
        ..CountedPhoto::default()
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
        .render_pixels(
            &source,
            &whole_source_request(&exposed(0.5).development, [WIDTH, HEIGHT]),
        )
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

#[test]
fn the_histogram_counts_the_framed_photo_only() {
    let engine = headless_engine();
    let source = engine.upload(&ramp_and_patches());
    let top_left_quarter = CountedPhoto {
        framing: Framing {
            frame: CropFrame {
                centre: [0.25, 0.25],
                size: [0.5, 0.5],
            },
            ..Framing::default()
        },
        ..CountedPhoto::default()
    };

    let histogram = histogram_of(&engine, &source, &top_left_quarter).unwrap();

    let whole = engine
        .render_pixels(
            &source,
            &whole_source_request(&Development::default(), [WIDTH, HEIGHT]),
        )
        .unwrap();
    let held: Vec<u8> = whole
        .rgba
        .chunks(WIDTH as usize * 4)
        .take(HEIGHT as usize / 2)
        .flat_map(|row| &row[..WIDTH as usize * 2])
        .copied()
        .collect();
    assert_eq!(histogram, Histogram::of_display_pixels(&held));
    assert_eq!(
        histogram.luminance().iter().sum::<u32>(),
        WIDTH * HEIGHT / 4
    );
}

#[test]
fn the_histogram_is_counted_again_when_the_framing_changes() {
    let engine = headless_engine();
    let source = engine.upload(&ramp_and_patches());
    let mut developed = DevelopedHistogram::default();
    let straightened = CountedPhoto {
        framing: Framing::default().straightened([WIDTH, HEIGHT], 5.0),
        ..CountedPhoto::default()
    };

    assert!(developed.follow(&engine, &source, &CountedPhoto::default()));
    assert!(developed.follow(&engine, &source, &straightened));
    assert!(!developed.follow(&engine, &source, &straightened));
}
