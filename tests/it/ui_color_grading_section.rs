use egui::accesskit::{Role, Toggled};
use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use ziv::develop::domain::color_grading::{ColorGrading, TonalZone, ZoneGrade};
use ziv::develop::ui::color_grading_section::{
    BALANCE_LABEL, BLENDING_LABEL, GRADE_HUE_LABEL, GRADE_LUMINANCE_LABEL, GRADE_SATURATION_LABEL,
    THREE_WAY_LABEL, color_grading_section, grading_zone_label, zone_luminance_label,
    zone_wheel_label,
};

use crate::ui_color_wheel::{in_wheel, press_and_release, wheel_area};

const THREE_WAY_ZONES: [TonalZone; 3] = [
    TonalZone::Shadows,
    TonalZone::Midtones,
    TonalZone::Highlights,
];
const SECTION_WIDTH: f32 = 252.0;

fn section_harness(grading: ColorGrading) -> Harness<'static, ColorGrading> {
    let mut harness = Harness::builder()
        .with_size(vec2(SECTION_WIDTH, 520.0))
        .build_ui_state(
            |ui, grading: &mut ColorGrading| *grading = color_grading_section(ui, grading),
            grading,
        );
    harness.run();
    harness
}

fn teal() -> ZoneGrade {
    ZoneGrade {
        hue: 180.0,
        saturation: 40.0,
        luminance: 10.0,
    }
}

fn show(harness: &mut Harness<'static, ColorGrading>, zone: TonalZone) {
    let is_edited = !harness.state().of(zone).is_default();
    harness
        .get_by_label(&grading_zone_label(zone, is_edited))
        .click();
    harness.run();
}

fn slider_value(harness: &Harness<'static, ColorGrading>, label: &str) -> Option<f64> {
    harness
        .get_by_role_and_label(Role::Slider, label)
        .accesskit_node()
        .numeric_value()
}

fn has_slider(harness: &Harness<'static, ColorGrading>, label: &str) -> bool {
    harness
        .query_by_role_and_label(Role::Slider, label)
        .is_some()
}

fn step_right(harness: &mut Harness<'static, ColorGrading>, label: &str) {
    harness.get_by_role_and_label(Role::Slider, label).focus();
    harness.run();
    harness.key_press(egui::Key::ArrowRight);
    harness.run();
}

#[test]
fn the_section_starts_on_the_three_wheels_each_with_its_luminance() {
    let harness = section_harness(ColorGrading::default());

    let three_way = harness.get_by_label(THREE_WAY_LABEL);
    assert_eq!(three_way.accesskit_node().toggled(), Some(Toggled::True));
    let lefts = THREE_WAY_ZONES.map(|zone| wheel_area(&harness, zone_wheel_label(zone)).left());
    assert!(lefts[0] < lefts[1] && lefts[1] < lefts[2], "{lefts:?}");
    for zone in THREE_WAY_ZONES {
        assert!(has_slider(&harness, zone_luminance_label(zone)), "{zone:?}");
    }
    assert!(
        harness
            .query_by_label(zone_wheel_label(TonalZone::Global))
            .is_none()
    );
}

#[test]
fn the_three_wheels_fit_the_width_of_the_section() {
    let harness = section_harness(ColorGrading::default());

    let highlights = wheel_area(&harness, zone_wheel_label(TonalZone::Highlights));

    assert!(highlights.right() <= SECTION_WIDTH, "{highlights:?}");
}

#[test]
fn a_wheel_of_the_three_way_view_grades_its_own_zone() {
    let mut harness = section_harness(ColorGrading::default());
    let midtones = wheel_area(&harness, zone_wheel_label(TonalZone::Midtones));

    press_and_release(&mut harness, in_wheel(midtones, 90.0, 0.5));
    step_right(&mut harness, zone_luminance_label(TonalZone::Highlights));

    let grading = harness.state();
    assert!((grading.midtones.hue - 90.0).abs() < 2.0, "{grading:?}");
    assert!(grading.midtones.saturation > 40.0);
    assert_eq!(grading.highlights.luminance, 1.0);
    assert!(grading.shadows.is_default());
}

#[test]
fn a_zone_view_shows_one_wheel_and_the_three_sliders_of_the_zone() {
    for zone in TonalZone::ALL {
        let mut harness = section_harness(ColorGrading::default().with(zone, teal()));

        show(&mut harness, zone);

        assert!(harness.query_by_label(zone_wheel_label(zone)).is_some());
        assert_eq!(slider_value(&harness, GRADE_HUE_LABEL), Some(180.0));
        assert_eq!(slider_value(&harness, GRADE_SATURATION_LABEL), Some(40.0));
        assert_eq!(slider_value(&harness, GRADE_LUMINANCE_LABEL), Some(10.0));
        let other_wheels = TonalZone::ALL
            .into_iter()
            .filter(|other| *other != zone)
            .filter(|other| harness.query_by_label(zone_wheel_label(*other)).is_some());
        assert_eq!(other_wheels.count(), 0);
    }
}

#[test]
fn the_wheel_and_the_sliders_of_a_zone_show_the_same_values() {
    let mut harness = section_harness(ColorGrading::default());

    show(&mut harness, TonalZone::Global);
    let wheel = wheel_area(&harness, zone_wheel_label(TonalZone::Global));
    press_and_release(&mut harness, in_wheel(wheel, 270.0, 0.5));

    let global = harness.state().global;
    assert!((global.hue - 270.0).abs() < 2.0, "{global:?}");
    assert_eq!(
        slider_value(&harness, GRADE_HUE_LABEL),
        Some(f64::from(global.hue))
    );
    assert_eq!(
        slider_value(&harness, GRADE_SATURATION_LABEL),
        Some(f64::from(global.saturation))
    );
}

#[test]
fn blending_and_balance_are_under_every_view() {
    let mut harness = section_harness(ColorGrading::default());

    assert_eq!(slider_value(&harness, BLENDING_LABEL), Some(50.0));
    assert_eq!(slider_value(&harness, BALANCE_LABEL), Some(0.0));
    for zone in TonalZone::ALL {
        show(&mut harness, zone);
        assert!(has_slider(&harness, BLENDING_LABEL), "{zone:?}");
        assert!(has_slider(&harness, BALANCE_LABEL), "{zone:?}");
    }
}

#[test]
fn moving_blending_and_balance_changes_them_alone() {
    let mut harness = section_harness(ColorGrading::default());

    step_right(&mut harness, BLENDING_LABEL);
    step_right(&mut harness, BALANCE_LABEL);

    let expected = ColorGrading {
        blending: 51.0,
        balance: 1.0,
        ..ColorGrading::default()
    };
    assert_eq!(*harness.state(), expected);
}

#[test]
fn a_zone_away_from_its_defaults_is_marked_on_the_selector() {
    let grading = ColorGrading::default().with(TonalZone::Highlights, teal());
    let harness = section_harness(grading);

    for zone in TonalZone::ALL {
        let marked = grading_zone_label(zone, true);
        let is_marked = harness.query_by_label(&marked).is_some();
        assert_eq!(is_marked, zone == TonalZone::Highlights, "{zone:?}");
    }
}
