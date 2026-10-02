use std::path::Path;

use egui::RichText;

use crate::design::ui::floating_pill::floating_pill;
use crate::design::ui::theme::{color, space};
use crate::enhance::application::enhancer::EnhancementStep;
use crate::models::ui::download_label::download_label;
use crate::photo::domain::photo_name::photo_name;

pub const STOP_ENHANCEMENT_LABEL: &str = "Cancel enhancement";

/// What the enhancement of `photo` is doing; before its first step it is
/// already enhancing.
pub fn enhancement_label(photo: &Path, step: Option<EnhancementStep>) -> String {
    let share = match step {
        Some(EnhancementStep::Downloading { model, received }) => {
            return download_label(model, received);
        }
        Some(EnhancementStep::Enhancing { share }) => share,
        None => 0.0,
    };
    let percent = (share * 100.0).floor();
    format!("Enhancing {}, {percent} %", photo_name(photo))
}

/// Floats over the bottom of `over`. Returns whether Cancel was asked for.
pub fn enhancement_progress(ui: &egui::Ui, over: egui::Rect, label: &str) -> bool {
    floating_pill(ui, ("enhancement progress", over), |ui| {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(RichText::new(label).color(color::TEXT));
            ui.add_space(space::S);
            ui.button(STOP_ENHANCEMENT_LABEL).clicked()
        })
        .inner
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enhance::domain::enhancement_model::ENHANCEMENT_MODEL;

    const PHOTO: &str = "/shoot/DSC07070.ARW";

    #[test]
    fn progress_names_the_photo_and_how_far_it_is() {
        let step = EnhancementStep::Enhancing { share: 0.428 };

        assert_eq!(
            enhancement_label(Path::new(PHOTO), Some(step)),
            "Enhancing DSC07070.ARW, 42 %"
        );
        assert_eq!(
            enhancement_label(Path::new(PHOTO), None),
            "Enhancing DSC07070.ARW, 0 %"
        );
    }

    #[test]
    fn download_says_how_far_it_is() {
        let step = EnhancementStep::Downloading {
            model: &ENHANCEMENT_MODEL,
            received: 40_500_000,
        };

        assert_eq!(
            enhancement_label(Path::new(PHOTO), Some(step)),
            "Downloading the enhancement model, 40 of 130 MB"
        );
    }
}
