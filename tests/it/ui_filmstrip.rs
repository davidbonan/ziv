use std::path::Path;

use crate::themed::is_themed;
use egui::{Key, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use ziv::library::ui::empty_state::OPEN_BUTTON_LABEL;
use ziv::library::ui::filmstrip::{
    EXPORT_BUTTON_LABEL, FILMSTRIP_HEIGHT, FilmstripIntent, FilmstripPhoto, ThumbnailState,
    filmstrip,
};

const OTHER_CONTROL: &str = "Another control";
const NAMES: [&str; 3] = ["DSC1.ARW", "DSC2.ARW", "DSC10.ARW"];

struct Strip {
    thumbnails: [ThumbnailState; 3],
    selected: Option<usize>,
    intent: Option<FilmstripIntent>,
    other_control: f32,
}

fn strip_harness(thumbnails: [ThumbnailState; 3]) -> Harness<'static, Strip> {
    let strip = Strip {
        thumbnails,
        selected: Some(0),
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
                let photos: Vec<FilmstripPhoto<'_>> = NAMES
                    .iter()
                    .zip(&strip.thumbnails)
                    .map(|(name, thumbnail)| FilmstripPhoto {
                        path: Path::new(name),
                        thumbnail,
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

    let positions = NAMES.map(|name| {
        harness
            .get_by_label(&format!("{name}, loading"))
            .rect()
            .left()
    });
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
fn open_button_asks_to_open_photos() {
    let mut harness = strip_harness(all_loading());

    harness.get_by_label(OPEN_BUTTON_LABEL).click();
    harness.run();

    assert_eq!(harness.state().intent, Some(FilmstripIntent::Open));
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
fn export_button_asks_to_export() {
    let mut harness = strip_harness(all_loading());

    harness.get_by_label(EXPORT_BUTTON_LABEL).click();
    harness.run();

    assert_eq!(harness.state().intent, Some(FilmstripIntent::Export));
}
