use egui::{Event, Key, Modifiers, MouseWheelUnit, TouchPhase, pos2, vec2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use ziv::viewport::domain::view::{ACTUAL_SIZE, Placement, View, Viewport};
use ziv::viewport::ui::photo_viewport::{PHOTO_VIEWPORT_LABEL, photo_viewport};

const WIDE_PHOTO: [u32; 2] = [4000, 2000];

struct Shown {
    photo: [u32; 2],
    view: View,
    available: egui::Rect,
    photo_rect: egui::Rect,
    rendered: Option<Placement>,
}

impl Shown {
    fn scale(&self) -> f32 {
        self.rendered.expect("the photo was rendered").scale
    }
}

fn viewport_harness(photo: [u32; 2], pixels_per_point: f32) -> Harness<'static, Shown> {
    let shown = Shown {
        photo,
        view: View::fit(),
        available: egui::Rect::NOTHING,
        photo_rect: egui::Rect::NOTHING,
        rendered: None,
    };
    let mut harness = Harness::builder()
        .with_size(vec2(400.0, 300.0))
        .with_pixels_per_point(pixels_per_point)
        .build_ui_state(
            |ui, shown: &mut Shown| {
                shown.available = ui.available_rect_before_wrap();
                let output = photo_viewport(ui, shown.photo, shown.view, |placement| {
                    shown.rendered = Some(*placement);
                    egui::TextureId::default()
                });
                shown.view = output.view;
                shown.photo_rect = output.photo_rect;
            },
            shown,
        );
    harness.run();
    harness
}

fn middle(harness: &Harness<'_, Shown>) -> egui::Pos2 {
    harness.state().available.center()
}

#[test]
fn wide_photo_spans_the_width_and_is_centered() {
    let harness = viewport_harness(WIDE_PHOTO, 1.0);
    let shown = harness.state();

    let width = shown.available.width();
    assert_eq!(shown.photo_rect.size(), vec2(width, width / 2.0));
    assert_eq!(shown.photo_rect.center(), shown.available.center());
}

#[test]
fn tall_photo_spans_the_height() {
    let harness = viewport_harness([2000, 4000], 1.0);
    let shown = harness.state();

    let height = shown.available.height();
    assert_eq!(shown.photo_rect.size(), vec2(height / 2.0, height));
}

#[test]
fn photo_is_rendered_at_the_screen_pixel_density() {
    let harness = viewport_harness(WIDE_PHOTO, 2.0);
    let shown = harness.state();

    let rendered_width = shown.available.width() * 2.0;
    assert_eq!(
        shown.rendered.unwrap().screen_size,
        [rendered_width, rendered_width / 2.0]
    );
    assert_eq!(shown.photo_rect.width(), shown.available.width());
}

#[test]
fn small_photo_keeps_its_size() {
    let harness = viewport_harness([100, 50], 1.0);

    assert_eq!(harness.state().photo_rect.size(), vec2(100.0, 50.0));
}

#[test]
fn scrolling_over_the_photo_zooms_in() {
    let mut harness = viewport_harness(WIDE_PHOTO, 1.0);
    let fit_scale = harness.state().scale();

    harness.hover_at(middle(&harness));
    harness.event(Event::MouseWheel {
        unit: MouseWheelUnit::Point,
        delta: vec2(0.0, 100.0),
        phase: TouchPhase::Move,
        modifiers: Modifiers::NONE,
    });
    harness.run();

    assert!(harness.state().scale() > fit_scale);
    assert!(!harness.state().view.is_fit());
}

#[test]
fn clicking_toggles_between_fit_and_actual_size() {
    let mut harness = viewport_harness(WIDE_PHOTO, 1.0);

    harness.get_by_label(PHOTO_VIEWPORT_LABEL).click();
    harness.run();
    assert_eq!(harness.state().scale(), ACTUAL_SIZE);

    harness.get_by_label(PHOTO_VIEWPORT_LABEL).click();
    harness.run();
    assert!(harness.state().view.is_fit());
}

#[test]
fn keyboard_shortcuts_switch_between_actual_size_and_fit() {
    let mut harness = viewport_harness(WIDE_PHOTO, 1.0);

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Num1);
    harness.run();
    assert_eq!(harness.state().scale(), ACTUAL_SIZE);

    harness.key_press_modifiers(Modifiers::COMMAND, Key::Num0);
    harness.run();
    assert!(harness.state().view.is_fit());
}

#[test]
fn dragging_a_zoomed_photo_pans_it() {
    let mut harness = viewport_harness(WIDE_PHOTO, 1.0);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::Num1);
    harness.run();
    let viewport = Viewport {
        photo: WIDE_PHOTO,
        size: harness.state().available.size().into(),
    };
    let photo_position_under = |harness: &Harness<'_, Shown>, pointer: egui::Pos2| {
        let in_viewport = pointer - harness.state().available.min;
        let [x, y] = harness
            .state()
            .view
            .photo_position_at(&viewport, in_viewport.into());
        pos2(x, y)
    };
    let start = middle(&harness);
    let end = start + vec2(-60.0, 40.0);
    let grabbed = photo_position_under(&harness, start);

    harness.drag_at(start);
    harness.run();
    harness.hover_at(end);
    harness.run();
    harness.drop_at(end);
    harness.run();

    let dropped = photo_position_under(&harness, end);
    assert!(
        (dropped - grabbed).length() < 1.0,
        "grabbed {grabbed:?}, dropped on {dropped:?}"
    );
}
