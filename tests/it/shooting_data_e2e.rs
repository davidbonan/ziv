use std::path::PathBuf;

use ziv::photo::domain::shooting_data::ShootingData;
use ziv::photo::infrastructure::file_decoder::FileDecoder;

const SHOT: [&str; 4] = ["ISO 400", "35 mm", "f/2.8", "1/250 s"];

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn shooting_data_of(name: &str) -> ShootingData {
    FileDecoder.decode(&fixture(name)).unwrap().shooting_data
}

#[test]
fn shooting_data_are_read_from_a_jpeg_a_png_and_a_tiff() {
    for name in [
        "patches_with_shooting_data.jpg",
        "patches_with_shooting_data.png",
        "patches_with_shooting_data.tiff",
    ] {
        assert_eq!(shooting_data_of(name).labels(), SHOT, "{name}");
    }
}

#[test]
fn a_file_without_shooting_data_gives_none() {
    for name in ["patches.png", "patches.tiff", "patches_rotated_90_cw.jpg"] {
        assert_eq!(shooting_data_of(name), ShootingData::default(), "{name}");
    }
}

#[test]
fn shooting_data_are_read_from_a_raw() {
    let path = fixture("local/DSC07070.ARW");
    if !path.exists() {
        eprintln!("skipped: {} is not on this machine", path.display());
        return;
    }

    let labels = FileDecoder.decode(&path).unwrap().shooting_data.labels();

    assert_eq!(labels, ["ISO 125", "28 mm", "f/2.8", "1/4000 s"]);
}
