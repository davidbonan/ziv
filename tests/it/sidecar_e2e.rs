use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ziv::develop::domain::adjustments::Adjustments;
use ziv::develop::domain::color_grading::ZoneGrade;
use ziv::develop::domain::color_mixer::ColorRange;
use ziv::develop::domain::coverage_image::CoverageImage;
use ziv::develop::domain::mask::{Mask, MaskShape};
use ziv::develop::domain::tone_curve::ToneCurve;
use ziv::develop::domain::zone::{Zone, ZoneMask};

use ziv::develop::domain::edit::Edit;
use ziv::develop::domain::edit_storage::EditStorage;
use ziv::develop::domain::framing::{Framing, Turn};
use ziv::develop::infrastructure::sidecar_files::{SidecarFiles, sidecar_path};

fn photo_in(folder: &tempfile::TempDir) -> PathBuf {
    folder.path().join("DSC07070.ARW")
}

fn brighter() -> Edit {
    Edit::from(Adjustments {
        exposure: 1.25,
        ..Adjustments::default()
    })
}

fn files_of(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn stored_edit_is_one_sidecar_named_after_the_whole_file_name() {
    let folder = tempfile::tempdir().unwrap();
    let photo = photo_in(&folder);

    SidecarFiles.store_edit(&photo, &brighter()).unwrap();

    assert_eq!(files_of(folder.path()), ["DSC07070.ARW.ziv.json"]);
    assert_eq!(SidecarFiles.stored_edit(&photo), Ok(Some(brighter())));
}

#[test]
fn zone_mask_is_stored_with_its_coverage() {
    let folder = tempfile::tempdir().unwrap();
    let photo = photo_in(&folder);
    let values = (0..64 * 48).map(|texel| (texel % 256) as u8).collect();
    let sky = MaskShape::Zone(ZoneMask {
        zone: Zone::Sky,
        coverage: Arc::new(CoverageImage::new([64, 48], values).unwrap()),
    });
    let edit = Edit {
        masks: vec![Mask::of(sky)],
        ..brighter()
    };

    SidecarFiles.store_edit(&photo, &edit).unwrap();

    assert_eq!(files_of(folder.path()), ["DSC07070.ARW.ziv.json"]);
    assert_eq!(SidecarFiles.stored_edit(&photo), Ok(Some(edit)));
}

#[test]
fn tone_curve_color_mixer_and_color_grading_are_stored_with_the_edit() {
    let folder = tempfile::tempdir().unwrap();
    let photo = photo_in(&folder);
    let mut edit = brighter();
    edit.tone_curves.blue = ToneCurve::try_from(vec![[0.0, 0.15], [1.0, 1.0]]).unwrap();
    edit.color_mixer.saturation[ColorRange::Green] = -40.0;
    edit.color_grading.shadows = ZoneGrade {
        hue: 185.0,
        saturation: 30.0,
        luminance: -5.0,
    };
    edit.color_grading.balance = 20.0;

    SidecarFiles.store_edit(&photo, &edit).unwrap();

    assert_eq!(SidecarFiles.stored_edit(&photo), Ok(Some(edit)));
}

#[test]
fn photo_without_sidecar_has_no_stored_edit() {
    let folder = tempfile::tempdir().unwrap();

    assert_eq!(SidecarFiles.stored_edit(&photo_in(&folder)), Ok(None));
}

#[test]
fn storing_a_default_edit_removes_the_sidecar() {
    let folder = tempfile::tempdir().unwrap();
    let photo = photo_in(&folder);
    SidecarFiles.store_edit(&photo, &brighter()).unwrap();

    SidecarFiles.store_edit(&photo, &Edit::default()).unwrap();
    SidecarFiles.store_edit(&photo, &Edit::default()).unwrap();

    assert!(files_of(folder.path()).is_empty());
}

#[test]
fn unreadable_sidecar_is_reported_with_its_reason() {
    let folder = tempfile::tempdir().unwrap();
    let photo = photo_in(&folder);
    fs::write(sidecar_path(&photo), r#"{ "version": 99 }"#).unwrap();

    let reason = SidecarFiles.stored_edit(&photo).unwrap_err();

    assert!(reason.contains("newer ziv"), "{reason}");
}

#[test]
fn folder_that_cannot_be_written_reports_the_failure() {
    let folder = tempfile::tempdir().unwrap();
    let missing_folder = folder.path().join("gone").join("DSC07070.ARW");

    assert!(
        SidecarFiles
            .store_edit(&missing_folder, &brighter())
            .is_err()
    );
}

#[test]
fn a_photo_whose_only_change_is_its_framing_has_a_stored_edit() {
    let folder = tempfile::tempdir().unwrap();
    let photo = photo_in(&folder);
    let turned = Edit {
        framing: Framing {
            turn: Turn::default().turned_right(),
            ..Framing::default()
        },
        ..Edit::default()
    };

    SidecarFiles.store_edit(&photo, &turned).unwrap();

    assert_eq!(SidecarFiles.stored_edit(&photo), Ok(Some(turned)));
}
