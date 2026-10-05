use std::path::PathBuf;

use ziv::photo::infrastructure::file_decoder::FileDecoder;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn details_of_a_standard_image_give_its_upright_size_and_its_file_size() {
    let landscape = FileDecoder.read_details(&fixture("patches.png"));
    let turned = FileDecoder.read_details(&fixture("patches_rotated_90_cw.jpg"));

    assert_eq!(landscape.pixel_size, Some([48, 32]));
    assert_eq!(turned.pixel_size, Some([32, 48]));
    let on_disk = std::fs::metadata(fixture("patches.png")).unwrap().len();
    assert_eq!(landscape.file_bytes, Some(on_disk));
}

#[test]
fn a_file_that_tells_no_date_has_none() {
    let details = FileDecoder.read_details(&fixture("patches_with_shooting_data.jpg"));

    assert_eq!(details.shot_at, None);
    assert_eq!(details.labels().len(), 2);
}

#[test]
fn details_of_a_raw_give_the_size_it_develops_to_and_when_it_was_shot() {
    let path = fixture("local/DSC07070.ARW");
    if !path.exists() {
        eprintln!("skipped: {} is not on this machine", path.display());
        return;
    }

    let details = FileDecoder.read_details(&path);

    assert_eq!(details.pixel_size, Some([7008, 4672]));
    assert!(details.shot_at.is_some());
    eprintln!("{:?}", details.labels());
}
