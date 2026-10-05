use crate::themed::is_themed;
use egui::accesskit::Toggled;
use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use ziv::library::domain::mark::Rating;
use ziv::library::domain::series_filter::SeriesFilter;
use ziv::library::ui::series_filter_switch::ALL_PHOTOS_LABEL;
use ziv::shell::domain::window_mode::WindowMode;
use ziv::shell::ui::top_bar::{
    BEFORE_LABEL, CULL_LABEL, DEVELOP_LABEL, EXPORT_BUTTON_LABEL, SIDEBAR_TOGGLE_LABEL,
    TOP_BAR_HEIGHT, TopBarIntent, TopBarShown, top_bar,
};

struct Bar {
    shown: TopBarShown<'static>,
    intent: Option<TopBarIntent>,
}

const TWO_STARS_OR_MORE: SeriesFilter = SeriesFilter::AtLeast(Rating::of(2));

const ON_A_READY_PHOTO: TopBarShown<'static> = TopBarShown {
    is_sidebar_shown: true,
    series_name: Some("Lofoten"),
    photo_name: Some("DSC07070.ARW"),
    mode: WindowMode::Develop,
    filter: Some(TWO_STARS_OR_MORE),
    zoom_readout: Some("Fit · 24 %"),
    is_before_shown: false,
    can_before_be_shown: true,
    can_export: true,
};

const WITHOUT_SERIES: TopBarShown<'static> = TopBarShown {
    is_sidebar_shown: true,
    series_name: None,
    photo_name: None,
    mode: WindowMode::Develop,
    filter: None,
    zoom_readout: None,
    is_before_shown: false,
    can_before_be_shown: false,
    can_export: false,
};

fn bar_harness(shown: TopBarShown<'static>) -> Harness<'static, Bar> {
    let bar = Bar {
        shown,
        intent: None,
    };
    let mut harness = Harness::builder()
        .with_size(vec2(900.0, TOP_BAR_HEIGHT))
        .build_ui_state(
            |ui, bar: &mut Bar| {
                if !is_themed(ui) {
                    return;
                }
                if let Some(intent) = top_bar(ui, &bar.shown) {
                    bar.intent = Some(intent);
                }
            },
            bar,
        );
    harness.run();
    harness
}

fn asked_by_clicking(label: &str) -> Option<TopBarIntent> {
    let mut harness = bar_harness(ON_A_READY_PHOTO);
    harness.get_by_label(label).click();
    harness.run();
    harness.state().intent
}

#[test]
fn bar_names_the_series_and_the_photo_and_says_the_zoom() {
    let harness = bar_harness(ON_A_READY_PHOTO);

    for label in ["Lofoten", "DSC07070.ARW", "Fit · 24 %"] {
        assert!(harness.query_by_label(label).is_some(), "{label}");
    }
}

#[test]
fn each_button_asks_for_what_it_says() {
    assert_eq!(
        asked_by_clicking(SIDEBAR_TOGGLE_LABEL),
        Some(TopBarIntent::ToggleSidebar)
    );
    assert_eq!(
        asked_by_clicking(BEFORE_LABEL),
        Some(TopBarIntent::ToggleBefore)
    );
    assert_eq!(
        asked_by_clicking(EXPORT_BUTTON_LABEL),
        Some(TopBarIntent::Export)
    );
}

#[test]
fn mode_switch_asks_for_the_mode_clicked_and_shows_the_current_one_as_on() {
    assert_eq!(
        asked_by_clicking(CULL_LABEL),
        Some(TopBarIntent::SwitchTo(WindowMode::Cull))
    );
    assert_eq!(
        asked_by_clicking(DEVELOP_LABEL),
        Some(TopBarIntent::SwitchTo(WindowMode::Develop))
    );

    let harness = bar_harness(ON_A_READY_PHOTO);
    let is_on = |label| harness.get_by_label(label).accesskit_node().toggled();
    assert_eq!(is_on(DEVELOP_LABEL), Some(Toggled::True));
    assert_eq!(is_on(CULL_LABEL), Some(Toggled::False));
}

#[test]
fn without_series_before_and_export_wait_and_nothing_is_named() {
    let harness = bar_harness(WITHOUT_SERIES);

    for label in [BEFORE_LABEL, EXPORT_BUTTON_LABEL] {
        let button = harness.get_by_label(label);
        assert!(button.accesskit_node().is_disabled(), "{label}");
    }
    assert!(harness.query_by_label("/").is_none());
}

#[test]
fn filter_shows_its_stars_and_asks_for_the_one_clicked() {
    let harness = bar_harness(ON_A_READY_PHOTO);
    let is_on = |label| harness.get_by_label(label).accesskit_node().toggled();
    assert_eq!(is_on("2 stars or more"), Some(Toggled::True));
    assert_eq!(is_on("3 stars or more"), Some(Toggled::False));
    assert_eq!(is_on(ALL_PHOTOS_LABEL), Some(Toggled::False));

    assert_eq!(
        asked_by_clicking("4 stars or more"),
        Some(TopBarIntent::Filter(SeriesFilter::at_least(4)))
    );
    assert_eq!(
        asked_by_clicking(ALL_PHOTOS_LABEL),
        Some(TopBarIntent::Filter(SeriesFilter::All))
    );
}

#[test]
fn without_series_there_is_no_filter() {
    let harness = bar_harness(WITHOUT_SERIES);

    assert!(harness.query_by_label(ALL_PHOTOS_LABEL).is_none());
}
