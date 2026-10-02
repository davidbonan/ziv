use egui::Key;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use ziv::color::domain::illuminant::Illuminant;
use ziv::develop::domain::adjustments::Adjustments;
use ziv::develop::domain::brush::BrushMask;
use ziv::develop::domain::edit::{Edit, MOST_MASKS};
use ziv::develop::domain::linear_gradient::LinearGradient;
use ziv::develop::domain::mask::{Mask, MaskKind, MaskShape};
use ziv::develop::domain::radial_gradient::RadialGradient;
use ziv::develop::domain::zone::{DetectionTool, ZoneTool};
use ziv::develop::ui::develop_panel::{
    BRUSH_SIZE_LABEL, DONE_LABEL, DevelopPanelState, EXPOSURE_LABEL, FEATHER_LABEL, INVERT_LABEL,
    OVERLAY_LABEL, RESET_LABEL, TEMPERATURE_LABEL, develop_panel,
};
use ziv::develop::ui::masks_section::{
    BACKGROUND_TOOL_LABEL, LINEAR_GRADIENT_TOOL_LABEL, MaskSelection, PEOPLE_TOOL_LABEL,
    SUBJECT_TOOL_LABEL, hide_label, remove_label, show_label,
};
use ziv::photo::domain::photo_kind::PhotoKind;

use crate::themed::is_themed;

const FIRST: &str = "Linear gradient 1";
const SECOND: &str = "Linear gradient 2";
const RAW: PhotoKind = PhotoKind::Raw {
    as_shot: Illuminant {
        temperature: 5200.0,
        tint: 10.0,
    },
};

fn gradient(exposure: f32) -> Mask {
    let shape = MaskShape::LinearGradient(LinearGradient {
        full: [0.2, 0.2],
        none: [0.6, 0.4],
    });
    Mask {
        adjustments: Adjustments {
            exposure,
            ..Adjustments::default()
        },
        ..Mask::of(shape)
    }
}

fn with_masks(masks: Vec<Mask>) -> DevelopPanelState {
    DevelopPanelState {
        edit: Edit {
            masks,
            ..Edit::default()
        },
        ..DevelopPanelState::default()
    }
}

fn two_masks() -> DevelopPanelState {
    with_masks(vec![gradient(1.0), gradient(-2.0)])
}

fn selecting(index: usize, state: DevelopPanelState) -> DevelopPanelState {
    DevelopPanelState {
        mask_selection: MaskSelection::of(index),
        ..state
    }
}

fn panel(state: DevelopPanelState) -> Harness<'static, DevelopPanelState> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(300.0, 900.0))
        .build_ui_state(
            |ui, state: &mut DevelopPanelState| {
                if is_themed(ui) {
                    *state = develop_panel(ui, &RAW, state.clone());
                }
            },
            state,
        );
    harness.run();
    harness
}

fn click(harness: &mut Harness<'static, DevelopPanelState>, label: &str) {
    harness.get_by_label(label).click();
    harness.run();
}

fn slider_value(harness: &Harness<'static, DevelopPanelState>, label: &str) -> Option<f64> {
    harness
        .get_by_role_and_label(Role::Slider, label)
        .accesskit_node()
        .numeric_value()
}

#[test]
fn masks_are_listed_by_name() {
    let harness = panel(two_masks());

    assert!(harness.query_by_label(FIRST).is_some());
    assert!(harness.query_by_label(SECOND).is_some());
}

#[test]
fn tool_button_arms_its_tool_and_disarms_it_when_asked_again() {
    let mut harness = panel(DevelopPanelState::default());

    click(&mut harness, LINEAR_GRADIENT_TOOL_LABEL);
    assert_eq!(
        harness.state().mask_selection.armed_tool,
        Some(MaskKind::LinearGradient)
    );

    click(&mut harness, LINEAR_GRADIENT_TOOL_LABEL);
    assert_eq!(harness.state().mask_selection.armed_tool, None);
}

#[test]
fn tools_are_disabled_when_the_photo_holds_all_the_masks_it_can() {
    let harness = panel(with_masks(vec![gradient(0.0); MOST_MASKS]));

    assert!(
        harness
            .get_by_label(LINEAR_GRADIENT_TOOL_LABEL)
            .accesskit_node()
            .is_disabled()
    );
}

#[test]
fn selected_mask_shows_its_own_adjustments() {
    let mut harness = panel(two_masks());
    assert_eq!(slider_value(&harness, EXPOSURE_LABEL), Some(0.0));

    click(&mut harness, SECOND);

    assert_eq!(harness.state().mask_selection.selected, Some(1));
    assert_eq!(slider_value(&harness, EXPOSURE_LABEL), Some(-2.0));
}

#[test]
fn sliders_change_the_selected_mask_and_leave_the_photo_alone() {
    let mut harness = panel(selecting(0, two_masks()));

    harness
        .get_by_role_and_label(Role::Slider, EXPOSURE_LABEL)
        .focus();
    harness.run();
    harness.key_press(Key::ArrowRight);
    harness.run();

    let edit = &harness.state().edit;
    assert_eq!(edit.masks[0].adjustments.exposure, 1.1);
    assert_eq!(edit.masks[1], gradient(-2.0));
    assert_eq!(edit.adjustments, Adjustments::default());
}

#[test]
fn mask_white_balance_is_relative_on_a_raw() {
    let photo = panel(two_masks());
    let mask = panel(selecting(0, two_masks()));

    assert_eq!(slider_value(&photo, TEMPERATURE_LABEL), Some(5200.0));
    assert_eq!(slider_value(&mask, TEMPERATURE_LABEL), Some(0.0));
}

#[test]
fn done_escape_and_a_second_click_return_to_the_photo() {
    let mut done = panel(selecting(0, two_masks()));
    let mut escaped = panel(selecting(0, two_masks()));
    let mut clicked_again = panel(selecting(0, two_masks()));

    click(&mut done, DONE_LABEL);
    escaped.key_press(Key::Escape);
    escaped.run();
    clicked_again
        .get_by_role_and_label(Role::Button, FIRST)
        .click();
    clicked_again.run();

    for harness in [&done, &escaped, &clicked_again] {
        assert_eq!(harness.state().mask_selection, MaskSelection::default());
        assert_eq!(harness.state().edit, two_masks().edit);
    }
}

#[test]
fn eye_hides_the_mask_and_shows_it_again() {
    let mut harness = panel(two_masks());

    click(&mut harness, &hide_label(FIRST));
    assert!(harness.state().edit.masks[0].is_hidden);
    assert!(!harness.state().edit.masks[1].is_hidden);

    click(&mut harness, &show_label(FIRST));
    assert!(!harness.state().edit.masks[0].is_hidden);
}

#[test]
fn remove_button_deletes_its_mask_and_keeps_the_other_selected() {
    let mut harness = panel(selecting(1, two_masks()));

    click(&mut harness, &remove_label(FIRST));

    assert_eq!(harness.state().edit.masks, [gradient(-2.0)]);
    assert_eq!(harness.state().mask_selection.selected, Some(0));
}

#[test]
fn delete_key_removes_the_selected_mask() {
    let mut harness = panel(selecting(0, two_masks()));

    harness.key_press(Key::Delete);
    harness.run();

    assert_eq!(harness.state().edit.masks, [gradient(-2.0)]);
    assert_eq!(harness.state().mask_selection.selected, None);
}

#[test]
fn invert_swaps_what_the_selected_mask_covers() {
    let mut harness = panel(selecting(1, two_masks()));

    click(&mut harness, INVERT_LABEL);

    assert!(harness.state().edit.masks[1].is_inverted);
    assert!(!harness.state().edit.masks[0].is_inverted);
}

#[test]
fn reset_removes_the_masks() {
    let mut harness = panel(selecting(0, two_masks()));

    click(&mut harness, RESET_LABEL);

    assert_eq!(*harness.state(), DevelopPanelState::default());
}

fn is_overlaid(harness: &Harness<'static, DevelopPanelState>) -> bool {
    let state = harness.state();
    state.mask_selection.overlaid_mask(&state.edit).is_some()
}

#[test]
fn overlay_shows_on_a_mask_without_adjustment_until_one_is_moved() {
    let mut harness = panel(selecting(0, with_masks(vec![gradient(0.0)])));
    assert!(is_overlaid(&harness));

    harness
        .get_by_role_and_label(Role::Slider, EXPOSURE_LABEL)
        .focus();
    harness.run();
    harness.key_press(Key::ArrowRight);
    harness.run();

    assert!(!is_overlaid(&harness));
}

#[test]
fn overlay_key_and_button_show_and_hide_the_overlay() {
    let mut harness = panel(selecting(0, two_masks()));
    assert!(!is_overlaid(&harness));

    harness.key_press(Key::O);
    harness.run();
    assert!(is_overlaid(&harness));

    click(&mut harness, OVERLAY_LABEL);
    assert!(!is_overlaid(&harness));
}

#[test]
fn no_overlay_without_a_selected_mask() {
    let harness = panel(with_masks(vec![gradient(0.0)]));

    assert!(!is_overlaid(&harness));
}

#[test]
fn feather_slider_shows_only_for_a_shape_that_has_a_feather() {
    let ellipse = Mask::of(MaskShape::RadialGradient(RadialGradient {
        centre: [0.5, 0.3],
        radii: [0.2, 0.1],
        rotation: 0.0,
        feather: 50.0,
    }));
    let linear = panel(selecting(0, two_masks()));
    let mut radial = panel(selecting(0, with_masks(vec![ellipse])));
    assert!(linear.query_by_label(FEATHER_LABEL).is_none());

    radial
        .get_by_role_and_label(Role::Slider, FEATHER_LABEL)
        .focus();
    radial.run();
    radial.key_press(Key::ArrowRight);
    radial.run();

    assert_eq!(radial.state().edit.masks[0].shape.feather(), Some(51.0));
}

#[test]
fn brush_sliders_show_for_a_brush_mask_and_set_the_brush() {
    let brush_mask = Mask::of(MaskShape::Brush(BrushMask::default()));
    let gradient_selected = panel(selecting(0, two_masks()));
    let mut brush_selected = panel(selecting(0, with_masks(vec![brush_mask])));
    assert!(gradient_selected.query_by_label(BRUSH_SIZE_LABEL).is_none());

    brush_selected
        .get_by_role_and_label(Role::Slider, BRUSH_SIZE_LABEL)
        .focus();
    brush_selected.run();
    brush_selected.key_press(Key::ArrowRight);
    brush_selected.run();

    assert_eq!(brush_selected.state().mask_selection.brush.size, 21.0);
}

#[test]
fn zone_tool_asks_for_its_detection() {
    let mut harness = panel(DevelopPanelState::default());

    click(&mut harness, BACKGROUND_TOOL_LABEL);
    let background = DetectionTool::Zone(ZoneTool::Background);
    assert_eq!(harness.state().asked_detection, Some(background));

    click(&mut harness, PEOPLE_TOOL_LABEL);
    assert_eq!(harness.state().asked_detection, Some(DetectionTool::People));
    assert_eq!(harness.state().mask_selection.armed_tool, None);
}

#[test]
fn zone_tools_wait_for_the_running_detection_and_for_room() {
    let detecting = DevelopPanelState {
        is_detecting: true,
        ..DevelopPanelState::default()
    };
    let full = with_masks(vec![gradient(0.0); MOST_MASKS]);

    for state in [detecting, full] {
        let harness = panel(state);

        let subject_tool = harness.get_by_label(SUBJECT_TOOL_LABEL);
        assert!(subject_tool.accesskit_node().is_disabled());
    }
}

#[test]
fn masks_title_carries_the_number_of_masks() {
    assert!(panel(two_masks()).query_by_label("Masks (2)").is_some());
    assert!(
        panel(with_masks(Vec::new()))
            .query_by_label("Masks")
            .is_some()
    );
}
