use egui::{Align2, CornerRadius, Frame, Margin, vec2};

use super::theme::{color, space};

const RADIUS: u8 = 6;
/// How far up a pill floats to leave its place to another one.
pub const PILL_STACKING_STEP: f32 = 44.0;

/// A raised strip floating over the bottom of `over`: showing it moves
/// nothing else.
pub fn floating_pill<Shown>(
    ui: &egui::Ui,
    (name, over): (&str, egui::Rect),
    add_contents: impl FnOnce(&mut egui::Ui) -> Shown,
) -> Shown {
    egui::Area::new(ui.id().with(name))
        .order(egui::Order::Foreground)
        .pivot(Align2::CENTER_BOTTOM)
        .fixed_pos(over.center_bottom() - vec2(0.0, space::L))
        .show(ui.ctx(), |ui| {
            Frame::new()
                .fill(color::RAISED)
                .corner_radius(CornerRadius::same(RADIUS))
                .inner_margin(Margin::symmetric(space::M as i8, space::S as i8))
                .show(ui, add_contents)
                .inner
        })
        .inner
}
