pub const EMPTY_LIBRARY_LABEL: &str = "No photos yet";

pub fn empty_state(ui: &mut egui::Ui) {
    ui.centered_and_justified(|ui| {
        ui.label(EMPTY_LIBRARY_LABEL);
    });
}
