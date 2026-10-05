use std::path::Path;

use crate::themed::is_themed;
use egui::{Key, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use ziv::library::domain::mark::{Mark, Rating};
use ziv::library::ui::filmstrip::{FILMSTRIP_HEIGHT, FilmstripIntent, filmstrip};
use ziv::library::ui::photo_thumbnail::{PhotoThumbnail, ThumbnailState};

const OTHER_CONTROL: &str = "Another control";
const EDITED: &str = "DSC1.ARW";
const NAMES: [&str; 3] = [EDITED, "DSC2.ARW", "DSC10.ARW"];

struct Strip {
    thumbnails: [ThumbnailState; 3],
    selected: Option<usize>,
    mark_of_all: Mark,
    intent: Option<FilmstripIntent>,
    other_control: f32,
}

fn strip_harness(thumbnails: [ThumbnailState; 3]) -> Harness<'static, Strip> {
    let strip = Strip {
        thumbnails,
        selected: Some(0),
        mark_of_all: Mark::default(),
        intent: None,
        other_control: 0.0,
    };
    let mut harness = Harness::builder()
        .with_size(vec2(600.0, FILMSTRIP_HEIGHT + 48.0))
        .build_ui_state(
            |ui, strip: &mut Strip| {
                if !is_themed(ui) {
                    return;
                }
                let photos: Vec<PhotoThumbnail<'_>> = NAMES
                    .iter()
                    .zip(&strip.thumbnails)
                    .map(|(name, thumbnail)| PhotoThumbnail {
                        path: Path::new(name),
                        thumbnail,
                        is_edited: *name == EDITED,
                        mark: strip.mark_of_all,
                    })
                    .collect();
                ui.add(egui::Slider::new(&mut strip.other_control, 0.0..=1.0).text(OTHER_CONTROL));
                if let Some(intent) = filmstrip(ui, &photos, strip.selected) {
                    strip.intent = Some(intent);
                }
            },
            strip,
        );
    harness.run();
    harness
}

fn all_loading() -> [ThumbnailState; 3] {
    [
        ThumbnailState::Loading,
        ThumbnailState::Loading,
        ThumbnailState::Loading,
    ]
}

#[test]
fn every_photo_has_its_place_in_session_order() {
    let harness = strip_harness(all_loading());

    let labels = [
        "DSC1.ARW, loading, edited",
        "DSC2.ARW, loading",
        "DSC10.ARW, loading",
    ];
    let positions = labels.map(|label| harness.get_by_label(label).rect().left());
    assert!(positions[0] < positions[1] && positions[1] < positions[2]);
}

#[test]
fn failed_photo_keeps_its_place_and_is_marked() {
    let harness = strip_harness([
        ThumbnailState::Loading,
        ThumbnailState::Failed,
        ThumbnailState::Loading,
    ]);

    assert!(
        harness
            .query_by_label("DSC2.ARW, could not be opened")
            .is_some()
    );
}

#[test]
fn missing_photo_keeps_its_place_and_is_marked() {
    let harness = strip_harness([
        ThumbnailState::Loading,
        ThumbnailState::NotFound,
        ThumbnailState::Loading,
    ]);

    assert!(harness.query_by_label("DSC2.ARW, not found").is_some());
}

#[test]
fn ready_thumbnail_is_named_after_its_photo() {
    let mut harness = strip_harness(all_loading());
    let texture =
        harness
            .ctx
            .load_texture("thumbnail", egui::ColorImage::example(), Default::default());
    harness.state_mut().thumbnails[2] = ThumbnailState::Ready(texture);
    harness.run();

    assert!(harness.query_by_label("DSC10.ARW").is_some());
}

#[test]
fn clicking_a_thumbnail_asks_to_select_it() {
    let mut harness = strip_harness(all_loading());

    harness.get_by_label("DSC10.ARW, loading").click();
    harness.run();

    assert_eq!(harness.state().intent, Some(FilmstripIntent::Select(2)));
}

#[test]
fn arrow_keys_ask_for_the_neighbouring_photos() {
    let mut harness = strip_harness(all_loading());

    harness.key_press(Key::ArrowRight);
    harness.run();
    assert_eq!(harness.state().intent, Some(FilmstripIntent::SelectNext));

    harness.key_press(Key::ArrowLeft);
    harness.run();
    assert_eq!(
        harness.state().intent,
        Some(FilmstripIntent::SelectPrevious)
    );
}

#[test]
fn strip_starts_with_the_position_of_the_selected_photo() {
    let mut harness = strip_harness(all_loading());
    assert!(harness.query_by_label("1 / 3").is_some());

    harness.state_mut().selected = Some(2);
    harness.run();

    assert!(harness.query_by_label("3 / 3").is_some());
}

#[test]
fn strip_holds_the_photos_and_no_button_of_its_own() {
    let harness = strip_harness(all_loading());

    let buttons = harness.query_all_by_role(egui::accesskit::Role::Button);
    assert_eq!(buttons.count(), NAMES.len());
}

#[test]
fn arrow_keys_are_left_to_a_focused_control() {
    let mut harness = strip_harness(all_loading());

    harness.get_by_role(egui::accesskit::Role::Slider).focus();
    harness.run();
    harness.key_press(Key::ArrowRight);
    harness.run();

    assert_eq!(harness.state().intent, None);
    assert!(harness.state().other_control > 0.0);
}

#[test]
fn a_thumbnail_tells_the_marks_of_its_photo() {
    let mut harness = strip_harness(all_loading());

    harness.state_mut().mark_of_all = Mark {
        rating: Rating::of(1),
        is_rejected: false,
    };
    harness.run();

    assert!(
        harness
            .query_by_label("DSC2.ARW, loading, 1 star")
            .is_some()
    );
}
