use egui::{Align, Layout, vec2};

use crate::design::ui::theme::{color, regular, type_size};
use crate::photo::domain::shooting_data::ShootingData;

const LINE_HEIGHT: f32 = 16.0;

/// The shooting data spread over the width. Its place is kept when there is none.
pub fn shooting_data_line(ui: &mut egui::Ui, shooting_data: &ShootingData) {
    let font = regular(type_size::CAPTION);
    let labels = shooting_data.labels();
    let width_of = |label: &String| {
        let laid_out = ui
            .painter()
            .layout_no_wrap(label.clone(), font.clone(), color::TEXT_MUTED);
        laid_out.size().x
    };
    let written: f32 = labels.iter().map(width_of).sum();
    let gaps = labels.len().saturating_sub(1).max(1) as f32;
    let size = vec2(ui.available_width(), LINE_HEIGHT);
    ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
        ui.set_min_size(size);
        ui.spacing_mut().item_spacing.x = ((size.x - written) / gaps).max(0.0);
        for label in labels {
            let text = egui::RichText::new(label)
                .font(font.clone())
                .color(color::TEXT_MUTED);
            ui.add(egui::Label::new(text).selectable(false));
        }
    });
}
