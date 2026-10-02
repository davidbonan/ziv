use std::path::Path;

use crate::themed::is_themed;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use ziv::photo::domain::decode_error::DecodeError;
use ziv::viewport::ui::photo_status::{failed_label, loading_label, photo_failed, photo_loading};

const PHOTO: &str = "/photos/DSC07070.ARW";

#[test]
fn loading_photo_is_announced_by_its_file_name() {
    let mut harness = Harness::new_ui(|ui| {
        if is_themed(ui) {
            photo_loading(ui, Path::new(PHOTO));
        }
    });
    harness.step();
    harness.step();

    assert_eq!(loading_label(Path::new(PHOTO)), "Loading DSC07070.ARW…");
    assert!(harness.query_by_label("Loading DSC07070.ARW…").is_some());
}

#[test]
fn failed_photo_shows_its_file_name_and_the_reason() {
    let error = DecodeError::new("corrupt file");
    let mut harness = Harness::new_ui(|ui| {
        if is_themed(ui) {
            photo_failed(ui, Path::new(PHOTO), &error);
        }
    });
    harness.run();

    assert!(
        harness
            .query_by_label(&failed_label(Path::new(PHOTO)))
            .is_some()
    );
    assert!(harness.query_by_label("corrupt file").is_some());
}
