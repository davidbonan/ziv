use egui::{Pos2, Rect, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use ziv::develop::domain::color_grading::ZoneGrade;
use ziv::develop::ui::color_wheel::{Wheel, color_wheel};

const WHEEL_NAME: &str = "Shadows wheel";
const DIAMETER: f32 = 160.0;
// Two clicks one frame apart must stay within egui's double-click delay.
const DOUBLE_CLICKABLE_STEP_SECONDS: f32 = 0.1;

fn wheel_harness(grade: ZoneGrade) -> Harness<'static, ZoneGrade> {
    let wheel = Wheel {
        name: WHEEL_NAME,
        diameter: DIAMETER,
    };
    let mut harness = Harness::builder()
        .with_size(vec2(200.0, 200.0))
        .with_step_dt(DOUBLE_CLICKABLE_STEP_SECONDS)
        .build_ui_state(
            move |ui, grade: &mut ZoneGrade| *grade = color_wheel(ui, &wheel, *grade),
            grade,
        );
    harness.run();
    harness
}

pub fn wheel_area<State>(harness: &Harness<'static, State>, name: &str) -> Rect {
    harness.get_by_label(name).rect()
}

/// A place in the wheel: `along` the way from its centre to its edge, at the angle `hue`.
pub fn in_wheel(area: Rect, hue: f32, along: f32) -> Pos2 {
    let (sine, cosine) = hue.to_radians().sin_cos();
    area.center() + vec2(cosine, -sine) * (area.width() / 2.0 * along)
}

pub fn press_and_release<State>(harness: &mut Harness<'static, State>, at: Pos2) {
    harness.hover_at(at);
    harness.run();
    harness.drag_at(at);
    harness.run();
    harness.drop_at(at);
    harness.run();
}

fn assert_grade(harness: &Harness<'static, ZoneGrade>, hue: f32, saturation: f32) {
    let grade = harness.state();
    let is_close = (grade.hue - hue).abs() < 1.5 && (grade.saturation - saturation).abs() < 4.0;
    assert!(
        is_close,
        "{grade:?} is not hue {hue}, saturation {saturation}"
    );
}

#[test]
fn the_wheel_is_a_square_found_by_its_name() {
    let harness = wheel_harness(ZoneGrade::default());

    assert_eq!(
        wheel_area(&harness, WHEEL_NAME).size(),
        vec2(DIAMETER, DIAMETER)
    );
}

#[test]
fn a_press_sets_the_hue_from_the_angle_and_the_saturation_from_the_distance() {
    for (hue, along) in [(90.0, 0.5), (180.0, 0.25), (300.0, 0.75)] {
        let mut harness = wheel_harness(ZoneGrade::default());
        let place = in_wheel(wheel_area(&harness, WHEEL_NAME), hue, along);

        press_and_release(&mut harness, place);

        let reach = DIAMETER / (DIAMETER - 8.0);
        assert_grade(&harness, hue, along * 100.0 * reach);
    }
}

#[test]
fn a_drag_moves_the_point_and_stops_at_the_rim() {
    let mut harness = wheel_harness(ZoneGrade::default());
    let area = wheel_area(&harness, WHEEL_NAME);

    harness.hover_at(in_wheel(area, 0.0, 0.2));
    harness.run();
    harness.drag_at(in_wheel(area, 0.0, 0.2));
    harness.run();
    harness.hover_at(in_wheel(area, 45.0, 1.6));
    harness.run();
    harness.drop_at(in_wheel(area, 45.0, 1.6));
    harness.run();

    assert_grade(&harness, 45.0, 100.0);
}

#[test]
fn a_press_keeps_the_luminance_of_the_zone() {
    let lit = ZoneGrade {
        luminance: 35.0,
        ..ZoneGrade::default()
    };
    let mut harness = wheel_harness(lit);
    let place = in_wheel(wheel_area(&harness, WHEEL_NAME), 200.0, 0.5);

    press_and_release(&mut harness, place);

    assert_eq!(harness.state().luminance, 35.0);
}

#[test]
fn a_double_click_returns_hue_and_saturation_to_zero() {
    let tinted = ZoneGrade {
        hue: 210.0,
        saturation: 60.0,
        luminance: -20.0,
    };
    let mut harness = wheel_harness(tinted);
    let place = in_wheel(wheel_area(&harness, WHEEL_NAME), 30.0, 0.4);

    harness.hover_at(place);
    harness.run();
    for pressed in [true, false, true, false] {
        harness.event(egui::Event::PointerButton {
            pos: place,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
        harness.step();
    }
    harness.run();

    let untinted = ZoneGrade {
        luminance: -20.0,
        ..ZoneGrade::default()
    };
    assert_eq!(*harness.state(), untinted);
}
