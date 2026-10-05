use egui::RichText;

use crate::design::ui::primary_button::primary_button;
use crate::design::ui::theme::{color, medium, space, type_size};

pub const CONFIRM_TRASH_LABEL: &str = "Move to Trash";
pub const CANCEL_TRASH_LABEL: &str = "Cancel";
const CONSEQUENCE: &str =
    "The files leave their folder, with their edits. They stay in the Trash until it is emptied.";
const WIDTH: f32 = 340.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrashConfirmationIntent {
    Confirm,
    Cancel,
}

/// "Move 3 photos to the Trash?"
pub fn trash_question(photo_count: usize) -> String {
    match photo_count {
        1 => "Move 1 photo to the Trash?".to_owned(),
        count => format!("Move {count} photos to the Trash?"),
    }
}

/// Asks before `photo_count` photos go to the Trash. Returns what the user answered this frame.
pub fn trash_confirmation(
    ui: &mut egui::Ui,
    photo_count: usize,
) -> Option<TrashConfirmationIntent> {
    ui.set_width(WIDTH);
    ui.label(RichText::new(trash_question(photo_count)).font(medium(type_size::TITLE)));
    ui.add_space(space::XS);
    ui.label(RichText::new(CONSEQUENCE).color(color::TEXT_MUTED));
    ui.add_space(space::M);
    ui.horizontal(|ui| {
        if primary_button(ui, CONFIRM_TRASH_LABEL).clicked() {
            return Some(TrashConfirmationIntent::Confirm);
        }
        let cancelled = ui.button(CANCEL_TRASH_LABEL).clicked();
        cancelled.then_some(TrashConfirmationIntent::Cancel)
    })
    .inner
}
