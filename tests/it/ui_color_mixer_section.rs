use egui::accesskit::{Role, Toggled};
use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use ziv::develop::domain::color_mixer::{ColorMixer, ColorRange, MixerAdjustment};
use ziv::develop::ui::color_mixer_section::{
    color_mixer_section, color_range_label, mixer_adjustment_label,
};

fn section_harness(mixer: ColorMixer) -> Harness<'static, ColorMixer> {
    let mut harness = Harness::builder()
        .with_size(vec2(268.0, 420.0))
        .build_ui_state(
            |ui, mixer: &mut ColorMixer| *mixer = color_mixer_section(ui, mixer),
            mixer,
        );
    harness.run();
    harness
}

fn with(adjust: impl FnOnce(&mut ColorMixer)) -> ColorMixer {
    let mut mixer = ColorMixer::default();
    adjust(&mut mixer);
    mixer
}

fn label_of(harness: &Harness<'static, ColorMixer>, adjustment: MixerAdjustment) -> String {
    let is_edited = !harness.state().of(adjustment).is_default();
    mixer_adjustment_label(adjustment, is_edited)
}

fn show(harness: &mut Harness<'static, ColorMixer>, adjustment: MixerAdjustment) {
    let label = label_of(harness, adjustment);
    harness.get_by_label(&label).click();
    harness.run();
}

fn slider_value(harness: &Harness<'static, ColorMixer>, range: ColorRange) -> Option<f64> {
    harness
        .get_by_role_and_label(Role::Slider, color_range_label(range))
        .accesskit_node()
        .numeric_value()
}

fn step_right(harness: &mut Harness<'static, ColorMixer>, range: ColorRange) {
    harness
        .get_by_role_and_label(Role::Slider, color_range_label(range))
        .focus();
    harness.run();
    harness.key_press(egui::Key::ArrowRight);
    harness.run();
}

#[test]
fn the_selector_offers_the_three_adjustments_and_starts_on_hue() {
    let harness = section_harness(ColorMixer::default());

    for adjustment in MixerAdjustment::ALL {
        let label = label_of(&harness, adjustment);
        let is_shown = harness.get_by_label(&label).accesskit_node().toggled();
        let expected = match adjustment == MixerAdjustment::Hue {
            true => Toggled::True,
            false => Toggled::False,
        };
        assert_eq!(is_shown, Some(expected), "{adjustment:?}");
    }
}

#[test]
fn the_eight_color_ranges_have_their_slider_from_red_down_to_magenta() {
    let harness = section_harness(ColorMixer::default());

    let tops = ColorRange::ALL.map(|range| {
        let slider = harness.get_by_role_and_label(Role::Slider, color_range_label(range));
        slider.rect().top()
    });

    assert!(tops.windows(2).all(|pair| pair[0] < pair[1]), "{tops:?}");
}

#[test]
fn the_sliders_show_the_values_of_the_adjustment_chosen() {
    let mut harness = section_harness(with(|mixer| {
        mixer.hue[ColorRange::Green] = 30.0;
        mixer.luminance[ColorRange::Green] = -45.0;
    }));

    assert_eq!(slider_value(&harness, ColorRange::Green), Some(30.0));
    show(&mut harness, MixerAdjustment::Luminance);
    assert_eq!(slider_value(&harness, ColorRange::Green), Some(-45.0));
    show(&mut harness, MixerAdjustment::Saturation);
    assert_eq!(slider_value(&harness, ColorRange::Green), Some(0.0));
}

#[test]
fn moving_a_slider_changes_its_range_of_the_adjustment_shown_alone() {
    let mut harness = section_harness(ColorMixer::default());

    show(&mut harness, MixerAdjustment::Saturation);
    step_right(&mut harness, ColorRange::Blue);

    let expected = with(|mixer| mixer.saturation[ColorRange::Blue] = 1.0);
    assert_eq!(*harness.state(), expected);
}

#[test]
fn an_adjustment_holding_a_value_away_from_its_default_is_marked() {
    let harness = section_harness(with(|mixer| mixer.luminance[ColorRange::Aqua] = -20.0));

    for adjustment in MixerAdjustment::ALL {
        let marked = mixer_adjustment_label(adjustment, true);
        let is_marked = harness.query_by_label(&marked).is_some();
        assert_eq!(
            is_marked,
            adjustment == MixerAdjustment::Luminance,
            "{adjustment:?}"
        );
    }
}
