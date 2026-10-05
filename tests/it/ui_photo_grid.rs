use std::path::Path;

use crate::themed::is_themed;
use egui::accesskit::Toggled;
use egui::{Key, Modifiers, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use ziv::library::domain::mark::{Mark, Rating};
use ziv::library::ui::photo_grid::{
    GridIntent, GridShown, MOVE_TO_TRASH_LABEL, REMOVE_FROM_SERIES_LABEL, TRASH_REJECTED_LABEL,
    grid_header, photo_grid,
};
use ziv::library::ui::photo_thumbnail::{PhotoThumbnail, ThumbnailState};

const NAMES: [&str; 7] = [
    "1.ARW", "2.ARW", "3.ARW", "4.ARW", "5.ARW", "6.ARW", "7.ARW",
];
/// Room for three cells in a row, not for four.
const GRID_WIDTH: f32 = 440.0;

// Two clicks one frame apart must stay within egui's double-click delay.
const DOUBLE_CLICKABLE_STEP_SECONDS: f32 = 0.1;

const MARKED: Mark = Mark {
    rating: Rating::of(3),
    is_rejected: true,
};

struct Grid {
    selected: Option<usize>,
    is_in_selection: [bool; NAMES.len()],
    intent: Option<GridIntent>,
}

fn grid_harness(selected: usize) -> Harness<'static, Grid> {
    let grid = Grid {
        selected: Some(selected),
        is_in_selection: [false; NAMES.len()],
        intent: None,
    };
    let mut harness = Harness::builder()
        .with_step_dt(DOUBLE_CLICKABLE_STEP_SECONDS)
        .with_size(vec2(GRID_WIDTH, 400.0))
        .build_ui_state(
            |ui, grid: &mut Grid| {
                if !is_themed(ui) {
                    return;
                }
                let photos: Vec<PhotoThumbnail<'_>> = NAMES
                    .iter()
                    .map(|name| PhotoThumbnail {
                        path: Path::new(name),
                        thumbnail: &ThumbnailState::Loading,
                        is_edited: *name == "2.ARW",
                        mark: match *name {
                            "7.ARW" => MARKED,
                            _ => Mark::default(),
                        },
                    })
                    .collect();
                let shown = GridShown {
                    photos: &photos,
                    selected: grid.selected,
                    is_in_selection: &grid.is_in_selection,
                };
                if let Some(intent) = photo_grid(ui, &shown) {
                    grid.intent = Some(intent);
                }
            },
            grid,
        );
    harness.run();
    harness
}

fn asked_by_pressing(key: Key, selected: usize) -> Option<GridIntent> {
    let mut harness = grid_harness(selected);
    harness.key_press(key);
    harness.run();
    harness.state().intent
}

#[test]
fn photos_fill_rows_in_series_order() {
    let harness = grid_harness(0);

    let cell = |label| harness.get_by_label(label).rect();
    let (first, third, fourth) = (
        cell("1.ARW, loading"),
        cell("3.ARW, loading"),
        cell("4.ARW, loading"),
    );
    assert!(first.left() < third.left());
    assert_eq!(first.top(), third.top());
    assert_eq!(first.left(), fourth.left());
    assert!(first.top() < fourth.top());
    assert!(harness.query_by_label("2.ARW, loading, edited").is_some());
}

#[test]
fn clicking_a_cell_asks_to_select_it() {
    let mut harness = grid_harness(0);

    harness.get_by_label("6.ARW, loading").click();
    harness.run();

    assert_eq!(harness.state().intent, Some(GridIntent::Select(5)));
}

#[test]
fn arrow_keys_ask_for_the_neighbours_in_the_row_and_in_the_column() {
    assert_eq!(
        asked_by_pressing(Key::ArrowRight, 3),
        Some(GridIntent::Select(4))
    );
    assert_eq!(
        asked_by_pressing(Key::ArrowLeft, 3),
        Some(GridIntent::Select(2))
    );
    assert_eq!(
        asked_by_pressing(Key::ArrowDown, 3),
        Some(GridIntent::Select(6))
    );
    assert_eq!(
        asked_by_pressing(Key::ArrowUp, 3),
        Some(GridIntent::Select(0))
    );
}

#[test]
fn arrow_keys_stop_where_the_grid_ends() {
    assert_eq!(
        asked_by_pressing(Key::ArrowDown, 4),
        Some(GridIntent::Select(4))
    );
    assert_eq!(
        asked_by_pressing(Key::ArrowLeft, 0),
        Some(GridIntent::Select(0))
    );
}

fn asked_by_clicking_with(modifiers: Modifiers) -> Option<GridIntent> {
    let mut harness = grid_harness(0);
    harness
        .get_by_label("6.ARW, loading")
        .click_modifiers(modifiers);
    harness.run();
    harness.state().intent
}

#[test]
fn a_click_with_cmd_toggles_and_one_with_shift_extends() {
    assert_eq!(
        asked_by_clicking_with(Modifiers::COMMAND),
        Some(GridIntent::Toggle(5))
    );
    assert_eq!(
        asked_by_clicking_with(Modifiers::SHIFT),
        Some(GridIntent::ExtendTo(5))
    );
}

#[test]
fn cmd_a_asks_for_every_photo_and_enter_for_developing_the_selected_one() {
    let mut harness = grid_harness(3);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::A);
    harness.run();
    assert_eq!(harness.state().intent, Some(GridIntent::SelectAll));

    assert_eq!(asked_by_pressing(Key::Enter, 3), Some(GridIntent::Open(3)));
}

#[test]
fn a_double_click_asks_for_developing_that_photo() {
    let mut harness = grid_harness(0);

    let cell = harness.get_by_label("6.ARW, loading").rect().center();
    harness.hover_at(cell);
    harness.run();
    for pressed in [true, false, true, false] {
        harness.event(egui::Event::PointerButton {
            pos: cell,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
        harness.step();
    }

    assert_eq!(harness.state().intent, Some(GridIntent::Open(5)));
}

#[test]
fn every_photo_of_the_selection_shows_as_selected() {
    let mut harness = grid_harness(0);
    harness.state_mut().is_in_selection[..3].fill(true);
    harness.run();

    let is_on = |label| harness.get_by_label(label).accesskit_node().toggled();
    assert_eq!(is_on("1.ARW, loading"), Some(Toggled::True));
    assert_eq!(is_on("3.ARW, loading"), Some(Toggled::True));
    assert_eq!(is_on("4.ARW, loading"), Some(Toggled::False));
}

#[test]
fn a_cell_tells_the_marks_of_its_photo() {
    let harness = grid_harness(0);

    assert!(
        harness
            .query_by_label("7.ARW, loading, 3 stars, rejected")
            .is_some()
    );
}

#[test]
fn delete_asks_to_remove_the_selection() {
    for key in [Key::Backspace, Key::Delete] {
        assert_eq!(
            asked_by_pressing(key, 3),
            Some(GridIntent::RemoveSelection),
            "{key:?}"
        );
    }
}

fn asked_by_the_menu_of_the_sixth_cell(label: &str) -> Option<GridIntent> {
    let mut harness = grid_harness(0);
    harness.get_by_label("6.ARW, loading").click_secondary();
    harness.run();
    harness.get_by_label(label).click();
    harness.run();
    harness.state().intent
}

#[test]
fn menu_of_a_cell_asks_to_remove_from_the_series_or_to_move_to_the_trash() {
    assert_eq!(
        asked_by_the_menu_of_the_sixth_cell(REMOVE_FROM_SERIES_LABEL),
        Some(GridIntent::RemoveFromSeries(5))
    );
    assert_eq!(
        asked_by_the_menu_of_the_sixth_cell(MOVE_TO_TRASH_LABEL),
        Some(GridIntent::MoveToTrash(5))
    );
}

#[test]
fn cmd_delete_asks_to_move_the_selection_to_the_trash() {
    let mut harness = grid_harness(3);

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Backspace);
    harness.run();

    assert_eq!(harness.state().intent, Some(GridIntent::TrashSelection));
}

fn header_harness(has_rejected_photos: bool) -> Harness<'static, bool> {
    let mut harness = Harness::builder()
        .with_size(vec2(GRID_WIDTH, 60.0))
        .build_ui_state(
            move |ui, is_asked: &mut bool| {
                if is_themed(ui) {
                    *is_asked |= grid_header(ui, has_rejected_photos);
                }
            },
            false,
        );
    harness.run();
    harness
}

#[test]
fn header_asks_to_trash_the_rejected_photos_when_there_are_some() {
    let mut harness = header_harness(true);
    harness.get_by_label(TRASH_REJECTED_LABEL).click();
    harness.run();
    assert!(*harness.state());

    let without = header_harness(false);
    let button = without.get_by_label(TRASH_REJECTED_LABEL);
    assert!(button.accesskit_node().is_disabled());
}
