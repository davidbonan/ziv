use std::path::Path;

use egui::{Align, Layout, Rect, RichText, UiBuilder, vec2};

use crate::design::ui::theme::{color, medium, space, type_size};
use crate::photo::domain::decode_error::DecodeError;
use crate::photo::domain::photo_name::photo_name;

const STATUS_HEIGHT: f32 = 56.0;

pub fn loading_label(path: &Path) -> String {
    format!("Loading {}…", photo_name(path))
}

pub fn failed_label(path: &Path) -> String {
    format!("{} could not be opened", photo_name(path))
}

fn centered_status(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    let area = ui.available_rect_before_wrap();
    let block = Rect::from_center_size(area.center(), vec2(area.width(), STATUS_HEIGHT));
    let builder = UiBuilder::new()
        .max_rect(block)
        .layout(Layout::top_down(Align::Center));
    ui.scope_builder(builder, add_contents);
}

pub fn photo_loading(ui: &mut egui::Ui, path: &Path) {
    centered_status(ui, |ui| {
        ui.add(egui::Spinner::new().size(18.0).color(color::TEXT_MUTED));
        ui.add_space(space::XS);
        ui.label(RichText::new(loading_label(path)).color(color::TEXT_MUTED));
    });
}

pub fn photo_failed(ui: &mut egui::Ui, path: &Path, error: &DecodeError) {
    centered_status(ui, |ui| {
        let title = RichText::new(failed_label(path))
            .font(medium(type_size::TITLE))
            .color(color::TEXT);
        ui.label(title);
        ui.add_space(space::XS);
        ui.label(RichText::new(error.to_string()).color(color::TEXT_MUTED));
    });
}
