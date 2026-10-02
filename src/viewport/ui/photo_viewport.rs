use egui::{
    Color32, CursorIcon, Key, KeyboardShortcut, Modifiers, Rect, Sense, Vec2, WidgetInfo,
    WidgetType, pos2,
};

use crate::viewport::domain::view::{Placement, View, Viewport};

pub const PHOTO_VIEWPORT_LABEL: &str = "Photo";

const FIT_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Num0);
const ACTUAL_SIZE_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::Num1);
/// Zoom factor is `exp(scrolled points × this)`: about ×1.3 per wheel notch.
const SCROLL_ZOOM_SPEED: f32 = 0.005;

pub struct PhotoViewportOutput {
    /// The view after this frame's gestures.
    pub view: View,
    /// Where the photo was painted.
    pub photo_rect: Rect,
    /// Where the whole photo lies, the part scrolled out of sight included.
    pub whole_photo_rect: Rect,
}

/// Shows the photo through `view` and applies zoom and pan gestures to it.
/// `display_texture` returns the texture rendered for a placement.
pub fn photo_viewport(
    ui: &mut egui::Ui,
    photo_size: [u32; 2],
    view: View,
    display_texture: impl FnOnce(&Placement) -> egui::TextureId,
) -> PhotoViewportOutput {
    let pixels_per_point = ui.pixels_per_point();
    let area = ui.available_rect_before_wrap();
    let response = ui.allocate_rect(area, Sense::click_and_drag());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Image, true, PHOTO_VIEWPORT_LABEL));

    let viewport = Viewport {
        photo: photo_size,
        size: (area.size() * pixels_per_point).into(),
    };
    let in_viewport_pixels =
        |position: egui::Pos2| <[f32; 2]>::from((position - area.min) * pixels_per_point);

    let mut view = view;
    // Not `hover_pos`: a mask being edited over the photo takes the hover.
    let pointer_over = ui
        .input(|input| input.pointer.hover_pos())
        .filter(|_| response.contains_pointer());
    if let Some(pointer) = pointer_over {
        let (pinch, scrolled) = ui.input(|input| (input.zoom_delta(), input.smooth_scroll_delta.y));
        let factor = pinch * (scrolled * SCROLL_ZOOM_SPEED).exp();
        if factor != 1.0 {
            view = view.zoomed_around(&viewport, in_viewport_pixels(pointer), factor);
        }
    }
    if response.clicked() {
        let clicked = response.interact_pointer_pos().unwrap_or(area.center());
        view = view.toggled_at(&viewport, in_viewport_pixels(clicked));
    }
    if response.dragged() {
        view = view.panned_by(&viewport, (response.drag_delta() * pixels_per_point).into());
    }
    if ui.input_mut(|input| input.consume_shortcut(&FIT_SHORTCUT)) {
        view = View::fit();
    }
    if ui.input_mut(|input| input.consume_shortcut(&ACTUAL_SIZE_SHORTCUT)) {
        view = view.at_actual_size(&viewport);
    }
    let view = view.constrained_to(&viewport);
    if !view.is_fit() {
        response.on_hover_cursor(CursorIcon::Grab);
    }

    let placement = view.placement(&viewport);
    let photo_rect = Rect::from_min_size(
        area.min + Vec2::from(placement.screen_min) / pixels_per_point,
        Vec2::from(placement.screen_size) / pixels_per_point,
    );
    let whole_texture = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    ui.painter().image(
        display_texture(&placement),
        photo_rect,
        whole_texture,
        Color32::WHITE,
    );

    let points_per_photo_pixel = placement.scale / pixels_per_point;
    let whole_photo_rect = Rect::from_min_size(
        photo_rect.min - Vec2::from(placement.photo_min) * points_per_photo_pixel,
        Vec2::new(photo_size[0] as f32, photo_size[1] as f32) * points_per_photo_pixel,
    );
    PhotoViewportOutput {
        view,
        photo_rect,
        whole_photo_rect,
    }
}
