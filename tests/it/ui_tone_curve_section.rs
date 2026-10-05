use egui::accesskit::Toggled;
use egui::vec2;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use ziv::develop::domain::tone_curve::{CurveChannel, CurvePoint, ToneCurve, ToneCurves};
use ziv::develop::ui::tone_curve_section::{
    RESET_CURVE_LABEL, curve_channel_label, tone_curve_section,
};

use crate::ui_tone_curve_graph::drag;

fn section_harness(curves: ToneCurves) -> Harness<'static, ToneCurves> {
    let mut harness = Harness::builder()
        .with_size(vec2(248.0, 340.0))
        .build_ui_state(
            |ui, curves: &mut ToneCurves| *curves = tone_curve_section(ui, curves, None),
            curves,
        );
    harness.run();
    harness
}

fn curve(points: &[CurvePoint]) -> ToneCurve {
    ToneCurve::try_from(points.to_vec()).unwrap()
}

fn raised_middle() -> ToneCurve {
    curve(&[[0.0, 0.0], [0.5, 0.7], [1.0, 1.0]])
}

fn raised_on(channel: CurveChannel) -> ToneCurves {
    ToneCurves::default().with(channel, raised_middle())
}

fn show(harness: &mut Harness<'static, ToneCurves>, channel: CurveChannel) {
    let is_edited = !harness.state().of(channel).is_identity();
    harness
        .get_by_label(&curve_channel_label(channel, is_edited))
        .click();
    harness.run();
}

fn is_shown(harness: &Harness<'static, ToneCurves>, channel: CurveChannel) -> bool {
    let is_edited = !harness.state().of(channel).is_identity();
    harness
        .get_by_label(&curve_channel_label(channel, is_edited))
        .accesskit_node()
        .toggled()
        == Some(Toggled::True)
}

#[test]
fn the_selector_offers_the_four_channels_and_starts_on_rgb() {
    let harness = section_harness(ToneCurves::default());

    for channel in CurveChannel::ALL {
        let is_rgb = channel == CurveChannel::Rgb;
        assert_eq!(is_shown(&harness, channel), is_rgb, "{channel:?}");
    }
}

#[test]
fn the_selector_fits_the_width_of_the_section() {
    let harness = section_harness(ToneCurves::default());

    let blue = curve_channel_label(CurveChannel::Blue, false);

    assert!(harness.get_by_label(&blue).rect().right() <= 248.0);
}

#[test]
fn a_point_added_on_the_graph_goes_to_the_channel_shown() {
    let mut harness = section_harness(ToneCurves::default());

    show(&mut harness, CurveChannel::Red);
    drag(&mut harness, [0.5, 0.7], [0.5, 0.7]);

    let curves = harness.state();
    assert!(is_shown(&harness, CurveChannel::Red));
    assert_eq!(curves.red.points().len(), 3);
    assert!(curves.rgb.is_identity() && curves.green.is_identity() && curves.blue.is_identity());
}

#[test]
fn the_graph_shows_the_points_of_the_channel_chosen() {
    let mut harness = section_harness(raised_on(CurveChannel::Blue));

    show(&mut harness, CurveChannel::Blue);
    drag(&mut harness, [0.5, 0.7], [0.5, 0.3]);

    let blue = harness.state().blue.points();
    assert_eq!(blue.len(), 3);
    assert!((blue[1][1] - 0.3).abs() < 0.01, "{blue:?}");
}

#[test]
fn a_channel_with_a_curve_away_from_the_identity_is_marked() {
    let harness = section_harness(raised_on(CurveChannel::Green));

    for channel in CurveChannel::ALL {
        let is_green = channel == CurveChannel::Green;
        let marked = curve_channel_label(channel, true);
        let is_marked = harness.query_by_label(&marked).is_some();
        assert_eq!(is_marked, is_green, "{channel:?}");
    }
}

#[test]
fn reset_curve_is_disabled_while_the_curve_shown_is_the_identity() {
    let harness = section_harness(raised_on(CurveChannel::Green));

    let reset = harness.get_by_label(RESET_CURVE_LABEL);

    assert!(reset.accesskit_node().is_disabled());
}

#[test]
fn reset_curve_returns_the_channel_shown_alone_to_the_identity() {
    let both = raised_on(CurveChannel::Rgb).with(CurveChannel::Green, raised_middle());
    let mut harness = section_harness(both);

    show(&mut harness, CurveChannel::Green);
    harness.get_by_label(RESET_CURVE_LABEL).click();
    harness.run();

    assert!(harness.state().green.is_identity());
    assert_eq!(harness.state().rgb, raised_middle());
}
