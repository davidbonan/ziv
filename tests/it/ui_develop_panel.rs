use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use ziv::color::domain::illuminant::Illuminant;
use ziv::develop::domain::adjustments::Adjustments;
use ziv::develop::domain::white_balance::WhiteBalance;
use ziv::develop::ui::develop_panel::{
    BLACKS_LABEL, CONTRAST_LABEL, COPY_LABEL, DevelopPanelState, EXPOSURE_LABEL, HIGHLIGHTS_LABEL,
    PASTE_LABEL, RESET_LABEL, SATURATION_LABEL, SHADOWS_LABEL, TEMPERATURE_LABEL, TINT_LABEL,
    TONE_GROUP_LABEL, VIBRANCE_LABEL, WHITES_LABEL, develop_panel,
};
use ziv::photo::domain::photo_kind::PhotoKind;

use crate::themed::is_themed;

const RAW: PhotoKind = PhotoKind::Raw {
    as_shot: Illuminant {
        temperature: 5200.0,
        tint: 10.0,
    },
};

fn showing(edit: Adjustments) -> DevelopPanelState {
    DevelopPanelState {
        edit: edit.into(),
        ..DevelopPanelState::default()
    }
}

fn before_harness(state: DevelopPanelState) -> Harness<'static, DevelopPanelState> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(300.0, 600.0))
        .build_ui_state(
            |ui, state: &mut DevelopPanelState| {
                if is_themed(ui) {
                    *state = develop_panel(ui, &PhotoKind::StandardImage, state.clone());
                }
            },
            state,
        );
    harness.run();
    harness
}

fn panel_harness_of(kind: PhotoKind, edit: Adjustments) -> Harness<'static, Adjustments> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(300.0, 600.0))
        .build_ui_state(
            move |ui, edit: &mut Adjustments| {
                if is_themed(ui) {
                    *edit = develop_panel(ui, &kind, showing(*edit)).edit.adjustments;
                }
            },
            edit,
        );
    harness.run();
    harness
}

fn panel_harness(edit: Adjustments) -> Harness<'static, Adjustments> {
    panel_harness_of(PhotoKind::StandardImage, edit)
}

fn slider_value(harness: &Harness<'static, Adjustments>, label: &str) -> Option<f64> {
    harness
        .get_by_role_and_label(Role::Slider, label)
        .accesskit_node()
        .numeric_value()
}

fn exposure_slider<'a>(harness: &'a Harness<'static, Adjustments>) -> egui_kittest::Node<'a> {
    harness.get_by_role_and_label(Role::Slider, EXPOSURE_LABEL)
}

#[test]
fn exposure_slider_shows_the_value_of_the_edit() {
    let harness = panel_harness(Adjustments {
        exposure: 1.25,
        ..Adjustments::default()
    });

    assert_eq!(
        exposure_slider(&harness).accesskit_node().numeric_value(),
        Some(1.25)
    );
}

#[test]
fn moving_the_exposure_slider_changes_the_edit() {
    let mut harness = panel_harness(Adjustments::default());

    exposure_slider(&harness).focus();
    harness.run();
    harness.key_press(egui::Key::ArrowRight);
    harness.run();

    assert!(harness.state().exposure > 0.0);
}

fn stepped_right(label: &str) -> Adjustments {
    let mut harness = panel_harness(Adjustments::default());

    harness.get_by_role_and_label(Role::Slider, label).focus();
    harness.run();
    harness.key_press(egui::Key::ArrowRight);
    harness.run();

    *harness.state()
}

#[test]
fn each_tone_and_presence_slider_changes_its_own_adjustment() {
    let raised = |adjustment: fn(&mut Adjustments) -> &mut f32| {
        let mut edit = Adjustments::default();
        *adjustment(&mut edit) = 1.0;
        edit
    };

    assert_eq!(
        stepped_right(CONTRAST_LABEL),
        raised(|edit| &mut edit.contrast)
    );
    assert_eq!(
        stepped_right(HIGHLIGHTS_LABEL),
        raised(|edit| &mut edit.highlights)
    );
    assert_eq!(
        stepped_right(SHADOWS_LABEL),
        raised(|edit| &mut edit.shadows)
    );
    assert_eq!(stepped_right(WHITES_LABEL), raised(|edit| &mut edit.whites));
    assert_eq!(stepped_right(BLACKS_LABEL), raised(|edit| &mut edit.blacks));
    assert_eq!(
        stepped_right(VIBRANCE_LABEL),
        raised(|edit| &mut edit.vibrance)
    );
    assert_eq!(
        stepped_right(SATURATION_LABEL),
        raised(|edit| &mut edit.saturation)
    );
}

#[test]
fn untouched_raw_shows_its_as_shot_white_balance() {
    let harness = panel_harness_of(RAW, Adjustments::default());

    assert_eq!(slider_value(&harness, TEMPERATURE_LABEL), Some(5200.0));
    assert_eq!(slider_value(&harness, TINT_LABEL), Some(10.0));
}

#[test]
fn untouched_standard_image_shows_a_white_balance_of_zero() {
    let harness = panel_harness(Adjustments::default());

    assert_eq!(slider_value(&harness, TEMPERATURE_LABEL), Some(0.0));
    assert_eq!(slider_value(&harness, TINT_LABEL), Some(0.0));
}

#[test]
fn moving_the_temperature_of_a_raw_sets_a_white_balance_in_kelvin() {
    let mut harness = panel_harness_of(RAW, Adjustments::default());

    harness
        .get_by_role_and_label(Role::Slider, TEMPERATURE_LABEL)
        .focus();
    harness.run();
    harness.key_press(egui::Key::ArrowRight);
    harness.run();

    let expected = WhiteBalance {
        temperature: 5250.0,
        tint: 10.0,
    };
    assert_eq!(harness.state().white_balance, Some(expected));
}

#[test]
fn white_balance_brought_back_to_as_shot_is_no_white_balance_edit() {
    let warmer = Adjustments {
        white_balance: Some(WhiteBalance {
            temperature: 5250.0,
            tint: 10.0,
        }),
        ..Adjustments::default()
    };
    let mut harness = panel_harness_of(RAW, warmer);

    harness
        .get_by_role_and_label(Role::Slider, TEMPERATURE_LABEL)
        .focus();
    harness.run();
    harness.key_press(egui::Key::ArrowLeft);
    harness.run();

    assert_eq!(harness.state().white_balance, None);
}

fn edited() -> Adjustments {
    Adjustments {
        exposure: 1.0,
        contrast: 20.0,
        white_balance: Some(WhiteBalance {
            temperature: 30.0,
            tint: 0.0,
        }),
        ..Adjustments::default()
    }
}

#[test]
fn reset_returns_every_adjustment_to_its_default() {
    let mut harness = panel_harness(edited());

    harness.get_by_label(RESET_LABEL).click();
    harness.run();

    assert_eq!(*harness.state(), Adjustments::default());
}

#[test]
fn reset_is_disabled_when_nothing_is_edited() {
    let harness = panel_harness(Adjustments::default());

    assert!(
        harness
            .get_by_label(RESET_LABEL)
            .accesskit_node()
            .is_disabled()
    );
}

#[test]
fn disabled_panel_shows_the_edit_and_lets_nothing_change() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(300.0, 600.0))
        .build_ui_state(
            |ui, edit: &mut Adjustments| {
                if is_themed(ui) {
                    ui.disable();
                    *edit = develop_panel(ui, &PhotoKind::StandardImage, showing(*edit))
                        .edit
                        .adjustments;
                }
            },
            edited(),
        );
    harness.run();

    let exposure = exposure_slider(&harness);
    assert!(exposure.accesskit_node().is_disabled());
    assert!(
        harness
            .get_by_label(RESET_LABEL)
            .accesskit_node()
            .is_disabled()
    );
    let position = exposure.rect().center();
    harness.drag_at(position);
    harness.run();
    harness.drop_at(position + egui::vec2(40.0, 0.0));
    harness.run();

    assert_eq!(*harness.state(), edited());
}

#[derive(Default)]
struct Asked {
    is_copy_asked: bool,
    is_paste_asked: bool,
}

fn asked_by_clicking(label: &str) -> Asked {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(300.0, 600.0))
        .build_ui_state(
            |ui, asked: &mut Asked| {
                if !is_themed(ui) {
                    return;
                }
                let shown = DevelopPanelState {
                    can_paste: true,
                    ..showing(edited())
                };
                let left = develop_panel(ui, &PhotoKind::StandardImage, shown);
                asked.is_copy_asked |= left.is_copy_asked;
                asked.is_paste_asked |= left.is_paste_asked;
            },
            Asked::default(),
        );
    harness.run();
    harness.get_by_label(label).click();
    harness.run();
    std::mem::take(harness.state_mut())
}

#[test]
fn foot_asks_to_copy_and_to_paste_the_edit() {
    let copy = asked_by_clicking(COPY_LABEL);
    assert!(copy.is_copy_asked && !copy.is_paste_asked);

    let paste = asked_by_clicking(PASTE_LABEL);
    assert!(paste.is_paste_asked && !paste.is_copy_asked);
}

#[test]
fn paste_waits_for_a_copied_edit() {
    let harness = before_harness(showing(edited()));

    assert!(
        harness
            .get_by_label(PASTE_LABEL)
            .accesskit_node()
            .is_disabled()
    );
}

#[test]
fn foot_stays_at_the_bottom_of_the_panel() {
    let harness = before_harness(showing(edited()));

    let reset = harness.get_by_label(RESET_LABEL).rect();
    assert!(reset.bottom() > 560.0, "{reset:?}");
}

#[test]
fn changing_an_adjustment_leaves_before() {
    let mut harness = before_harness(DevelopPanelState {
        is_before_shown: true,
        ..showing(edited())
    });

    harness
        .get_by_role_and_label(Role::Slider, CONTRAST_LABEL)
        .focus();
    harness.run();
    harness.key_press(egui::Key::ArrowRight);
    harness.run();

    assert!(!harness.state().is_before_shown);
    assert_eq!(harness.state().edit.adjustments.contrast, 21.0);
}

#[test]
fn section_folds_and_unfolds_by_its_title_and_keeps_its_values() {
    let mut harness = panel_harness(edited());

    harness.get_by_label(TONE_GROUP_LABEL).click();
    harness.run();
    assert!(harness.query_by_label(EXPOSURE_LABEL).is_none());
    assert!(harness.query_by_label(VIBRANCE_LABEL).is_some());
    assert_eq!(*harness.state(), edited());

    harness.get_by_label(TONE_GROUP_LABEL).click();
    harness.run();
    assert!(harness.query_by_label(EXPOSURE_LABEL).is_some());
}
