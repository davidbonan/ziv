use crate::themed::is_themed;
use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use ziv::shell::ui::top_bar::{
    BEFORE_LABEL, EXPORT_BUTTON_LABEL, SIDEBAR_TOGGLE_LABEL, TOP_BAR_HEIGHT, TopBarIntent,
    TopBarShown, top_bar,
};

struct Bar {
    shown: TopBarShown<'static>,
    intent: Option<TopBarIntent>,
}

const ON_A_READY_PHOTO: TopBarShown<'static> = TopBarShown {
    is_sidebar_shown: true,
    series_name: Some("Lofoten"),
    photo_name: Some("DSC07070.ARW"),
    zoom_readout: Some("Fit · 24 %"),
    is_before_shown: false,
    can_before_be_shown: true,
    can_export: true,
};

const WITHOUT_SERIES: TopBarShown<'static> = TopBarShown {
    is_sidebar_shown: true,
    series_name: None,
    photo_name: None,
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
fn without_series_before_and_export_wait_and_nothing_is_named() {
    let harness = bar_harness(WITHOUT_SERIES);

    for label in [BEFORE_LABEL, EXPORT_BUTTON_LABEL] {
        let button = harness.get_by_label(label);
        assert!(button.accesskit_node().is_disabled(), "{label}");
    }
    assert!(harness.query_by_label("/").is_none());
}
