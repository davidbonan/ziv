use egui::accesskit::Role;
use egui::{Key, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use ziv::design::ui::adjustment_slider::{AdjustmentSlider, Track, TrackScale, value_field_label};

use crate::themed::is_themed;

const EXPOSURE: AdjustmentSlider = AdjustmentSlider {
    label: "Exposure",
    range: -5.0..=5.0,
    default: 0.0,
    step: 0.1,
    decimals: 2,
    scale: TrackScale::Linear,
    track: Track::AccentFill,
};

// Two clicks one frame apart must stay within egui's double-click delay.
const DOUBLE_CLICKABLE_STEP_SECONDS: f32 = 0.1;

fn slider_harness(value: f32) -> Harness<'static, f32> {
    let mut harness = Harness::builder()
        .with_size(vec2(300.0, 80.0))
        .with_step_dt(DOUBLE_CLICKABLE_STEP_SECONDS)
        .build_ui_state(
            |ui, value: &mut f32| {
                if is_themed(ui) {
                    *value = EXPOSURE.show(ui, *value);
                }
            },
            value,
        );
    harness.run();
    harness
}

fn track<'a>(harness: &'a Harness<'static, f32>) -> egui_kittest::Node<'a> {
    harness.get_by_role_and_label(Role::Slider, EXPOSURE.label)
}

fn type_value(harness: &mut Harness<'static, f32>, text: &str, confirm: Key) {
    harness
        .get_by_label(&value_field_label(EXPOSURE.label))
        .click();
    harness.run();
    harness.get_by_role(Role::TextInput).type_text(text);
    harness.run();
    harness.key_press(confirm);
    harness.run();
}

fn double_click_at(harness: &mut Harness<'static, f32>, position: egui::Pos2) {
    harness.hover_at(position);
    harness.run();
    for pressed in [true, false, true, false] {
        harness.event(egui::Event::PointerButton {
            pos: position,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
        harness.step();
    }
    harness.run();
}

#[test]
fn slider_exposes_its_value() {
    let harness = slider_harness(1.25);

    assert_eq!(track(&harness).accesskit_node().numeric_value(), Some(1.25));
}

#[test]
fn dragging_the_track_moves_the_value_with_the_pointer() {
    let mut harness = slider_harness(0.0);
    let start = track(&harness).rect().center();

    harness.drag_at(start);
    harness.run();
    harness.hover_at(start + vec2(60.0, 0.0));
    harness.run();
    harness.drop_at(start + vec2(60.0, 0.0));
    harness.run();

    let travelled = *harness.state() / 10.0 * track(&harness).rect().width();
    assert!(
        (40.0..=70.0).contains(&travelled),
        "value {} after a 60 pt drag",
        harness.state()
    );
}

#[test]
fn pressing_the_track_without_dragging_keeps_the_value() {
    let mut harness = slider_harness(1.0);
    let far_left = track(&harness).rect().left_center() + vec2(10.0, 0.0);

    harness.drag_at(far_left);
    harness.run();
    harness.drop_at(far_left);
    harness.run();

    assert_eq!(*harness.state(), 1.0);
}

#[test]
fn double_click_returns_to_the_default() {
    let mut harness = slider_harness(2.5);

    let middle = track(&harness).rect().center();
    double_click_at(&mut harness, middle);

    assert_eq!(*harness.state(), 0.0);
}

#[test]
fn arrow_keys_step_the_focused_slider() {
    let mut harness = slider_harness(0.0);

    track(&harness).focus();
    harness.run();
    harness.key_press(Key::ArrowRight);
    harness.run();

    assert_eq!(*harness.state(), 0.1);
}

#[test]
fn dragging_does_not_take_the_keyboard_focus() {
    let mut harness = slider_harness(0.0);
    let start = track(&harness).rect().center();

    harness.drag_at(start);
    harness.run();
    harness.drop_at(start + vec2(20.0, 0.0));
    harness.run();

    assert!(!track(&harness).is_focused());
}

#[test]
fn typed_value_replaces_the_shown_one() {
    let mut harness = slider_harness(1.0);

    type_value(&mut harness, "2.5", Key::Enter);

    assert_eq!(*harness.state(), 2.5);
}

#[test]
fn typed_value_outside_the_range_is_brought_back_into_it() {
    let mut harness = slider_harness(0.0);

    type_value(&mut harness, "12", Key::Enter);

    assert_eq!(*harness.state(), 5.0);
}

#[test]
fn escape_abandons_the_typed_value() {
    let mut harness = slider_harness(1.0);

    type_value(&mut harness, "3", Key::Escape);

    assert_eq!(*harness.state(), 1.0);
}

#[test]
fn arrow_keys_stay_with_the_slider_when_a_control_sits_beside_it() {
    let mut harness = Harness::builder()
        .with_size(vec2(500.0, 80.0))
        .build_ui_state(
            |ui, value: &mut f32| {
                if !is_themed(ui) {
                    return;
                }
                ui.horizontal(|ui| {
                    let _ = ui.button("Beside");
                    ui.vertical(|ui| *value = EXPOSURE.show(ui, *value));
                });
            },
            0.0,
        );
    harness.run();

    track(&harness).focus();
    harness.run();
    for _ in 0..3 {
        harness.key_press(Key::ArrowLeft);
        harness.run();
    }

    assert!((*harness.state() + 0.3).abs() < 1e-5, "{}", harness.state());
}
