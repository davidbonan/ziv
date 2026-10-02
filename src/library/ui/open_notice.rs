use egui::{Align2, CornerRadius, Frame, Margin, RichText, vec2};

use crate::design::ui::theme::{color, space};

pub const NO_PHOTO_FOUND_NOTICE: &str = "No photo found in what was opened";

const RADIUS: u8 = 6;
const SHOWN_SECONDS: f64 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OpenNotice {
    pub text: &'static str,
    over_at: f64,
}

impl OpenNotice {
    pub fn shown_at(now: f64, text: &'static str) -> Self {
        Self {
            text,
            over_at: now + SHOWN_SECONDS,
        }
    }

    pub fn seconds_left_at(&self, now: f64) -> f64 {
        (self.over_at - now).max(0.0)
    }

    pub fn is_over_at(&self, now: f64) -> bool {
        now >= self.over_at
    }
}

/// Floats over the bottom of `over`: showing it moves nothing else.
pub fn open_notice(ui: &egui::Ui, over: egui::Rect, notice: &str) {
    egui::Area::new(ui.id().with("open notice"))
        .order(egui::Order::Foreground)
        .pivot(Align2::CENTER_BOTTOM)
        .fixed_pos(over.center_bottom() - vec2(0.0, space::L))
        .interactable(false)
        .show(ui.ctx(), |ui| {
            Frame::new()
                .fill(color::RAISED)
                .corner_radius(CornerRadius::same(RADIUS))
                .inner_margin(Margin::symmetric(space::M as i8, space::S as i8))
                .show(ui, |ui| ui.label(RichText::new(notice).color(color::TEXT)));
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notice_goes_away_by_itself() {
        let notice = OpenNotice::shown_at(10.0, NO_PHOTO_FOUND_NOTICE);

        assert!(!notice.is_over_at(10.0 + SHOWN_SECONDS / 2.0));
        assert!(notice.is_over_at(10.0 + SHOWN_SECONDS));
    }
}
