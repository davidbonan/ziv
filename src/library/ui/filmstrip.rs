use egui::{Key, Modifiers, Sense, Vec2, WidgetInfo, WidgetType, vec2};

use crate::design::ui::theme::{color, regular, type_size};

use super::photo_thumbnail::{
    PhotoThumbnail, keep_selection_visible, paint_cell_ring, paint_thumbnail, thumbnail_label,
};

pub const FILMSTRIP_HEIGHT: f32 = 96.0;
const CELL_SIZE: Vec2 = vec2(108.0, 72.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilmstripIntent {
    Select(usize),
    SelectPrevious,
    SelectNext,
}

/// Where the selected photo is in the series: "3 / 148".
pub fn position_label(selected: Option<usize>, count: usize) -> String {
    match selected {
        Some(index) => format!("{} / {count}", index + 1),
        None => count.to_string(),
    }
}

fn thumbnail_cell(
    ui: &mut egui::Ui,
    photo: &PhotoThumbnail<'_>,
    is_selected: bool,
) -> egui::Response {
    let (cell, response) = ui.allocate_exact_size(CELL_SIZE, Sense::click());
    response.widget_info(|| {
        let label = thumbnail_label(photo);
        WidgetInfo::selected(WidgetType::SelectableLabel, true, is_selected, label)
    });
    paint_thumbnail(ui, cell, photo);
    paint_cell_ring(ui.painter(), cell, is_selected, response.hovered());
    response
}

/// Arrow keys browse the photos unless a focused control wants them for itself.
fn arrow_key_intent(ui: &mut egui::Ui) -> Option<FilmstripIntent> {
    if ui.memory(|memory| memory.focused().is_some()) {
        return None;
    }
    if ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::ArrowLeft)) {
        return Some(FilmstripIntent::SelectPrevious);
    }
    ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::ArrowRight))
        .then_some(FilmstripIntent::SelectNext)
}

/// The band of thumbnails. Returns what the user asked for this frame, if anything.
pub fn filmstrip(
    ui: &mut egui::Ui,
    photos: &[PhotoThumbnail<'_>],
    selected: Option<usize>,
) -> Option<FilmstripIntent> {
    let mut intent = arrow_key_intent(ui);

    ui.horizontal_centered(|ui| {
        let position = egui::RichText::new(position_label(selected, photos.len()))
            .font(regular(type_size::CAPTION))
            .color(color::TEXT_MUTED);
        ui.label(position);
        egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                let mut selected_cell = None;
                for (index, photo) in photos.iter().enumerate() {
                    let is_selected = selected == Some(index);
                    let cell = thumbnail_cell(ui, photo, is_selected);
                    if cell.clicked() {
                        intent = Some(FilmstripIntent::Select(index));
                    }
                    if is_selected {
                        selected_cell = Some(cell);
                    }
                }
                keep_selection_visible(ui, selected, selected_cell.as_ref());
            });
        });
    });
    intent
}
