use egui::RichText;

use super::floating_pill::floating_pill;
use super::theme::color;

const SHOWN_SECONDS: f64 = 4.0;

/// A short message that goes away by itself.
#[derive(Debug, Clone, PartialEq)]
pub struct Notice {
    pub text: String,
    over_at: f64,
}

impl Notice {
    pub fn shown_at(now: f64, text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
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
pub fn notice_toast(ui: &egui::Ui, over: egui::Rect, text: &str) {
    floating_pill(ui, ("notice", over), |ui| {
        ui.label(RichText::new(text).color(color::TEXT));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notice_goes_away_by_itself() {
        let notice = Notice::shown_at(10.0, "No photo found");

        assert!(!notice.is_over_at(10.0 + SHOWN_SECONDS / 2.0));
        assert!(notice.is_over_at(10.0 + SHOWN_SECONDS));
    }
}
