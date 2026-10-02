use std::path::Path;

use egui::{
    Color32, Key, Modifiers, Rect, Sense, Stroke, StrokeKind, Vec2, WidgetInfo, WidgetType, pos2,
    vec2,
};

use egui::text::{LayoutJob, TextWrapping};

use crate::design::ui::theme::{CONTROL_RADIUS, color, regular, space, type_size};
use crate::photo::domain::photo_name::photo_name;

pub const FILMSTRIP_HEIGHT: f32 = 96.0;
const CELL_SIZE: Vec2 = vec2(108.0, 72.0);
const SELECTION_STROKE_WIDTH: f32 = 2.0;
const NAME_MARGIN: f32 = 6.0;
const EDITED_MARKER_RADIUS: f32 = 2.5;
const EDITED_MARKER_INSET: f32 = 8.0;

pub enum ThumbnailState {
    Loading,
    Ready(egui::TextureHandle),
    Failed,
    /// The file is no longer where the series has it.
    NotFound,
}

pub struct FilmstripPhoto<'a> {
    pub path: &'a Path,
    pub thumbnail: &'a ThumbnailState,
    pub is_edited: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilmstripIntent {
    Select(usize),
    SelectPrevious,
    SelectNext,
}

/// What assistive technology and tests read for a thumbnail.
pub fn thumbnail_label(photo: &FilmstripPhoto<'_>) -> String {
    let name = photo_name(photo.path);
    let state = match photo.thumbnail {
        ThumbnailState::Loading => ", loading",
        ThumbnailState::Ready(_) => "",
        ThumbnailState::Failed => ", could not be opened",
        ThumbnailState::NotFound => ", not found",
    };
    let edited = match photo.is_edited {
        true => ", edited",
        false => "",
    };
    format!("{name}{state}{edited}")
}

/// Where the selected photo is in the series: "3 / 148".
pub fn position_label(selected: Option<usize>, count: usize) -> String {
    match selected {
        Some(index) => format!("{} / {count}", index + 1),
        None => count.to_string(),
    }
}

// Ringed in the dark: it stays visible on a bright picture.
fn paint_edited_marker(painter: &egui::Painter, cell: Rect) {
    let centre = cell.right_bottom() - vec2(EDITED_MARKER_INSET, EDITED_MARKER_INSET);
    painter.circle_filled(centre, EDITED_MARKER_RADIUS + 1.5, color::CANVAS);
    painter.circle_filled(centre, EDITED_MARKER_RADIUS, color::ACCENT);
}

fn largest_rect_with_aspect(inside: Rect, aspect: f32) -> Rect {
    let size = if inside.aspect_ratio() > aspect {
        vec2(inside.height() * aspect, inside.height())
    } else {
        vec2(inside.width(), inside.width() / aspect)
    };
    Rect::from_center_size(inside.center(), size)
}

fn paint_name(ui: &egui::Ui, cell: Rect, text: String, color: Color32) {
    let mut job = LayoutJob::simple_singleline(text, regular(type_size::CAPTION), color);
    job.wrap = TextWrapping::truncate_at_width(cell.width() - 2.0 * NAME_MARGIN);
    let galley = ui.painter().layout_job(job);
    let position = cell.center() - galley.size() / 2.0;
    ui.painter().galley(position, galley, color);
}

fn thumbnail_cell(
    ui: &mut egui::Ui,
    photo: &FilmstripPhoto<'_>,
    is_selected: bool,
) -> egui::Response {
    let (cell, response) = ui.allocate_exact_size(CELL_SIZE, Sense::click());
    response.widget_info(|| {
        let label = thumbnail_label(photo);
        WidgetInfo::selected(WidgetType::SelectableLabel, true, is_selected, label)
    });

    let painter = ui.painter();
    painter.rect_filled(cell, CONTROL_RADIUS, color::CANVAS);
    match photo.thumbnail {
        ThumbnailState::Loading => {
            paint_name(ui, cell, photo_name(photo.path), color::TEXT_DISABLED)
        }
        ThumbnailState::Ready(texture) => {
            let whole_texture = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
            let picture = largest_rect_with_aspect(cell.shrink(space::XS), texture.aspect_ratio());
            painter.image(texture.id(), picture, whole_texture, Color32::WHITE);
            let outline = Stroke::new(1.0, color::PICTURE_OUTLINE);
            painter.rect_stroke(picture, 0.0, outline, StrokeKind::Inside);
        }
        ThumbnailState::Failed | ThumbnailState::NotFound => {
            paint_name(ui, cell, photo_name(photo.path), color::DANGER)
        }
    }
    if photo.is_edited {
        paint_edited_marker(painter, cell);
    }
    if is_selected {
        let stroke = Stroke::new(SELECTION_STROKE_WIDTH, color::ACCENT);
        painter.rect_stroke(cell, CONTROL_RADIUS, stroke, StrokeKind::Inside);
    } else if response.hovered() {
        let stroke = Stroke::new(1.0, color::TRACK);
        painter.rect_stroke(cell, CONTROL_RADIUS, stroke, StrokeKind::Inside);
    }
    response
}

fn keep_selection_visible(
    ui: &egui::Ui,
    selected: Option<usize>,
    selected_cell: Option<&egui::Response>,
) {
    let last_shown_id = ui.id().with("last selection scrolled to");
    let last_shown: Option<usize> = ui.data(|data| data.get_temp(last_shown_id)).flatten();
    if last_shown == selected {
        return;
    }
    if let Some(cell) = selected_cell {
        cell.scroll_to_me(None);
    }
    ui.data_mut(|data| data.insert_temp(last_shown_id, selected));
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
    photos: &[FilmstripPhoto<'_>],
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
