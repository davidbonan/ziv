use crate::themed::is_themed;
use egui::{Key, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use ziv::library::domain::import_day::ImportDay;
use ziv::library::ui::series_sidebar::{
    IMPORT_BUTTON_LABEL, LOCATE_LABEL, REMOVE_LABEL, RENAME_LABEL, SERIES_NAME_FIELD_LABEL,
    SERIES_SIDEBAR_WIDTH, SHOW_IN_FINDER_LABEL, SeriesRow, SeriesShown, SidebarIntent,
    series_sidebar, unusable_catalog_warning,
};

const MISSING: &str = "Portraits";
const SERIES: [(&str, usize, usize, u8); 4] = [
    ("Lofoten", 37, 148, 28),
    ("Mariage", 0, 412, 21),
    ("23 photos, 14 Sep", 23, 23, 14),
    (MISSING, 12, 64, 2),
];
const PORTRAITS: &str = "Portraits, Folder not found";
const LOFOTEN: &str = "Lofoten, 37 of 148 edited, 28 Sep";
const MARIAGE: &str = "Mariage, 0 of 412 edited, 21 Sep";
const PHOTO_SET: &str = "23 photos, 14 Sep, 23 of 23 edited, 14 Sep";

struct Sidebar {
    open: Option<usize>,
    warning: Option<&'static str>,
    intent: Option<SidebarIntent>,
}

fn sidebar_harness() -> Harness<'static, Sidebar> {
    let sidebar = Sidebar {
        open: Some(0),
        warning: None,
        intent: None,
    };
    let mut harness = Harness::builder()
        .with_size(vec2(SERIES_SIDEBAR_WIDTH + 200.0, 400.0))
        .build_ui_state(
            |ui, sidebar: &mut Sidebar| {
                if !is_themed(ui) {
                    return;
                }
                let rows = SERIES.map(|(name, edited_count, photo_count, day)| SeriesRow {
                    name,
                    photo_count,
                    edited_count,
                    imported_on: ImportDay {
                        year: 2026,
                        month: 9,
                        day,
                    },
                    cover: [None; 4],
                    is_missing: name == MISSING,
                });
                let shown = SeriesShown {
                    rows: &rows,
                    open: sidebar.open,
                    warning: sidebar.warning,
                };
                if let Some(intent) = series_sidebar(ui, &shown) {
                    sidebar.intent = Some(intent);
                }
            },
            sidebar,
        );
    harness.run();
    harness
}

#[test]
fn every_series_has_its_row_in_catalog_order() {
    let harness = sidebar_harness();

    let tops = [LOFOTEN, MARIAGE, PHOTO_SET].map(|label| harness.get_by_label(label).rect().top());
    assert!(tops[0] < tops[1] && tops[1] < tops[2]);
}

#[test]
fn clicking_a_row_asks_to_open_its_series() {
    let mut harness = sidebar_harness();

    harness.get_by_label(MARIAGE).click();
    harness.run();

    assert_eq!(harness.state().intent, Some(SidebarIntent::Open(1)));
}

#[test]
fn import_button_asks_to_import_photos() {
    let mut harness = sidebar_harness();

    harness.get_by_label(IMPORT_BUTTON_LABEL).click();
    harness.run();

    assert_eq!(harness.state().intent, Some(SidebarIntent::Import));
}

#[test]
fn missing_series_says_its_folder_is_not_found() {
    let harness = sidebar_harness();

    assert!(harness.query_by_label(PORTRAITS).is_some());
}

fn menu_of(row: &str) -> Harness<'static, Sidebar> {
    let mut harness = sidebar_harness();
    harness.get_by_label(row).click_secondary();
    harness.run();
    harness
}

#[test]
fn menu_of_a_series_removes_it_and_shows_it_in_the_finder() {
    let mut harness = menu_of(MARIAGE);
    assert!(harness.query_by_label(LOCATE_LABEL).is_none());
    harness.get_by_label(SHOW_IN_FINDER_LABEL).click();
    harness.run();
    assert_eq!(harness.state().intent, Some(SidebarIntent::ShowInFinder(1)));

    let mut harness = menu_of(MARIAGE);
    harness.get_by_label(REMOVE_LABEL).click();
    harness.run();
    assert_eq!(harness.state().intent, Some(SidebarIntent::Remove(1)));
}

#[test]
fn menu_of_a_missing_series_locates_it_and_cannot_show_it() {
    let mut harness = menu_of(PORTRAITS);

    assert!(
        harness
            .get_by_label(SHOW_IN_FINDER_LABEL)
            .accesskit_node()
            .is_disabled()
    );
    harness.get_by_label(LOCATE_LABEL).click();
    harness.run();

    assert_eq!(harness.state().intent, Some(SidebarIntent::Locate(3)));
}

fn renaming(row: &str) -> Harness<'static, Sidebar> {
    let mut harness = menu_of(row);
    harness.get_by_label(RENAME_LABEL).click();
    harness.run();
    harness
}

#[test]
fn typed_name_is_confirmed_by_enter() {
    let mut harness = renaming(MARIAGE);

    harness
        .get_by_label(SERIES_NAME_FIELD_LABEL)
        .type_text(" 2026");
    harness.run();
    harness.key_press(Key::Enter);
    harness.run();

    let renamed = SidebarIntent::Rename {
        index: 1,
        name: "Mariage 2026".to_owned(),
    };
    assert_eq!(harness.state().intent, Some(renamed));
    assert!(harness.query_by_label(SERIES_NAME_FIELD_LABEL).is_none());
}

#[test]
fn escape_leaves_the_name_as_it_was() {
    let mut harness = renaming(MARIAGE);

    harness
        .get_by_label(SERIES_NAME_FIELD_LABEL)
        .type_text(" 2026");
    harness.run();
    harness.key_press(Key::Escape);
    harness.run();

    assert_eq!(harness.state().intent, None);
    assert!(harness.query_by_label(MARIAGE).is_some());
}

#[test]
fn catalog_that_cannot_be_used_is_told_with_its_file_and_what_follows() {
    let warning = unusable_catalog_warning(
        Some(std::path::Path::new("/data/ziv/catalog.json")),
        "it was written by a newer ziv",
    );
    let mut harness = sidebar_harness();
    harness.state_mut().warning = Some(warning.clone().leak());
    harness.run();

    assert!(warning.starts_with("/data/ziv/catalog.json cannot be used"));
    assert!(warning.ends_with("will not be remembered."));
    assert!(harness.query_by_label(&warning).is_some());
}
