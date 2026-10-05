use egui::{Align, Layout, RichText, vec2};

use crate::design::ui::theme::{color, medium, regular, space, type_size};
use crate::photo::domain::photo_details::PhotoDetails;
use crate::photo::domain::shooting_data::ShootingData;

pub const PHOTO_DETAILS_HEIGHT: f32 = 84.0;
const NAME_LINE_HEIGHT: f32 = 20.0;
const CAPTION_LINE_HEIGHT: f32 = 16.0;

pub struct PhotoDetailsShown<'a> {
    pub name: &'a str,
    pub shooting_data: &'a ShootingData,
    pub details: &'a PhotoDetails,
    /// How many photos the selection holds.
    pub selection_size: usize,
}

/// "5 selected"; nothing for a photo selected alone.
pub fn selection_label(selection_size: usize) -> Option<String> {
    (selection_size > 1).then(|| format!("{selection_size} selected"))
}

// Its place is kept when it holds nothing.
fn line(ui: &mut egui::Ui, height: f32, add_contents: impl FnOnce(&mut egui::Ui)) {
    let size = vec2(ui.available_width(), height);
    ui.allocate_ui_with_layout(size, Layout::left_to_right(Align::Center), |ui| {
        ui.set_min_size(size);
        ui.spacing_mut().item_spacing.x = space::L;
        add_contents(ui);
    });
}

fn caption_line(ui: &mut egui::Ui, labels: Vec<String>) {
    line(ui, CAPTION_LINE_HEIGHT, |ui| {
        for label in labels {
            let text = RichText::new(label)
                .font(regular(type_size::CAPTION))
                .color(color::TEXT_MUTED);
            ui.add(egui::Label::new(text).selectable(false));
        }
    });
}

/// The name of the photo, how it was shot, then what its file says.
pub fn photo_details_lines(ui: &mut egui::Ui, shown: &PhotoDetailsShown<'_>) {
    ui.spacing_mut().item_spacing.y = space::XS;
    line(ui, NAME_LINE_HEIGHT, |ui| {
        ui.label(RichText::new(shown.name).font(medium(type_size::BODY)));
        if let Some(count) = selection_label(shown.selection_size) {
            ui.label(RichText::new(count).color(color::ACCENT));
        }
    });
    caption_line(ui, shown.shooting_data.labels());
    caption_line(ui, shown.details.labels());
}
