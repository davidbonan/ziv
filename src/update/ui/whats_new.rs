use egui::RichText;
use egui_commonmark::CommonMarkCache;

use super::release_notes_text::release_notes_text;
use super::updates_dialog::{CLOSE_LABEL, DIALOG_WIDTH};
use crate::design::ui::primary_button::primary_button;
use crate::design::ui::theme::{medium, space, type_size};

pub const WHATS_NEW_TITLE: &str = "What's new";

/// The release notes, shown after an update. Returns whether Close was asked for.
pub fn whats_new(ui: &mut egui::Ui, notes_layout: &mut CommonMarkCache) -> bool {
    ui.set_width(DIALOG_WIDTH);
    ui.label(RichText::new(WHATS_NEW_TITLE).font(medium(type_size::TITLE)));
    ui.add_space(space::S);
    release_notes_text(ui, notes_layout);
    ui.add_space(space::M);
    primary_button(ui, CLOSE_LABEL).clicked()
}
