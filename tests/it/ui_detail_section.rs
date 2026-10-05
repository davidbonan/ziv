use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use ziv::develop::domain::edit::Edit;
use ziv::develop::ui::develop_panel::{DevelopPanelState, develop_panel};
use ziv::enhance::ui::detail_section::{ENHANCE_LABEL, INTENSITY_LABEL};
use ziv::photo::domain::photo_kind::PhotoKind;

use crate::themed::is_themed;

/// The panel as the app shows it: what it is given stays, what the user asks is kept.
fn panel_harness(state: DevelopPanelState) -> Harness<'static, DevelopPanelState> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(300.0, 2400.0))
        .build_ui_state(
            |ui, state: &mut DevelopPanelState| {
                if !is_themed(ui) {
                    return;
                }
                let left = develop_panel(ui, &PhotoKind::StandardImage, state.clone());
                state.edit = left.edit;
                state.is_enhancement_asked |= left.is_enhancement_asked;
            },
            state,
        );
    harness.run();
    harness
}

fn enhanced_at(enhancement_intensity: f32) -> DevelopPanelState {
    DevelopPanelState {
        edit: Edit {
            enhancement_intensity,
            ..Edit::default()
        },
        is_enhanced: true,
        ..DevelopPanelState::default()
    }
}

#[test]
fn photo_without_enhancement_offers_enhance_and_no_intensity() {
    let mut harness = panel_harness(DevelopPanelState::default());

    assert!(harness.query_by_label(INTENSITY_LABEL).is_none());
    harness.get_by_label(ENHANCE_LABEL).click();
    harness.run();

    assert!(harness.state().is_enhancement_asked);
}

#[test]
fn enhance_waits_for_the_enhancement_that_runs() {
    let harness = panel_harness(DevelopPanelState {
        is_enhancing: true,
        ..DevelopPanelState::default()
    });

    let enhance = harness.get_by_label(ENHANCE_LABEL);

    assert!(enhance.accesskit_node().is_disabled());
}

#[test]
fn enhanced_photo_shows_its_intensity_instead_of_enhance() {
    let mut harness = panel_harness(enhanced_at(60.0));

    let intensity = harness.get_by_role_and_label(Role::Slider, INTENSITY_LABEL);
    assert_eq!(intensity.accesskit_node().numeric_value(), Some(60.0));
    assert!(harness.query_by_label(ENHANCE_LABEL).is_none());

    intensity.focus();
    harness.run();
    harness.key_press(egui::Key::ArrowRight);
    harness.run();

    assert_eq!(harness.state().edit.enhancement_intensity, 61.0);
}
