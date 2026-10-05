use egui::{Pos2, Rect, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use ziv::develop::domain::aspect_ratio::RatioLock;
use ziv::develop::domain::framing::{CropFrame, Framing, Turn};
use ziv::develop::ui::crop_canvas::{
    CANVAS_MARGIN, CROP_FRAME_LABEL, CropCanvasShown, crop_canvas,
};
use ziv::develop::ui::crop_section::CropTools;
use ziv::photo::domain::picture_region::PictureRegion;

const PICTURE: [u32; 2] = [400, 200];

struct Shown {
    framing: Framing,
    tools: CropTools,
    is_level_drawn: bool,
    available: Rect,
    rendered: Option<(PictureRegion, [u32; 2])>,
    is_done: bool,
}

fn canvas(framing: Framing) -> Harness<'static, Shown> {
    let shown = Shown {
        framing,
        tools: CropTools::default(),
        is_level_drawn: false,
        available: Rect::NOTHING,
        rendered: None,
        is_done: false,
    };
    let mut harness = Harness::builder()
        .with_size(vec2(600.0, 400.0))
        .with_step_dt(0.1)
        .build_ui_state(
            |ui, shown: &mut Shown| {
                shown.available = ui.available_rect_before_wrap();
                let canvas = CropCanvasShown {
                    picture: PICTURE,
                    framing: shown.framing,
                    tools: shown.tools,
                };
                let output = crop_canvas(ui, &canvas, |region, size| {
                    shown.rendered = Some((region, size));
                    egui::TextureId::default()
                });
                shown.framing = output.framing;
                shown.is_done |= output.is_done;
                shown.is_level_drawn |= output.is_level_drawn;
            },
            shown,
        );
    harness.run();
    harness
}

/// Where a point of the picture, in pixels, is on screen: the picture is
/// shown at 100 % in the middle of the canvas.
fn on_screen(harness: &Harness<'static, Shown>, point: [f32; 2]) -> Pos2 {
    let area = harness.state().available.shrink(CANVAS_MARGIN);
    let picture = vec2(PICTURE[0] as f32, PICTURE[1] as f32);
    area.min + ((area.size() - picture) / 2.0).floor() + vec2(point[0], point[1])
}

fn drag(harness: &mut Harness<'static, Shown>, from: [f32; 2], to: [f32; 2]) {
    let [from, to] = [from, to].map(|point| on_screen(harness, point));
    harness.drag_at(from);
    harness.run();
    harness.hover_at(from.lerp(to, 0.5));
    harness.run();
    harness.hover_at(to);
    harness.run();
    harness.drop_at(to);
    harness.run();
}

fn middle_half() -> Framing {
    Framing {
        frame: CropFrame {
            centre: [0.5, 0.5],
            size: [0.5, 0.5],
        },
        ..Framing::default()
    }
}

fn assert_frame(harness: &Harness<'static, Shown>, centre: [f32; 2], size: [f32; 2]) {
    let frame = harness.state().framing.frame;
    let is_near = |actual: [f32; 2], expected: [f32; 2]| {
        (actual[0] - expected[0]).abs() < 0.01 && (actual[1] - expected[1]).abs() < 0.01
    };
    assert!(
        is_near(frame.centre, centre) && is_near(frame.size, size),
        "{frame:?} is not centre {centre:?}, size {size:?}"
    );
}

#[test]
fn the_whole_picture_is_shown_at_fit_under_a_named_frame() {
    let harness = canvas(middle_half());

    assert!(harness.query_by_label(CROP_FRAME_LABEL).is_some());
    assert_eq!(
        harness.state().rendered,
        Some((PictureRegion::WHOLE, PICTURE))
    );
}

#[test]
fn dragging_a_corner_handle_moves_its_two_sides() {
    let mut harness = canvas(Framing::default());

    drag(&mut harness, [0.0, 0.0], [100.0, 50.0]);

    assert_frame(&harness, [0.625, 0.625], [0.75, 0.75]);
}

#[test]
fn dragging_a_side_handle_moves_that_side_only() {
    let mut harness = canvas(Framing::default());

    drag(&mut harness, [400.0, 100.0], [300.0, 20.0]);

    assert_frame(&harness, [0.375, 0.5], [0.75, 1.0]);
}

#[test]
fn dragging_inside_the_frame_moves_it_without_resizing_it() {
    let mut harness = canvas(middle_half());

    drag(&mut harness, [200.0, 100.0], [240.0, 80.0]);

    assert_frame(&harness, [0.6, 0.4], [0.5, 0.5]);
}

#[test]
fn the_frame_never_leaves_the_picture() {
    let mut harness = canvas(middle_half());

    drag(&mut harness, [300.0, 150.0], [520.0, 290.0]);
    assert_frame(&harness, [0.625, 0.625], [0.75, 0.75]);

    drag(&mut harness, [200.0, 100.0], [-150.0, 100.0]);
    assert_frame(&harness, [0.375, 0.625], [0.75, 0.75]);
}

fn small_frame() -> Framing {
    Framing {
        frame: CropFrame {
            centre: [0.5, 0.5],
            size: [0.25, 0.5],
        },
        ..Framing::default()
    }
}

#[test]
fn dragging_outside_the_frame_turns_the_picture_with_the_pointer() {
    let mut harness = canvas(small_frame());

    drag(&mut harness, [300.0, 100.0], [300.0, 200.0]);

    let turned = harness.state().framing;
    assert!((turned.angle - 45.0).abs() < 0.5, "{}", turned.angle);
    assert_eq!(turned.frame, small_frame().frame);
}

#[test]
fn a_turn_stops_at_the_end_of_the_range_and_shrinks_a_frame_that_would_leave_the_picture() {
    let mut harness = canvas(middle_half());

    drag(&mut harness, [350.0, 100.0], [200.0, 10.0]);

    let turned = harness.state().framing;
    assert_eq!(turned.angle, -45.0);
    assert_frame(&harness, [0.5, 0.5], turned.frame.size);
    assert!(turned.frame.size[0] < 0.5);
}

#[test]
fn a_turned_picture_is_rendered_whole_around_its_upright_frame() {
    let harness = canvas(small_frame().straightened(PICTURE, 30.0));

    let (region, _) = harness.state().rendered.expect("the picture was rendered");
    assert!(region.across[1] < 0.0, "{region:?} is not turned");
    let centre = region.at([0.5, 0.5]);
    assert!((centre[0] - 0.5).abs() < 0.01 && (centre[1] - 0.5).abs() < 0.01);
}

#[test]
fn a_double_click_inside_the_frame_asks_to_leave() {
    let mut harness = canvas(middle_half());
    let inside = on_screen(&harness, [200.0, 100.0]);

    harness.hover_at(inside);
    harness.run();
    for pressed in [true, false, true, false] {
        harness.event(egui::Event::PointerButton {
            pos: inside,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
        harness.step();
    }
    harness.run();

    assert!(harness.state().is_done);
    assert_eq!(harness.state().framing, middle_half());
}

#[test]
fn a_locked_ratio_is_kept_by_a_corner_and_by_a_side_handle() {
    let mut harness = canvas(middle_half());
    harness.state_mut().tools.ratio = RatioLock::Custom;

    drag(&mut harness, [300.0, 150.0], [340.0, 150.0]);
    assert_frame(&harness, [0.55, 0.55], [0.6, 0.6]);

    drag(&mut harness, [340.0, 110.0], [300.0, 30.0]);
    assert_frame(&harness, [0.5, 0.55], [0.5, 0.5]);
}

fn level_armed(framing: Framing) -> Harness<'static, Shown> {
    let mut harness = canvas(framing);
    harness.state_mut().tools.is_level_armed = true;
    harness.run();
    harness
}

#[test]
fn a_line_of_the_level_tool_turns_the_picture_until_the_line_is_level() {
    let mut harness = level_armed(small_frame());

    drag(&mut harness, [100.0, 100.0], [300.0, 120.0]);

    let levelled = harness.state().framing;
    let tilt = (20.0_f32 / 200.0).atan().to_degrees();
    assert!((levelled.angle + tilt).abs() < 0.05, "{}", levelled.angle);
    assert_eq!(levelled.frame, small_frame().frame);
    assert!(harness.state().is_level_drawn);
}

#[test]
fn a_line_nearer_to_upright_makes_it_upright() {
    let mut harness = level_armed(small_frame());

    drag(&mut harness, [200.0, 20.0], [210.0, 180.0]);

    let tilt = (10.0_f32 / 160.0).atan().to_degrees();
    let angle = harness.state().framing.angle;
    assert!((angle - tilt).abs() < 0.05, "{angle}");
}

#[test]
fn a_line_too_short_changes_nothing_and_the_armed_tool_leaves_the_frame_alone() {
    let mut harness = level_armed(middle_half());

    drag(&mut harness, [300.0, 150.0], [302.0, 151.0]);

    assert_eq!(harness.state().framing, middle_half());
}

#[test]
fn a_handle_of_a_mirrored_photo_moves_the_side_it_is_seen_on() {
    let mut harness = canvas(Framing {
        turn: Turn::default().flipped_horizontally(),
        ..Framing::default()
    });

    drag(&mut harness, [400.0, 100.0], [300.0, 100.0]);

    assert_frame(&harness, [0.625, 0.5], [0.75, 1.0]);
}

#[test]
fn a_photo_turned_right_is_shown_on_its_side_with_its_frame() {
    let harness = canvas(Framing {
        turn: Turn::default().turned_right(),
        ..Framing::default()
    });

    let (region, size) = harness.state().rendered.unwrap();

    let height = (harness.state().available.height() - 2.0 * CANVAS_MARGIN) as u32;
    assert_eq!(size, [height / 2, height]);
    assert_eq!(region.at([0.0, 0.0]), [0.0, 1.0]);
}
