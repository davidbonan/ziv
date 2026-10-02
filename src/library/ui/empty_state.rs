use egui::{Align, Layout, Rect, RichText, UiBuilder, vec2};

use crate::design::ui::theme::{color, medium, space, type_size};

pub const EMPTY_LIBRARY_LABEL: &str = "Drop a folder or photos here";
pub const OPEN_BUTTON_LABEL: &str = "Open…";

const BLOCK_HEIGHT: f32 = 72.0;

/// Returns whether the user asked to open photos.
pub fn empty_state(ui: &mut egui::Ui) -> bool {
    let area = ui.available_rect_before_wrap();
    let block = Rect::from_center_size(area.center(), vec2(area.width(), BLOCK_HEIGHT));
    let builder = UiBuilder::new()
        .max_rect(block)
        .layout(Layout::top_down(Align::Center));
    ui.scope_builder(builder, |ui| {
        let invitation = RichText::new(EMPTY_LIBRARY_LABEL)
            .font(medium(type_size::TITLE))
            .color(color::TEXT);
        ui.label(invitation);
        ui.add_space(space::M);
        ui.button(OPEN_BUTTON_LABEL).clicked()
    })
    .inner
}
