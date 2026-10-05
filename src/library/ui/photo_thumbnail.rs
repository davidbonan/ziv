use std::path::Path;

use egui::text::{LayoutJob, TextWrapping};
use egui::{Align2, Color32, Rect, Stroke, StrokeKind, pos2, vec2};

use crate::design::ui::theme::{CONTROL_RADIUS, color, regular, space, type_size};
use crate::library::domain::mark::Mark;
use crate::photo::domain::photo_name::photo_name;

use super::mark_line::STAR;

const SELECTION_STROKE_WIDTH: f32 = 2.0;
const NAME_MARGIN: f32 = 6.0;
const EDITED_MARKER_RADIUS: f32 = 2.5;
const EDITED_MARKER_INSET: f32 = 8.0;
const REJECTED_VEIL: Color32 = Color32::from_black_alpha(170);
const REJECTED_CROSS_REACH: f32 = 4.0;
const MARK_INSET: f32 = 6.0;

pub enum ThumbnailState {
    Loading,
    Ready(egui::TextureHandle),
    Failed,
    /// The file is no longer where the series has it.
    NotFound,
}

/// A photo of the open series as the filmstrip and the grid show it.
pub struct PhotoThumbnail<'a> {
    pub path: &'a Path,
    pub thumbnail: &'a ThumbnailState,
    pub is_edited: bool,
    pub mark: Mark,
}

/// What assistive technology and tests read for a thumbnail.
pub fn thumbnail_label(photo: &PhotoThumbnail<'_>) -> String {
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
    format!("{name}{state}{edited}{}", photo.mark.spoken())
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

/// `text` on one line in the middle of `area`, cut short when it is wider.
pub fn paint_name(ui: &egui::Ui, area: Rect, text: String, color: Color32) {
    let mut job = LayoutJob::simple_singleline(text, regular(type_size::CAPTION), color);
    job.wrap = TextWrapping::truncate_at_width(area.width() - 2.0 * NAME_MARGIN);
    let galley = ui.painter().layout_job(job);
    let position = area.center() - galley.size() / 2.0;
    ui.painter().galley(position, galley, color);
}

// On a dark plate: stars stay readable on a bright picture.
fn paint_rating(painter: &egui::Painter, cell: Rect, stars: u8) {
    let font = regular(type_size::CAPTION);
    let text = STAR.repeat(usize::from(stars));
    let corner = cell.left_bottom() + vec2(MARK_INSET, -MARK_INSET);
    let galley = painter.layout_no_wrap(text, font, color::TEXT);
    let written = Align2::LEFT_BOTTOM.anchor_size(corner, galley.size());
    painter.rect_filled(written.expand2(vec2(3.0, 1.0)), 3.0, color::CANVAS);
    painter.galley(written.min, galley, color::TEXT);
}

// A rejected photo is dimmed and crossed.
fn paint_rejected(painter: &egui::Painter, cell: Rect) {
    painter.rect_filled(cell, CONTROL_RADIUS, REJECTED_VEIL);
    let centre = cell.left_top() + vec2(MARK_INSET, MARK_INSET) + vec2(4.0, 4.0);
    let stroke = Stroke::new(1.6, color::DANGER);
    for rising in [-1.0, 1.0] {
        let reach = vec2(REJECTED_CROSS_REACH, REJECTED_CROSS_REACH * rising);
        painter.line_segment([centre - reach, centre + reach], stroke);
    }
}

/// The picture of `photo` in `cell`, or its name while there is no picture.
pub fn paint_thumbnail(ui: &egui::Ui, cell: Rect, photo: &PhotoThumbnail<'_>) {
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
    if photo.mark.is_rejected {
        paint_rejected(painter, cell);
    }
    if photo.is_edited {
        paint_edited_marker(painter, cell);
    }
    if photo.mark.rating.stars() > 0 {
        paint_rating(painter, cell, photo.mark.rating.stars());
    }
}

/// The ring of the selected cell, a fainter one under the pointer.
pub fn paint_cell_ring(painter: &egui::Painter, cell: Rect, is_selected: bool, is_hovered: bool) {
    let stroke = match (is_selected, is_hovered) {
        (true, _) => Stroke::new(SELECTION_STROKE_WIDTH, color::ACCENT),
        (false, true) => Stroke::new(1.0, color::TRACK),
        (false, false) => return,
    };
    painter.rect_stroke(cell, CONTROL_RADIUS, stroke, StrokeKind::Inside);
}

/// Scrolls to the selected cell when the selection changed since the last frame.
pub fn keep_selection_visible(
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
