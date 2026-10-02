use std::fs;
use std::path::{Path, PathBuf};

use ziv::library::domain::session::Session;
use ziv::library::infrastructure::photo_files::photo_files_among;
use ziv::photo::infrastructure::file_decoder::FileDecoder;

/// A folder holding photos, other files, a hidden file and a sub-folder with a photo.
fn shoot_folder() -> tempfile::TempDir {
    let folder = tempfile::tempdir().unwrap();
    for name in [
        "DSC10.ARW",
        "DSC2.ARW",
        "cover.jpg",
        "notes.txt",
        ".hidden.jpg",
        "DSC2.xmp",
    ] {
        fs::write(folder.path().join(name), b"").unwrap();
    }
    fs::create_dir(folder.path().join("selects")).unwrap();
    fs::write(folder.path().join("selects/nested.jpg"), b"").unwrap();
    folder
}

fn is_photo(path: &Path) -> bool {
    FileDecoder.supports(path)
}

fn names(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect()
}

#[test]
fn opening_a_folder_gives_its_own_photos_in_natural_order() {
    let folder = shoot_folder();

    let session = Session::of(photo_files_among(&[folder.path().to_owned()], is_photo)).unwrap();

    assert_eq!(
        names(session.photos()),
        ["cover.jpg", "DSC2.ARW", "DSC10.ARW"]
    );
    assert_eq!(
        session.selected_photo(),
        Some(folder.path().join("cover.jpg").as_path())
    );
}

#[test]
fn opening_files_keeps_only_the_photos() {
    let folder = shoot_folder();
    let opened = ["DSC2.ARW", "notes.txt", "DSC2.xmp"].map(|name| folder.path().join(name));

    let photos = photo_files_among(&opened, is_photo);

    assert_eq!(names(&photos), ["DSC2.ARW"]);
}

#[test]
fn folder_without_photos_gives_nothing() {
    let folder = tempfile::tempdir().unwrap();
    fs::write(folder.path().join("notes.txt"), b"").unwrap();

    assert!(photo_files_among(&[folder.path().to_owned()], is_photo).is_empty());
}
