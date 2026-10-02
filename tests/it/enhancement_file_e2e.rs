use std::fs;
use std::path::PathBuf;

use ziv::enhance::domain::enhancement::Enhancement;
use ziv::enhance::domain::enhancement_storage::EnhancementStorage;
use ziv::enhance::domain::model_encoding::ModelEncoding;
use ziv::enhance::infrastructure::enhancement_files::{EnhancementFiles, enhancement_path};

const SIZE: [u32; 2] = [96, 64];

/// Differences like those of removed noise: small, other at every pixel and channel.
fn noise_removed() -> Enhancement {
    let mut state = 0x2545_f491_u32;
    let levels = (0..SIZE[0] * SIZE[1] * 3).map(|_| {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (118 + (state >> 24) % 21) as u8
    });
    Enhancement::new(SIZE, ModelEncoding::with_ceiling(2.5), levels.collect()).unwrap()
}

fn photo_in(folder: &tempfile::TempDir) -> PathBuf {
    folder.path().join("DSC07070.ARW")
}

#[test]
fn stored_enhancement_comes_back_as_it_was_written() {
    let folder = tempfile::tempdir().unwrap();
    let photo = photo_in(&folder);
    let enhancement = noise_removed();

    EnhancementFiles
        .store_enhancement(&photo, &enhancement)
        .unwrap();
    let stored = EnhancementFiles
        .stored_enhancement(&photo, SIZE)
        .unwrap()
        .unwrap();

    assert_eq!(
        enhancement_path(&photo),
        folder.path().join("DSC07070.ARW.ziv.enhanced")
    );
    assert_eq!(fs::read_dir(folder.path()).unwrap().count(), 1);
    assert_eq!(stored, enhancement);
}

#[test]
fn photo_without_file_or_of_another_size_has_no_enhancement() {
    let folder = tempfile::tempdir().unwrap();
    let photo = photo_in(&folder);

    let without_file = EnhancementFiles.stored_enhancement(&photo, SIZE);
    EnhancementFiles
        .store_enhancement(&photo, &noise_removed())
        .unwrap();
    let of_another_size = EnhancementFiles.stored_enhancement(&photo, [64, 96]);

    assert_eq!(without_file, Ok(None));
    assert_eq!(of_another_size, Ok(None));
}

#[test]
fn file_that_is_not_an_enhancement_is_reported() {
    let folder = tempfile::tempdir().unwrap();
    let photo = photo_in(&folder);
    fs::write(
        enhancement_path(&photo),
        "not an enhancement, and long enough",
    )
    .unwrap();

    let stored = EnhancementFiles.stored_enhancement(&photo, SIZE);

    assert_eq!(stored, Err("it is not an enhancement file".to_owned()));
}
