use egui::{Align, Layout, Rect, RichText, UiBuilder, vec2};

use crate::design::ui::theme::{color, space};
use crate::library::domain::series_filter::{SeriesFilter, stars_or_more};

pub const SHOW_ALL_LABEL: &str = "Show all";

const BLOCK_HEIGHT: f32 = 64.0;

/// "No photo at 2 stars or more".
pub fn no_photo_shown_label(filter: SeriesFilter) -> String {
    format!("No photo at {}", stars_or_more(filter.stars()))
}

/// What stands for the photos when the filter shows none. Returns whether the
/// user asked to show them all.
pub fn no_photo_shown(ui: &mut egui::Ui, filter: SeriesFilter) -> bool {
    let area = ui.available_rect_before_wrap();
    let block = Rect::from_center_size(area.center(), vec2(area.width(), BLOCK_HEIGHT));
    let builder = UiBuilder::new()
        .max_rect(block)
        .layout(Layout::top_down(Align::Center));
    ui.scope_builder(builder, |ui| {
        ui.label(RichText::new(no_photo_shown_label(filter)).color(color::TEXT_MUTED));
        ui.add_space(space::S);
        ui.button(SHOW_ALL_LABEL).clicked()
    })
    .inner
}
