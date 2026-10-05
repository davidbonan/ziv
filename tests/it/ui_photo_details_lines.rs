use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use ziv::photo::domain::photo_details::{PhotoDetails, ShotAt};
use ziv::photo::domain::shooting_data::ShootingData;
use ziv::photo::ui::photo_details_lines::{PhotoDetailsShown, photo_details_lines};

use crate::themed::is_themed;

fn lines_harness(shooting_data: ShootingData, details: PhotoDetails) -> Harness<'static> {
    lines_harness_for(shooting_data, details, 1)
}

fn lines_harness_for(
    shooting_data: ShootingData,
    details: PhotoDetails,
    selection_size: usize,
) -> Harness<'static> {
    let mut harness = Harness::builder()
        .with_size(vec2(600.0, 120.0))
        .build_ui(move |ui| {
            if !is_themed(ui) {
                return;
            }
            let shown = PhotoDetailsShown {
                name: "DSC07070.ARW",
                shooting_data: &shooting_data,
                details: &details,
                selection_size,
            };
            photo_details_lines(ui, &shown);
        });
    harness.run();
    harness
}

#[test]
fn lines_name_the_photo_then_say_how_it_was_shot_and_what_its_file_is() {
    let shot = ShootingData {
        iso: Some(125),
        aperture: Some(2.8),
        ..ShootingData::default()
    };
    let details = PhotoDetails {
        pixel_size: Some([7008, 4672]),
        shot_at: ShotAt::from_exif("2026:09:14 18:42:07"),
        file_bytes: Some(34_120_000),
    };

    let harness = lines_harness(shot, details);

    let top_of = |label| harness.get_by_label(label).rect().top();
    assert!(top_of("DSC07070.ARW") < top_of("ISO 125"));
    assert!(top_of("ISO 125") < top_of("7008 × 4672"));
    for label in ["f/2.8", "14 Sep 2026, 18:42", "34.1 MB"] {
        assert!(harness.query_by_label(label).is_some(), "{label}");
    }
}

#[test]
fn a_photo_whose_file_says_nothing_keeps_its_name() {
    let harness = lines_harness(ShootingData::default(), PhotoDetails::default());

    assert!(harness.query_by_label("DSC07070.ARW").is_some());
}

#[test]
fn a_selection_of_several_photos_is_counted_and_one_photo_alone_is_not() {
    let several = lines_harness_for(ShootingData::default(), PhotoDetails::default(), 5);
    let alone = lines_harness(ShootingData::default(), PhotoDetails::default());

    assert!(several.query_by_label("5 selected").is_some());
    assert!(alone.query_by_label_contains("selected").is_none());
}
