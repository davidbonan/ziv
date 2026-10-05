use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use ziv::color::domain::illuminant::Illuminant;
use ziv::develop::domain::adjustments::Adjustments;
use ziv::develop::domain::color_grading::TonalZone;
use ziv::develop::domain::color_mixer::ColorRange;
use ziv::develop::domain::edit::Edit;
use ziv::develop::domain::white_balance::WhiteBalance;
use ziv::develop::ui::color_grading_section::{GRADE_SATURATION_LABEL, grading_zone_label};
use ziv::develop::ui::develop_panel::{
    BLACKS_LABEL, COLOR_GRADING_GROUP_LABEL, COLOR_MIXER_GROUP_LABEL, CONTRAST_LABEL, COPY_LABEL,
    DETAIL_GROUP_LABEL, DevelopPanelState, EXPOSURE_LABEL, HIGHLIGHTS_LABEL, PASTE_LABEL,
    PRESENCE_GROUP_LABEL, RESET_LABEL, SATURATION_LABEL, SHADOWS_LABEL, TEMPERATURE_LABEL,
    TINT_LABEL, TONE_CURVE_GROUP_LABEL, TONE_GROUP_LABEL, VIBRANCE_LABEL,
    WHITE_BALANCE_GROUP_LABEL, WHITES_LABEL, develop_panel,
};
use ziv::develop::ui::masks_section::MASKS_GROUP_LABEL;
use ziv::develop::ui::tone_curve_graph::TONE_CURVE_GRAPH_LABEL;
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

#[test]
fn a_press_on_the_tone_curve_graph_adds_a_point_to_the_curve_of_the_photo() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(300.0, 1100.0))
        .build_ui_state(
            |ui, state: &mut DevelopPanelState| {
                if is_themed(ui) {
                    *state = develop_panel(ui, &PhotoKind::StandardImage, state.clone());
                }
            },
            DevelopPanelState::default(),
        );
    harness.run();
    let graph = harness.get_by_label(TONE_CURVE_GRAPH_LABEL).rect();
    let brighter_middle = graph.center() - egui::vec2(0.0, graph.height() / 4.0);

    harness.hover_at(brighter_middle);
    harness.run();
    harness.drag_at(brighter_middle);
    harness.run();
    harness.drop_at(brighter_middle);
    harness.run();

    let curve = &harness.state().edit.tone_curves.rgb;
    assert_eq!(curve.points().len(), 3);
    assert!(curve.applied(0.5) > 0.7);
}

#[test]
fn saturation_of_a_tonal_zone_and_saturation_of_the_photo_are_two_sliders() {
    let mut harness = tall_panel(DevelopPanelState::default());
    harness
        .get_by_label(&grading_zone_label(TonalZone::Shadows, false))
        .click();
    harness.run();

    let saturations: Vec<_> = harness
        .get_all_by_role_and_label(Role::Slider, GRADE_SATURATION_LABEL)
        .collect();
    assert_eq!(saturations.len(), 2);
    let of_the_zone = saturations
        .iter()
        .max_by(|one, other| one.rect().top().total_cmp(&other.rect().top()))
        .unwrap();
    of_the_zone.focus();
    harness.run();
    harness.key_press(egui::Key::ArrowRight);
    harness.run();

    let edit = &harness.state().edit;
    assert_eq!(edit.color_grading.shadows.saturation, 1.0);
    assert_eq!(edit.adjustments.saturation, 0.0);
}

fn tall_panel(state: DevelopPanelState) -> Harness<'static, DevelopPanelState> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(300.0, 2400.0))
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

#[test]
fn the_sections_of_a_photo_come_in_the_order_of_the_development() {
    let harness = tall_panel(DevelopPanelState::default());
    let sections = [
        MASKS_GROUP_LABEL,
        WHITE_BALANCE_GROUP_LABEL,
        TONE_GROUP_LABEL,
        PRESENCE_GROUP_LABEL,
        TONE_CURVE_GROUP_LABEL,
        COLOR_MIXER_GROUP_LABEL,
        COLOR_GRADING_GROUP_LABEL,
        DETAIL_GROUP_LABEL,
    ];

    let tops = sections.map(|section| {
        let title = harness.get_by_label(section);
        title.rect().top()
    });

    assert!(tops.windows(2).all(|pair| pair[0] < pair[1]), "{tops:?}");
}

#[test]
fn the_tone_curve_section_folds_like_the_others() {
    let mut harness = tall_panel(DevelopPanelState::default());

    harness.get_by_label(TONE_CURVE_GROUP_LABEL).click();
    harness.run();

    assert!(harness.query_by_label(TONE_CURVE_GRAPH_LABEL).is_none());
}

#[test]
fn reset_returns_the_curves_the_color_mixer_and_the_color_grading_to_their_defaults() {
    let mut edit = Edit::default();
    edit.tone_curves.red = edit.tone_curves.red.with_point_moved(0, [0.0, 0.2]);
    edit.color_mixer.hue[ColorRange::Red] = 30.0;
    edit.color_grading.balance = 40.0;
    let mut harness = tall_panel(DevelopPanelState {
        edit,
        ..DevelopPanelState::default()
    });

    harness.get_by_label(RESET_LABEL).click();
    harness.run();

    assert_eq!(*harness.state(), DevelopPanelState::default());
}
