use std::path::PathBuf;

use ziv::photo::domain::thumbnail::{THUMBNAIL_MAX_SIDE, Thumbnail};
use ziv::photo::infrastructure::file_decoder::FileDecoder;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn top_left(thumbnail: &Thumbnail) -> [u8; 3] {
    [thumbnail.rgba[0], thumbnail.rgba[1], thumbnail.rgba[2]]
}

#[test]
fn standard_image_thumbnail_is_upright() {
    let thumbnail = FileDecoder
        .decode_thumbnail(&fixture("patches_rotated_90_cw.jpg"))
        .unwrap();

    assert_eq!((thumbnail.width, thumbnail.height), (32, 48));
    assert!(
        top_left(&thumbnail).iter().all(|channel| *channel > 250),
        "white patch expected top-left"
    );
}

#[test]
fn raw_thumbnail_fits_the_thumbnail_bounds() {
    let path = fixture("local/DSC07070.ARW");
    if !path.exists() {
        eprintln!("skipped: {} is not on this machine", path.display());
        return;
    }

    let thumbnail = FileDecoder.decode_thumbnail(&path).unwrap();

    assert_eq!(thumbnail.width, THUMBNAIL_MAX_SIDE);
    assert!(thumbnail.height < thumbnail.width, "landscape photo");
    assert_eq!(
        thumbnail.rgba.len(),
        (thumbnail.width * thumbnail.height * 4) as usize
    );
}
