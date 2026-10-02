use egui::{Key, Pos2, Rect, pos2, vec2};
use egui_kittest::Harness;
use ziv::develop::domain::brush::{Brush, Stroke};
use ziv::develop::domain::linear_gradient::LinearGradient;
use ziv::develop::domain::mask::{Mask, MaskKind, MaskShape, PhotoPoint};
use ziv::develop::ui::mask_canvas::{MaskCanvasState, PhotoOnScreen, mask_canvas};
use ziv::develop::ui::masks_section::MaskSelection;

const WIDTH: f32 = 400.0;
const HEIGHT: f32 = 300.0;

fn canvas(state: MaskCanvasState) -> Harness<'static, MaskCanvasState> {
    let mut harness = Harness::builder()
        .with_size(vec2(WIDTH, HEIGHT))
        .build_ui_state(
            |ui, state: &mut MaskCanvasState| {
                let whole = Rect::from_min_size(Pos2::ZERO, vec2(WIDTH, HEIGHT));
                let photo = PhotoOnScreen {
                    area: whole,
                    whole_photo: whole,
                };
                *state = mask_canvas(ui, &photo, state.clone());
            },
            state,
        );
    harness.run();
    harness
}

fn drag(harness: &mut Harness<'static, MaskCanvasState>, from: Pos2, to: Pos2) {
    harness.drag_at(from);
    harness.run();
    harness.hover_at(from.lerp(to, 0.5));
    harness.run();
    harness.hover_at(to);
    harness.run();
    harness.drop_at(to);
    harness.run();
}

fn armed_with(tool: MaskKind) -> MaskCanvasState {
    MaskCanvasState {
        masks: Vec::new(),
        selection: MaskSelection {
            armed_tool: Some(tool),
            ..MaskSelection::default()
        },
    }
}

fn armed() -> MaskCanvasState {
    armed_with(MaskKind::LinearGradient)
}

fn gradient() -> LinearGradient {
    LinearGradient {
        full: [0.25, 0.25],
        none: [0.75, 0.25],
    }
}

fn selected_gradient() -> MaskCanvasState {
    MaskCanvasState {
        masks: vec![Mask::of(MaskShape::LinearGradient(gradient()))],
        selection: MaskSelection::of(0),
    }
}

fn on_screen(point: PhotoPoint) -> Pos2 {
    pos2(point[0] * WIDTH, point[1] * WIDTH)
}

fn only_gradient(harness: &Harness<'static, MaskCanvasState>) -> LinearGradient {
    let [mask] = harness.state().masks.as_slice() else {
        panic!("expected one mask, found {:?}", harness.state().masks);
    };
    let MaskShape::LinearGradient(gradient) = mask.shape else {
        panic!("expected a linear gradient, found {:?}", mask.shape);
    };
    gradient
}

fn assert_near(actual: PhotoPoint, expected: PhotoPoint) {
    let is_near = (actual[0] - expected[0]).abs() < 0.01 && (actual[1] - expected[1]).abs() < 0.01;
    assert!(is_near, "{actual:?} is not near {expected:?}");
}

#[test]
fn armed_tool_draws_a_gradient_from_where_the_drag_starts_to_where_it_ends() {
    let mut harness = canvas(armed());

    drag(&mut harness, pos2(100.0, 60.0), pos2(300.0, 240.0));

    let drawn = only_gradient(&harness);
    assert_near(drawn.full, [0.25, 0.15]);
    assert_near(drawn.none, [0.75, 0.6]);
    assert_eq!(harness.state().selection, MaskSelection::of(0));
}

#[test]
fn dragging_an_end_handle_moves_that_end() {
    let mut harness = canvas(selected_gradient());

    drag(&mut harness, on_screen(gradient().none), pos2(320.0, 200.0));

    let reshaped = only_gradient(&harness);
    assert_eq!(reshaped.full, gradient().full);
    assert_near(reshaped.none, [0.8, 0.5]);
}

#[test]
fn dragging_the_middle_handle_moves_the_whole_gradient() {
    let mut harness = canvas(selected_gradient());

    drag(&mut harness, on_screen([0.5, 0.25]), on_screen([0.5, 0.5]));

    let moved = only_gradient(&harness);
    assert_near(moved.full, [0.25, 0.5]);
    assert_near(moved.none, [0.75, 0.5]);
}

#[test]
fn dragging_away_from_the_handles_changes_nothing() {
    let mut harness = canvas(selected_gradient());

    drag(&mut harness, pos2(40.0, 250.0), pos2(200.0, 280.0));

    assert_eq!(*harness.state(), selected_gradient());
}

#[test]
fn holding_space_leaves_the_drag_to_the_photo() {
    let mut harness = canvas(armed());

    harness.key_down(Key::Space);
    drag(&mut harness, pos2(100.0, 60.0), pos2(300.0, 240.0));

    assert_eq!(*harness.state(), armed());
}

#[test]
fn nothing_is_drawn_without_a_tool_or_a_selected_mask() {
    let idle = MaskCanvasState {
        selection: MaskSelection::default(),
        ..selected_gradient()
    };
    let mut harness = canvas(idle.clone());

    drag(&mut harness, on_screen(gradient().none), pos2(320.0, 200.0));

    assert_eq!(*harness.state(), idle);
}

#[test]
fn radial_tool_draws_an_ellipse_around_where_the_drag_starts() {
    let mut harness = canvas(armed_with(MaskKind::RadialGradient));

    drag(&mut harness, pos2(200.0, 120.0), pos2(280.0, 160.0));

    let [mask] = harness.state().masks.as_slice() else {
        panic!("expected one mask, found {:?}", harness.state().masks);
    };
    let MaskShape::RadialGradient(drawn) = mask.shape else {
        panic!("expected a radial gradient, found {:?}", mask.shape);
    };
    assert_near(drawn.centre, [0.5, 0.3]);
    assert_near(drawn.radii, [0.2, 0.1]);
}

#[test]
fn rectangle_tool_draws_between_the_two_ends_of_the_drag() {
    let mut harness = canvas(armed_with(MaskKind::Rectangle));

    drag(&mut harness, pos2(80.0, 40.0), pos2(240.0, 160.0));

    let [mask] = harness.state().masks.as_slice() else {
        panic!("expected one mask, found {:?}", harness.state().masks);
    };
    let MaskShape::Rectangle(drawn) = &mask.shape else {
        panic!("expected a rectangle, found {:?}", mask.shape);
    };
    assert_near(drawn.centre, [0.4, 0.25]);
    assert_near(drawn.half_size, [0.2, 0.15]);
}

fn click_at(harness: &mut Harness<'static, MaskCanvasState>, position: Pos2) {
    harness.hover_at(position);
    harness.run();
    harness.drag_at(position);
    harness.run();
    harness.drop_at(position);
    harness.run();
}

fn polygon_corners(harness: &Harness<'static, MaskCanvasState>) -> Vec<PhotoPoint> {
    let [mask] = harness.state().masks.as_slice() else {
        panic!("expected one mask, found {:?}", harness.state().masks);
    };
    let MaskShape::Polygon(polygon) = &mask.shape else {
        panic!("expected a polygon, found {:?}", mask.shape);
    };
    polygon.corners.clone()
}

const TRIANGLE: [Pos2; 3] = [pos2(100.0, 50.0), pos2(300.0, 60.0), pos2(200.0, 250.0)];

#[test]
fn polygon_tool_adds_a_corner_a_click_and_enter_closes_it() {
    let mut harness = canvas(armed_with(MaskKind::Polygon));

    for corner in TRIANGLE {
        click_at(&mut harness, corner);
    }
    assert!(harness.state().masks.is_empty());
    harness.key_press(Key::Enter);
    harness.run();

    let corners = polygon_corners(&harness);
    assert_eq!(corners.len(), 3);
    assert_near(corners[2], [0.5, 0.625]);
    assert_eq!(harness.state().selection, MaskSelection::of(0));
}

#[test]
fn clicking_the_first_corner_closes_the_polygon() {
    let mut harness = canvas(armed_with(MaskKind::Polygon));

    for corner in TRIANGLE {
        click_at(&mut harness, corner);
    }
    click_at(&mut harness, TRIANGLE[0] + vec2(3.0, -2.0));

    assert_eq!(polygon_corners(&harness).len(), 3);
}

#[test]
fn two_corners_close_nothing() {
    let mut harness = canvas(armed_with(MaskKind::Polygon));

    click_at(&mut harness, TRIANGLE[0]);
    click_at(&mut harness, TRIANGLE[1]);
    harness.key_press(Key::Enter);
    harness.run();

    assert!(harness.state().masks.is_empty());
}

fn strokes(harness: &Harness<'static, MaskCanvasState>) -> Vec<Stroke> {
    let [mask] = harness.state().masks.as_slice() else {
        panic!("expected one mask, found {:?}", harness.state().masks);
    };
    let MaskShape::Brush(brush) = &mask.shape else {
        panic!("expected a brush mask, found {:?}", mask.shape);
    };
    brush.strokes.clone()
}

#[test]
fn brush_tool_paints_a_stroke_along_the_drag_with_the_brush_as_set() {
    let mut harness = canvas(armed_with(MaskKind::Brush));

    drag(&mut harness, pos2(100.0, 60.0), pos2(300.0, 240.0));

    let [stroke] = strokes(&harness).try_into().unwrap();
    assert_near(stroke.points[0], [0.25, 0.15]);
    assert_near(*stroke.points.last().unwrap(), [0.75, 0.6]);
    assert_eq!(stroke.radius, Brush::default().radius());
    assert!(!stroke.is_erasing);
    assert_eq!(harness.state().selection.selected, Some(0));
    assert_eq!(harness.state().selection.armed_tool, None);
}

#[test]
fn each_drag_on_a_selected_brush_mask_adds_a_stroke() {
    let mut harness = canvas(armed_with(MaskKind::Brush));

    drag(&mut harness, pos2(100.0, 60.0), pos2(300.0, 60.0));
    drag(&mut harness, pos2(100.0, 200.0), pos2(300.0, 200.0));

    assert_eq!(strokes(&harness).len(), 2);
}

#[test]
fn click_without_a_drag_paints_a_dab() {
    let mut harness = canvas(armed_with(MaskKind::Brush));

    click_at(&mut harness, pos2(200.0, 150.0));

    let [dab] = strokes(&harness).try_into().unwrap();
    assert_eq!(dab.points.len(), 1);
    assert_near(dab.points[0], [0.5, 0.375]);
}

#[test]
fn stroke_made_with_alt_held_erases() {
    let mut harness = canvas(armed_with(MaskKind::Brush));
    drag(&mut harness, pos2(100.0, 60.0), pos2(300.0, 60.0));

    harness.event(egui::Event::ModifiersChanged(egui::Modifiers::ALT));
    drag(&mut harness, pos2(200.0, 20.0), pos2(200.0, 120.0));

    assert!(strokes(&harness)[1].is_erasing);
}

#[test]
fn bracket_keys_change_the_size_of_the_brush() {
    let mut harness = canvas(armed_with(MaskKind::Brush));

    harness.key_press(Key::CloseBracket);
    harness.run();
    assert_eq!(harness.state().selection.brush.size, 22.0);

    harness.key_press(Key::OpenBracket);
    harness.key_press(Key::OpenBracket);
    harness.run();
    assert_eq!(harness.state().selection.brush.size, 18.0);
}
