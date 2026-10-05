use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

use crate::update::domain::release_notes::RELEASE_NOTES;

const TALLEST_NOTES: f32 = 280.0;

/// The release notes of this build, scrolling once taller than their place.
pub fn release_notes_text(ui: &mut egui::Ui, layout: &mut CommonMarkCache) {
    egui::ScrollArea::vertical()
        .max_height(TALLEST_NOTES)
        .auto_shrink([false, true])
        .show(ui, |ui| {
            CommonMarkViewer::new().show(ui, layout, RELEASE_NOTES);
        });
}
