use egui::accesskit::{Role, Toggled};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use ziv::develop::domain::aspect_ratio::{NamedRatio, RatioLock};
use ziv::develop::domain::framing::{Framing, Turn};
use ziv::develop::ui::crop_section::{
    CropSectionShown, CropTools, FLIP_HORIZONTAL_LABEL, FLIP_VERTICAL_LABEL, LEVEL_LABEL,
    LOCK_RATIO_LABEL, RESET_CROP_LABEL, ROTATE_LEFT_LABEL, ROTATE_RIGHT_LABEL, RatioAsked,
    crop_section, ratio_label,
};
use ziv::develop::ui::develop_panel::DONE_LABEL;

use crate::themed::is_themed;

struct Shown {
    section: CropSectionShown,
    ratio_asked: Option<RatioAsked>,
    is_done: bool,
}

fn section(ratio: RatioLock) -> Harness<'static, Shown> {
    let shown = Shown {
        section: CropSectionShown {
            framing: Framing::default(),
            tools: CropTools {
                ratio,
                is_level_armed: false,
            },
        },
        ratio_asked: None,
        is_done: false,
    };
    let mut harness = Harness::builder()
        .with_size(egui::vec2(300.0, 300.0))
        .build_ui_state(
            |ui, shown: &mut Shown| {
                if is_themed(ui) {
                    let left = crop_section(ui, &shown.section);
                    shown.section.framing = left.framing;
                    shown.ratio_asked = shown.ratio_asked.or(left.ratio_asked);
                    shown.section.tools.is_level_armed = left.is_level_armed;
                    shown.is_done |= left.is_done;
                }
            },
            shown,
        );
    harness.run();
    harness
}

fn click(harness: &mut Harness<'static, Shown>, label: &str) {
    harness.get_by_role_and_label(Role::Button, label).click();
    harness.run();
}

fn is_chosen(harness: &Harness<'static, Shown>, ratio: RatioLock) -> bool {
    let label = ratio_label(ratio);
    let choice = harness.get_by_label(&label);
    choice.accesskit_node().toggled() == Some(Toggled::True)
}

const ORIGINAL: RatioLock = RatioLock::Named(NamedRatio::Original);

#[test]
fn the_selector_offers_free_and_the_named_ratios_and_shows_the_one_held() {
    let harness = section(ORIGINAL);

    for named in NamedRatio::ALL {
        let ratio = RatioLock::Named(named);
        assert_eq!(is_chosen(&harness, ratio), ratio == ORIGINAL);
    }
    assert!(!is_chosen(&harness, RatioLock::Free));
    assert!(
        harness
            .query_by_label(&ratio_label(RatioLock::Custom))
            .is_none()
    );
}

#[test]
fn custom_is_offered_only_while_the_frame_is_locked_on_it() {
    let harness = section(RatioLock::Custom);

    assert!(is_chosen(&harness, RatioLock::Custom));
}

#[test]
fn choosing_a_ratio_asks_for_it_and_free_lets_it_go() {
    let mut harness = section(ORIGINAL);

    click(
        &mut harness,
        &ratio_label(RatioLock::Named(NamedRatio::Square)),
    );
    assert_eq!(
        harness.state().ratio_asked,
        Some(RatioAsked::Named(NamedRatio::Square))
    );

    harness.state_mut().ratio_asked = None;
    click(&mut harness, &ratio_label(RatioLock::Free));
    assert_eq!(harness.state().ratio_asked, Some(RatioAsked::Free));
}

#[test]
fn the_lock_holds_the_ratio_of_the_frame_and_lets_it_go_when_asked_again() {
    let mut free = section(RatioLock::Free);
    let lock = free.get_by_label(LOCK_RATIO_LABEL);
    assert_eq!(lock.accesskit_node().toggled(), Some(Toggled::False));
    click(&mut free, LOCK_RATIO_LABEL);
    assert_eq!(free.state().ratio_asked, Some(RatioAsked::OfFrame));

    let mut locked = section(RatioLock::Custom);
    let lock = locked.get_by_label(LOCK_RATIO_LABEL);
    assert_eq!(lock.accesskit_node().toggled(), Some(Toggled::True));
    click(&mut locked, LOCK_RATIO_LABEL);
    assert_eq!(locked.state().ratio_asked, Some(RatioAsked::Free));
}

#[test]
fn done_asks_to_leave() {
    let mut harness = section(ORIGINAL);

    click(&mut harness, DONE_LABEL);

    assert!(harness.state().is_done);
}

#[test]
fn level_arms_the_level_tool_and_disarms_it_when_asked_again() {
    let mut harness = section(ORIGINAL);

    click(&mut harness, LEVEL_LABEL);
    assert!(harness.state().section.tools.is_level_armed);
    let level = harness.get_by_label(LEVEL_LABEL);
    assert_eq!(level.accesskit_node().toggled(), Some(Toggled::True));

    click(&mut harness, LEVEL_LABEL);
    assert!(!harness.state().section.tools.is_level_armed);
}

#[test]
fn the_rotate_buttons_turn_the_photo_by_a_quarter_turn_each_way() {
    let mut harness = section(ORIGINAL);

    click(&mut harness, ROTATE_RIGHT_LABEL);
    let right = Turn::default().turned_right();
    assert_eq!(harness.state().section.framing.turn, right);

    click(&mut harness, ROTATE_LEFT_LABEL);
    click(&mut harness, ROTATE_LEFT_LABEL);
    assert_eq!(
        harness.state().section.framing.turn,
        right.turned_left().turned_left()
    );
}

#[test]
fn the_flip_buttons_mirror_the_photo() {
    let mut harness = section(ORIGINAL);

    click(&mut harness, FLIP_HORIZONTAL_LABEL);
    let flipped = Turn::default().flipped_horizontally();
    assert_eq!(harness.state().section.framing.turn, flipped);

    click(&mut harness, FLIP_VERTICAL_LABEL);
    assert_eq!(
        harness.state().section.framing.turn,
        flipped.flipped_vertically()
    );
}

#[test]
fn reset_crop_waits_for_a_crop_then_gives_the_whole_picture_back_keeping_the_turn() {
    let mut harness = section(ORIGINAL);
    let reset = harness.get_by_label(RESET_CROP_LABEL);
    assert!(reset.accesskit_node().is_disabled());

    click(&mut harness, ROTATE_RIGHT_LABEL);
    harness.state_mut().section.framing.angle = 6.0;
    harness.run();
    click(&mut harness, RESET_CROP_LABEL);

    let framing = harness.state().section.framing;
    assert!(!framing.is_cropped());
    assert_eq!(framing.turn, Turn::default().turned_right());
    assert_eq!(harness.state().ratio_asked, Some(RatioAsked::OfFrame));
}
