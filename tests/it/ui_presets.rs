use std::sync::Arc;

use egui::Key;
use egui::accesskit::Role;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use ziv::develop::domain::coverage_image::CoverageImage;
use ziv::develop::domain::edit::{Edit, MOST_MASKS};
use ziv::develop::domain::linear_gradient::LinearGradient;
use ziv::develop::domain::mask::{Mask, MaskShape};
use ziv::develop::domain::preset::{Preset, PresetGroup};
use ziv::develop::domain::zone::{Zone, ZoneMask};
use ziv::develop::ui::develop_panel::{
    DONE_LABEL, DevelopPanelState, EXPOSURE_LABEL, PRESET_INTENSITY_LABEL, develop_panel,
};
use ziv::develop::ui::masks_section::{MaskSelection, remove_label};
use ziv::photo::domain::photo_kind::PhotoKind;

use crate::themed::is_themed;

const ENHANCED_SKY: &str = "Enhanced sky";
const SUBJECT_POP: &str = "Subject pop";
const APPLIED_ENHANCED_SKY: &str = "Enhanced sky 1";

fn panel(state: DevelopPanelState) -> Harness<'static, DevelopPanelState> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(300.0, 900.0))
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

fn of_edit(edit: Edit) -> DevelopPanelState {
    DevelopPanelState {
        edit,
        ..DevelopPanelState::default()
    }
}

fn detected_sky() -> ZoneMask {
    ZoneMask {
        zone: Zone::Sky,
        coverage: Arc::new(CoverageImage::new([1, 1], vec![255]).unwrap()),
    }
}

/// An edit with Enhanced sky applied, and the id of that applied preset.
fn with_enhanced_sky() -> (Edit, u32) {
    let mut edit = Edit::default();
    let id = edit.apply_preset(Preset::EnhancedSky, vec![detected_sky()]);
    (edit, id)
}

fn holding_masks(count: usize) -> Edit {
    let gradient = MaskShape::LinearGradient(LinearGradient {
        full: [0.2, 0.2],
        none: [0.6, 0.4],
    });
    Edit {
        masks: vec![Mask::of(gradient); count],
        ..Edit::default()
    }
}

fn every_preset() -> impl Iterator<Item = Preset> {
    let grouped = PresetGroup::ALL.into_iter();
    grouped.flat_map(|group| group.presets().iter().copied())
}

#[test]
fn presets_are_sorted_under_the_name_of_their_group() {
    let harness = panel(DevelopPanelState::default());

    for group in PresetGroup::ALL {
        let title = harness.get_by_label(group.name()).rect();
        for preset in group.presets() {
            assert!(harness.get_by_label(preset.name()).rect().top() > title.top());
        }
    }
    let scene = harness.get_by_label(PresetGroup::Scene.name()).rect();
    for preset in PresetGroup::Portrait.presets() {
        assert!(harness.get_by_label(preset.name()).rect().bottom() < scene.top());
    }
}

#[test]
fn presets_sit_two_a_row() {
    let harness = panel(DevelopPanelState::default());
    let button = |preset: Preset| harness.get_by_label(preset.name()).rect();

    let [teeth, eyes, skin] =
        [Preset::WhiterTeeth, Preset::BrightEyes, Preset::SkinGlow].map(button);

    assert_eq!(teeth.top(), eyes.top());
    assert!(eyes.left() >= teeth.right());
    assert!(skin.top() >= teeth.bottom());
    assert_eq!(skin.left(), teeth.left());
}

#[test]
fn each_preset_has_a_button_asking_for_it() {
    for preset in every_preset() {
        let mut harness = panel(DevelopPanelState::default());

        harness.get_by_label(preset.name()).click();
        harness.run_steps(1);

        assert_eq!(harness.state().asked_preset, Some(preset));
    }
}

#[test]
fn presets_wait_for_a_running_detection_and_for_room_among_the_masks() {
    let detecting = DevelopPanelState {
        is_detecting: true,
        ..DevelopPanelState::default()
    };

    for state in [detecting, of_edit(holding_masks(MOST_MASKS))] {
        let harness = panel(state);

        let preset = harness.get_by_label(ENHANCED_SKY);
        assert!(preset.accesskit_node().is_disabled());
    }
}

#[test]
fn preset_making_two_masks_waits_when_one_mask_is_left() {
    let harness = panel(of_edit(holding_masks(MOST_MASKS - 1)));

    let is_disabled = |label| harness.get_by_label(label).accesskit_node().is_disabled();
    assert!(is_disabled(SUBJECT_POP));
    assert!(!is_disabled(ENHANCED_SKY));
}

#[test]
fn applied_preset_of_two_masks_lists_both_under_it() {
    let mut edit = Edit::default();
    let detected = [Zone::Subject, Zone::Background].map(|zone| ZoneMask {
        zone,
        ..detected_sky()
    });
    edit.apply_preset(Preset::SubjectPop, detected.to_vec());

    let harness = panel(of_edit(edit));

    let top_of = |label| harness.get_by_label(label).rect().top();
    assert!(top_of("Subject pop 1") < top_of("Subject 1"));
    assert!(top_of("Subject 1") < top_of("Background 1"));
}

#[test]
fn applied_preset_is_listed_by_name_and_rank_above_its_mask() {
    let (edit, _) = with_enhanced_sky();

    let harness = panel(of_edit(edit));

    let applied = harness.get_by_label(APPLIED_ENHANCED_SKY);
    let mask = harness.get_by_role_and_label(Role::Button, "Sky 1");
    assert!(applied.rect().top() < mask.rect().top());
}

fn selecting_the_applied_preset() -> (DevelopPanelState, u32) {
    let (edit, id) = with_enhanced_sky();
    let state = DevelopPanelState {
        mask_selection: MaskSelection::default().selecting_applied_preset(Some(id)),
        ..of_edit(edit)
    };
    (state, id)
}

#[test]
fn selected_applied_preset_shows_its_intensity_instead_of_the_sliders_of_the_photo() {
    let (state, _) = selecting_the_applied_preset();

    let harness = panel(state);

    let intensity = harness.get_by_role_and_label(Role::Slider, PRESET_INTENSITY_LABEL);
    assert_eq!(intensity.accesskit_node().numeric_value(), Some(100.0));
    assert!(harness.query_by_label(EXPOSURE_LABEL).is_none());
}

#[test]
fn intensity_slider_doses_the_applied_preset() {
    let (state, id) = selecting_the_applied_preset();
    let mut harness = panel(state);

    harness
        .get_by_role_and_label(Role::Slider, PRESET_INTENSITY_LABEL)
        .focus();
    harness.run();
    for _ in 0..3 {
        harness.key_press(Key::ArrowLeft);
        harness.run();
    }

    let applied = harness.state().edit.applied_preset(id).unwrap();
    assert_eq!(applied.intensity, 97.0);
}

#[test]
fn done_returns_to_the_sliders_of_the_photo() {
    let (state, _) = selecting_the_applied_preset();
    let mut harness = panel(state);

    harness.get_by_label(DONE_LABEL).click();
    harness.run();

    assert_eq!(harness.state().mask_selection, MaskSelection::default());
    assert!(harness.query_by_label(EXPOSURE_LABEL).is_some());
}

/// A drawn mask, then Subject pop, then Enhanced sky; the ids of the two applied presets.
fn drawn_mask_and_two_applied_presets() -> (Edit, [u32; 2]) {
    let mut edit = holding_masks(1);
    let detected = [Zone::Subject, Zone::Background].map(|zone| ZoneMask {
        zone,
        ..detected_sky()
    });
    let subject_pop = edit.apply_preset(Preset::SubjectPop, detected.to_vec());
    let enhanced_sky = edit.apply_preset(Preset::EnhancedSky, vec![detected_sky()]);
    (edit, [subject_pop, enhanced_sky])
}

#[test]
fn remove_button_of_an_applied_preset_removes_all_its_masks_and_no_other() {
    let (edit, _) = drawn_mask_and_two_applied_presets();
    let mut harness = panel(of_edit(edit));

    harness.get_by_label(&remove_label("Subject pop 1")).click();
    harness.run();

    let left = &harness.state().edit;
    assert_eq!(left.mask_names(), ["Linear gradient 1", "Sky 1"]);
    assert!(harness.query_by_label("Subject pop 1").is_none());
}

#[test]
fn delete_key_removes_the_selected_applied_preset() {
    let (edit, [_, enhanced_sky]) = drawn_mask_and_two_applied_presets();
    let state = DevelopPanelState {
        mask_selection: MaskSelection::default().selecting_applied_preset(Some(enhanced_sky)),
        ..of_edit(edit)
    };
    let mut harness = panel(state);

    harness.key_press(Key::Delete);
    harness.run();

    let left = harness.state();
    assert_eq!(
        left.edit.mask_names(),
        ["Linear gradient 1", "Subject 1", "Background 1"]
    );
    assert_eq!(left.mask_selection, MaskSelection::default());
}

#[test]
fn removing_an_applied_preset_keeps_the_mask_selected_after_it_selected() {
    let (edit, _) = drawn_mask_and_two_applied_presets();
    let sky_mask = 3;
    let state = DevelopPanelState {
        mask_selection: MaskSelection::of(sky_mask),
        ..of_edit(edit)
    };
    let mut harness = panel(state);

    harness.get_by_label(&remove_label("Subject pop 1")).click();
    harness.run();

    let left = harness.state();
    let selected = left.mask_selection.selected.unwrap();
    assert_eq!(left.edit.mask_names()[selected], "Sky 1");
}

#[test]
fn removing_one_mask_of_an_applied_preset_leaves_its_other_mask_under_it() {
    let (edit, _) = drawn_mask_and_two_applied_presets();
    let mut harness = panel(of_edit(edit));

    harness.get_by_label(&remove_label("Subject 1")).click();
    harness.run();

    assert!(harness.query_by_label("Subject pop 1").is_some());
    assert!(harness.query_by_label("Background 1").is_some());

    harness.get_by_label(&remove_label("Background 1")).click();
    harness.run();

    assert!(harness.query_by_label("Subject pop 1").is_none());
    assert!(harness.query_by_label("Sky 1").is_some());
}

#[test]
fn applied_preset_is_selected_by_its_row_and_left_by_escape() {
    let (edit, id) = with_enhanced_sky();
    let mut harness = panel(of_edit(edit));

    harness.get_by_label(APPLIED_ENHANCED_SKY).click();
    harness.run();
    assert_eq!(harness.state().mask_selection.applied_preset, Some(id));

    harness.key_press(Key::Escape);
    harness.run();
    assert_eq!(harness.state().mask_selection, MaskSelection::default());
}
