use egui::{Pos2, Rect, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use ziv::develop::domain::tone_curve::{CLOSEST_POINTS, CurvePoint, ToneCurve};
use ziv::develop::ui::tone_curve_graph::{GraphShown, TONE_CURVE_GRAPH_LABEL, tone_curve_graph};
use ziv::histogram::domain::histogram::Histogram;

// Two clicks one frame apart must stay within egui's double-click delay.
const DOUBLE_CLICKABLE_STEP_SECONDS: f32 = 0.1;

fn shown<'a>(curve: &'a ToneCurve, histogram: Option<&'a Histogram>) -> GraphShown<'a> {
    GraphShown {
        curve,
        ink: egui::Color32::WHITE,
        histogram,
    }
}

fn graph_harness_over(
    histogram: Option<Histogram>,
    curve: ToneCurve,
) -> Harness<'static, ToneCurve> {
    let mut harness = Harness::builder()
        .with_size(vec2(216.0, 260.0))
        .with_step_dt(DOUBLE_CLICKABLE_STEP_SECONDS)
        .build_ui_state(
            move |ui, curve: &mut ToneCurve| {
                *curve = tone_curve_graph(ui, &shown(curve, histogram.as_ref()));
            },
            curve,
        );
    harness.run();
    harness
}

fn graph_harness(curve: ToneCurve) -> Harness<'static, ToneCurve> {
    graph_harness_over(None, curve)
}

fn curve(points: &[CurvePoint]) -> ToneCurve {
    ToneCurve::try_from(points.to_vec()).unwrap()
}

fn three_points() -> ToneCurve {
    curve(&[[0.0, 0.0], [0.5, 0.5], [1.0, 1.0]])
}

fn graph<State>(harness: &Harness<'static, State>) -> Rect {
    harness.get_by_label(TONE_CURVE_GRAPH_LABEL).rect()
}

fn on_screen<State>(harness: &Harness<'static, State>, [tone, result]: CurvePoint) -> Pos2 {
    let graph = graph(harness);
    graph.left_bottom() + vec2(tone * graph.width(), -result * graph.height())
}

fn press<State>(harness: &mut Harness<'static, State>, at: Pos2) {
    harness.hover_at(at);
    harness.run();
    harness.drag_at(at);
    harness.run();
}

pub fn drag<State>(harness: &mut Harness<'static, State>, from: CurvePoint, to: CurvePoint) {
    let (from, to) = (on_screen(harness, from), on_screen(harness, to));
    press(harness, from);
    harness.hover_at(from.lerp(to, 0.5));
    harness.run();
    harness.hover_at(to);
    harness.run();
    harness.drop_at(to);
    harness.run();
}

fn double_click(harness: &mut Harness<'static, ToneCurve>, at: CurvePoint) {
    let position = on_screen(harness, at);
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

fn assert_points(harness: &Harness<'static, ToneCurve>, expected: &[CurvePoint]) {
    let points = harness.state().points();
    let is_close = points.len() == expected.len()
        && points
            .iter()
            .flatten()
            .zip(expected.iter().flatten())
            .all(|(actual, expected)| (actual - expected).abs() < 0.01);
    assert!(is_close, "{points:?} is not {expected:?}");
}

#[test]
fn the_graph_is_a_square_found_by_its_name() {
    let harness = graph_harness(ToneCurve::default());

    let graph = graph(&harness);

    assert_eq!(graph.width(), graph.height());
    assert_eq!(graph.width(), 200.0);
}

#[test]
fn the_graph_over_a_histogram_keeps_its_square_and_its_points() {
    let grey_and_white = [[100, 100, 100, 255], [255, 255, 255, 255]];
    let histogram = Histogram::of_display_pixels(grey_and_white.as_flattened());
    let mut harness = graph_harness_over(Some(histogram), ToneCurve::default());

    drag(&mut harness, [0.25, 0.4], [0.25, 0.4]);

    assert_eq!(graph(&harness).size(), vec2(200.0, 200.0));
    assert_points(&harness, &[[0.0, 0.0], [0.25, 0.4], [1.0, 1.0]]);
}

#[test]
fn a_click_away_from_a_point_adds_a_point_there() {
    let mut harness = graph_harness(ToneCurve::default());

    drag(&mut harness, [0.25, 0.4], [0.25, 0.4]);

    assert_points(&harness, &[[0.0, 0.0], [0.25, 0.4], [1.0, 1.0]]);
}

#[test]
fn the_press_that_adds_a_point_can_drag_it() {
    let mut harness = graph_harness(ToneCurve::default());

    drag(&mut harness, [0.25, 0.4], [0.6, 0.8]);

    assert_points(&harness, &[[0.0, 0.0], [0.6, 0.8], [1.0, 1.0]]);
}

#[test]
fn a_drag_moves_the_point_it_starts_on() {
    let mut harness = graph_harness(three_points());

    drag(&mut harness, [0.5, 0.5], [0.3, 0.7]);

    assert_points(&harness, &[[0.0, 0.0], [0.3, 0.7], [1.0, 1.0]]);
}

#[test]
fn a_dragged_point_stops_at_its_neighbour() {
    let two_inner_points = curve(&[[0.0, 0.0], [0.3, 0.3], [0.7, 0.7], [1.0, 1.0]]);
    let mut harness = graph_harness(two_inner_points);

    drag(&mut harness, [0.3, 0.3], [0.9, 0.5]);

    let stopped = [0.7 - CLOSEST_POINTS, 0.5];
    assert_points(&harness, &[[0.0, 0.0], stopped, [0.7, 0.7], [1.0, 1.0]]);
}

#[test]
fn an_end_point_is_dragged_along_its_edge_and_lifts_black() {
    let mut harness = graph_harness(ToneCurve::default());

    drag(&mut harness, [0.0, 0.0], [0.0, 0.25]);

    assert_points(&harness, &[[0.0, 0.25], [1.0, 1.0]]);
}

#[test]
fn a_double_click_removes_a_point_but_not_an_end_point() {
    let mut harness = graph_harness(three_points());

    double_click(&mut harness, [0.5, 0.5]);
    assert_points(&harness, &[[0.0, 0.0], [1.0, 1.0]]);

    double_click(&mut harness, [1.0, 1.0]);
    assert_points(&harness, &[[0.0, 0.0], [1.0, 1.0]]);
}

#[test]
fn a_point_dragged_out_of_the_graph_is_removed() {
    let mut harness = graph_harness(three_points());

    drag(&mut harness, [0.5, 0.5], [0.5, -0.2]);

    assert_points(&harness, &[[0.0, 0.0], [1.0, 1.0]]);
}

#[test]
fn an_end_point_dragged_out_of_the_graph_stays_on_its_edge() {
    let mut harness = graph_harness(ToneCurve::default());

    drag(&mut harness, [1.0, 1.0], [1.0, 1.2]);

    assert_points(&harness, &[[0.0, 0.0], [1.0, 1.0]]);
}
