use egui::{Align2, CornerRadius, Frame, Margin, RichText, vec2};

use crate::design::ui::theme::{color, medium, space, type_size};

pub const BEFORE_BADGE_LABEL: &str = "Before";

const RADIUS: u8 = 6;

/// Floats over the top-left corner of `over`: showing it moves nothing else.
pub fn before_badge(ui: &egui::Ui, over: egui::Rect) {
    egui::Area::new(ui.id().with("before badge"))
        .order(egui::Order::Foreground)
        .pivot(Align2::LEFT_TOP)
        .fixed_pos(over.left_top() + vec2(space::M, space::M))
        .interactable(false)
        .show(ui.ctx(), |ui| {
            Frame::new()
                .fill(color::RAISED)
                .corner_radius(CornerRadius::same(RADIUS))
                .inner_margin(Margin::symmetric(space::S as i8, space::XS as i8))
                .show(ui, |ui| {
                    let text = RichText::new(BEFORE_BADGE_LABEL)
                        .font(medium(type_size::CAPTION))
                        .color(color::ACCENT);
                    ui.label(text)
                });
        });
}
