use egui::{
    Align, Key, KeyboardShortcut, Layout, Modifiers, Rect, Sense, Stroke, StrokeKind, Vec2,
    WidgetInfo, WidgetType, vec2,
};

use crate::design::ui::theme::{CONTROL_RADIUS, color, space};
use crate::library::domain::grid_step::GridStep;
use crate::photo::domain::photo_name::photo_name;

use super::photo_thumbnail::{
    PhotoThumbnail, keep_selection_visible, paint_cell_ring, paint_name, paint_thumbnail,
    thumbnail_label,
};

pub const PHOTO_GRID_WIDTH: f32 = 440.0;
const PICTURE_SIZE: Vec2 = vec2(132.0, 88.0);
const NAME_HEIGHT: f32 = 20.0;
const CELL_SIZE: Vec2 = vec2(PICTURE_SIZE.x, PICTURE_SIZE.y + NAME_HEIGHT);
const CELL_GAP: f32 = space::S;
const HEADER_HEIGHT: f32 = 24.0;
/// Room for one cell.
pub const NARROWEST_PHOTO_GRID: f32 = CELL_SIZE.x;

pub const REMOVE_FROM_SERIES_LABEL: &str = "Remove from series";
pub const MOVE_TO_TRASH_LABEL: &str = "Move to Trash…";
pub const TRASH_REJECTED_LABEL: &str = "Trash rejected…";
const SELECT_ALL_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::A);

pub struct GridShown<'a> {
    pub photos: &'a [PhotoThumbnail<'a>],
    pub selected: Option<usize>,
    /// Whether each photo is in the selection, in the same order.
    pub is_in_selection: &'a [bool],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridIntent {
    /// Selects that photo alone.
    Select(usize),
    /// Adds the photo to the selection or takes it out.
    Toggle(usize),
    /// Selects every photo from the selected one to this one.
    ExtendTo(usize),
    SelectAll,
    /// Develops that photo.
    Open(usize),
    /// Takes the selection out of the series.
    RemoveSelection,
    /// Takes out of the series the selection when this photo is in it, else this photo.
    RemoveFromSeries(usize),
    /// Asks to move the selection to the Trash.
    TrashSelection,
    /// Asks to move to the Trash the selection when this photo is in it, else this photo.
    MoveToTrash(usize),
    /// Asks to move every rejected photo of the series to the Trash.
    TrashRejected,
}

#[derive(Clone, Copy)]
enum CellState {
    Selected,
    InSelection,
    Alone,
}

fn columns_across(width: f32) -> usize {
    let columns = ((width + CELL_GAP) / (CELL_SIZE.x + CELL_GAP)).floor();
    (columns as usize).max(1)
}

fn grid_cell(ui: &mut egui::Ui, photo: &PhotoThumbnail<'_>, state: CellState) -> egui::Response {
    let (cell, response) = ui.allocate_exact_size(CELL_SIZE, Sense::click());
    let is_selected = matches!(state, CellState::Selected);
    response.widget_info(|| {
        let label = thumbnail_label(photo);
        let is_in_selection = !matches!(state, CellState::Alone);
        WidgetInfo::selected(WidgetType::SelectableLabel, true, is_in_selection, label)
    });
    let picture = Rect::from_min_size(cell.min, PICTURE_SIZE);
    paint_thumbnail(ui, picture, photo);
    paint_cell_ring(ui.painter(), picture, is_selected, response.hovered());
    if matches!(state, CellState::InSelection) {
        let ring = Stroke::new(1.0, color::ACCENT);
        ui.painter()
            .rect_stroke(picture, CONTROL_RADIUS, ring, StrokeKind::Inside);
    }
    let name_color = match state {
        CellState::Alone => color::TEXT_MUTED,
        CellState::Selected | CellState::InSelection => color::TEXT,
    };
    let name_area = Rect::from_min_max(picture.left_bottom(), cell.max);
    paint_name(ui, name_area, photo_name(photo.path), name_color);
    response
}

fn cell_menu(ui: &mut egui::Ui, index: usize) -> Option<GridIntent> {
    if ui.button(REMOVE_FROM_SERIES_LABEL).clicked() {
        return Some(GridIntent::RemoveFromSeries(index));
    }
    let trashed = ui.button(MOVE_TO_TRASH_LABEL).clicked();
    trashed.then_some(GridIntent::MoveToTrash(index))
}

fn clicked_cell_intent(ui: &egui::Ui, cell: &egui::Response, index: usize) -> Option<GridIntent> {
    if cell.double_clicked() {
        return Some(GridIntent::Open(index));
    }
    if !cell.clicked() {
        return None;
    }
    let modifiers = ui.input(|input| input.modifiers);
    Some(match (modifiers.command, modifiers.shift) {
        (true, _) => GridIntent::Toggle(index),
        (false, true) => GridIntent::ExtendTo(index),
        (false, false) => GridIntent::Select(index),
    })
}

/// Arrows move in the grid, `Enter` develops the selected photo, `Cmd+A`
/// selects everything, unless a focused control wants the keys for itself.
fn key_intent(ui: &mut egui::Ui, shown: &GridShown<'_>, columns: usize) -> Option<GridIntent> {
    if ui.memory(|memory| memory.focused().is_some()) {
        return None;
    }
    if ui.input_mut(|input| input.consume_shortcut(&SELECT_ALL_SHORTCUT)) {
        return Some(GridIntent::SelectAll);
    }
    // The key a Mac keyboard calls delete is Backspace; fn+delete is Delete.
    let is_delete_pressed_with = |modifiers: Modifiers| {
        [Key::Backspace, Key::Delete]
            .into_iter()
            .any(|key| ui.input_mut(|input| input.consume_key(modifiers, key)))
    };
    if is_delete_pressed_with(Modifiers::COMMAND) {
        return Some(GridIntent::TrashSelection);
    }
    if is_delete_pressed_with(Modifiers::NONE) {
        return Some(GridIntent::RemoveSelection);
    }
    let selected = shown.selected?;
    if ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Enter)) {
        return Some(GridIntent::Open(selected));
    }
    let steps = [
        (Key::ArrowLeft, GridStep::Previous),
        (Key::ArrowRight, GridStep::Next),
        (Key::ArrowUp, GridStep::Above),
        (Key::ArrowDown, GridStep::Below),
    ];
    steps
        .into_iter()
        .find(|(key, _)| ui.input_mut(|input| input.consume_key(Modifiers::NONE, *key)))
        .map(|(_, step)| GridIntent::Select(step.from(selected, shown.photos.len(), columns)))
}

/// The line above the grid. Returns whether the user asked to trash the rejected photos.
pub fn grid_header(ui: &mut egui::Ui, has_rejected_photos: bool) -> bool {
    let size = vec2(ui.available_width(), HEADER_HEIGHT);
    ui.allocate_ui_with_layout(size, Layout::right_to_left(Align::Center), |ui| {
        let button = egui::Button::new(TRASH_REJECTED_LABEL).frame_when_inactive(false);
        ui.add_enabled(has_rejected_photos, button).clicked()
    })
    .inner
}

/// The photos as rows of thumbnails. Returns what the user asked for this frame, if anything.
pub fn photo_grid(ui: &mut egui::Ui, shown: &GridShown<'_>) -> Option<GridIntent> {
    let columns = columns_across(ui.available_width());
    let mut intent = key_intent(ui, shown, columns);
    let state_of = |index: usize| match shown.selected {
        Some(selected) if selected == index => CellState::Selected,
        Some(_) if shown.is_in_selection.get(index) == Some(&true) => CellState::InSelection,
        _ => CellState::Alone,
    };

    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(CELL_GAP, CELL_GAP);
            let mut selected_cell = None;
            for (row, photos_of_row) in shown.photos.chunks(columns).enumerate() {
                ui.horizontal(|ui| {
                    for (column, photo) in photos_of_row.iter().enumerate() {
                        let index = row * columns + column;
                        let cell = grid_cell(ui, photo, state_of(index));
                        intent = clicked_cell_intent(ui, &cell, index).or(intent);
                        cell.context_menu(|ui| intent = cell_menu(ui, index).or(intent));
                        if shown.selected == Some(index) {
                            selected_cell = Some(cell);
                        }
                    }
                });
            }
            keep_selection_visible(ui, shown.selected, selected_cell.as_ref());
        });
    intent
}
