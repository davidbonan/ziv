use ziv::design::ui::theme::apply_theme;

/// Applies ziv's theme to the harness on its first frame. Fonts only take
/// effect on the next frame: returns `false` when the caller must not draw yet.
pub fn is_themed(ui: &egui::Ui) -> bool {
    let flag = egui::Id::new("ziv theme applied");
    if ui.data(|data| data.get_temp::<bool>(flag).is_some()) {
        return true;
    }
    apply_theme(ui.ctx());
    ui.data_mut(|data| data.insert_temp(flag, true));
    ui.ctx().request_repaint();
    false
}
