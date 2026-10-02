use egui::RichText;

use super::theme::{color, medium, type_size};

/// The one button of a screen filled with the accent: its main action.
pub fn primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let ink = match ui.is_enabled() {
        true => color::ON_ACCENT,
        false => color::TEXT_DISABLED,
    };
    let text = RichText::new(label)
        .font(medium(type_size::BODY))
        .color(ink);
    let fill = match ui.is_enabled() {
        true => color::ACCENT,
        false => color::RAISED,
    };
    ui.add(egui::Button::new(text).fill(fill))
}
