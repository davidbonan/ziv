use egui::RichText;

use crate::design::ui::theme::{color, regular, type_size};
use crate::update::domain::version::Version;

pub fn running_version_label(running: Version) -> String {
    format!("ziv {running}")
}

/// The running version, as a quiet button that opens the Updates dialog.
pub fn running_version_button(ui: &mut egui::Ui, running: Version) -> egui::Response {
    let text = RichText::new(running_version_label(running))
        .font(regular(type_size::CAPTION))
        .color(color::TEXT_MUTED);
    ui.add(egui::Button::new(text).frame_when_inactive(false))
}
