use std::path::PathBuf;

use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use ziv::export::domain::export_settings::{ExportFormat, ExportSettings, ExportSize};
use ziv::export::ui::export_dialog::{
    CANCEL_LABEL, CHANGE_DESTINATION_LABEL, CHOOSE_DESTINATION_LABEL, EXPORT_LABEL,
    ExportDialogIntent, LONG_EDGE_CHOICE_LABEL, LONG_EDGE_LABEL, NO_DESTINATION_LABEL, PNG_LABEL,
    QUALITY_LABEL, export_dialog,
};

use crate::themed::is_themed;

#[derive(Default)]
struct Dialog {
    settings: ExportSettings,
    intent: Option<ExportDialogIntent>,
}

fn dialog_harness(photos: &[&str], settings: ExportSettings) -> Harness<'static, Dialog> {
    let photos: Vec<PathBuf> = photos.iter().map(PathBuf::from).collect();
    let mut harness = Harness::builder()
        .with_size(egui::vec2(400.0, 420.0))
        .build_ui_state(
            move |ui, dialog: &mut Dialog| {
                if is_themed(ui) {
                    let output = export_dialog(ui, &photos, &dialog.settings);
                    dialog.settings = output.settings;
                    dialog.intent = output.intent.or(dialog.intent);
                }
            },
            Dialog {
                settings,
                intent: None,
            },
        );
    harness.run();
    harness
}

fn with_destination() -> ExportSettings {
    ExportSettings {
        destination: Some(PathBuf::from("/Users/me/Exports")),
        ..ExportSettings::default()
    }
}

fn button<'a>(harness: &'a Harness<'static, Dialog>, label: &'a str) -> egui_kittest::Node<'a> {
    harness.get_by_role_and_label(Role::Button, label)
}

fn is_slider_disabled(harness: &Harness<'static, Dialog>, label: &str) -> bool {
    harness
        .get_by_role_and_label(Role::Slider, label)
        .accesskit_node()
        .is_disabled()
}

#[test]
fn dialog_names_the_photo_or_counts_the_photos() {
    let one = dialog_harness(&["shoot/DSC07070.ARW"], with_destination());
    let several = dialog_harness(&["a.arw", "b.arw", "c.jpg"], with_destination());

    assert!(one.query_by_label("DSC07070.ARW").is_some());
    assert!(several.query_by_label("3 photos").is_some());
}

#[test]
fn export_waits_for_a_destination() {
    let without = dialog_harness(&["a.arw"], ExportSettings::default());
    let with = dialog_harness(&["a.arw"], with_destination());

    assert!(without.query_by_label(NO_DESTINATION_LABEL).is_some());
    assert!(
        button(&without, EXPORT_LABEL)
            .accesskit_node()
            .is_disabled()
    );
    assert!(with.query_by_label("/Users/me/Exports").is_some());
    assert!(!button(&with, EXPORT_LABEL).accesskit_node().is_disabled());
}

#[test]
fn deep_destination_is_shown_by_its_last_two_folders() {
    let settings = ExportSettings {
        destination: Some(PathBuf::from("/Users/me/Pictures/2026/Exports")),
        ..ExportSettings::default()
    };
    let harness = dialog_harness(&["a.arw"], settings);

    assert!(harness.query_by_label("…/2026/Exports").is_some());
}

#[test]
fn each_button_asks_for_what_it_says() {
    let cases = [
        (
            ExportSettings::default(),
            CHOOSE_DESTINATION_LABEL,
            ExportDialogIntent::ChooseDestination,
        ),
        (
            with_destination(),
            CHANGE_DESTINATION_LABEL,
            ExportDialogIntent::ChooseDestination,
        ),
        (with_destination(), CANCEL_LABEL, ExportDialogIntent::Cancel),
        (with_destination(), EXPORT_LABEL, ExportDialogIntent::Export),
    ];
    for (settings, label, intent) in cases {
        let mut harness = dialog_harness(&["a.arw"], settings);

        button(&harness, label).click();
        harness.run();

        assert_eq!(harness.state().intent, Some(intent), "{label}");
    }
}

#[test]
fn quality_is_for_jpeg_only() {
    let mut harness = dialog_harness(&["a.arw"], with_destination());
    assert!(!is_slider_disabled(&harness, QUALITY_LABEL));

    button(&harness, PNG_LABEL).click();
    harness.run();

    assert_eq!(harness.state().settings.format, ExportFormat::Png);
    assert!(is_slider_disabled(&harness, QUALITY_LABEL));
}

#[test]
fn long_edge_value_is_for_the_long_edge_size_only() {
    let mut harness = dialog_harness(&["a.arw"], with_destination());
    assert!(is_slider_disabled(&harness, LONG_EDGE_LABEL));

    button(&harness, LONG_EDGE_CHOICE_LABEL).click();
    harness.run();

    assert_eq!(harness.state().settings.size, ExportSize::LongEdge);
    assert!(!is_slider_disabled(&harness, LONG_EDGE_LABEL));
}
