use egui::RichText;

use crate::design::ui::floating_pill::floating_pill;
use crate::design::ui::theme::color;
use crate::develop::domain::preset::Preset;
use crate::develop::domain::zone::{PersonPart, ZoneTool};
use crate::models::ui::download_label::download_label;
use crate::zones::application::detection::DetectionAsked;
use crate::zones::application::zone_detector::DetectionStep;

/// What a detection of `asked` is doing; before its first step it is
/// already detecting.
pub fn detection_label(asked: &DetectionAsked, step: Option<DetectionStep>) -> String {
    if let Some(DetectionStep::Downloading { model, received }) = step {
        return download_label(model, received);
    }
    let detecting = match asked {
        DetectionAsked::Zone(ZoneTool::Subject) | DetectionAsked::Preset(Preset::SubjectPop) => {
            "Detecting the subject…"
        }
        DetectionAsked::Zone(ZoneTool::Background) => "Detecting the background…",
        DetectionAsked::Zone(ZoneTool::Sky)
        | DetectionAsked::Preset(Preset::EnhancedSky | Preset::DramaticSky | Preset::GoldenSky) => {
            "Detecting the sky…"
        }
        DetectionAsked::Preset(Preset::BrightEyes) => "Detecting the eyes…",
        DetectionAsked::Preset(Preset::SkinGlow) => "Detecting the skin…",
        DetectionAsked::Preset(Preset::LipColor) => "Detecting the lips…",
        DetectionAsked::Preset(Preset::HairShine) => "Detecting the hair…",
        DetectionAsked::Preset(Preset::WhiterTeeth) => "Detecting the teeth…",
        DetectionAsked::Persons => "Looking for people…",
        DetectionAsked::PersonParts { .. } => "Masking the chosen parts…",
    };
    detecting.to_owned()
}

/// What to tell the user when nothing of what was `asked` is in the photo.
pub fn nothing_found_label(asked: &DetectionAsked) -> String {
    match asked {
        DetectionAsked::Zone(ZoneTool::Subject | ZoneTool::Background)
        | DetectionAsked::Preset(Preset::SubjectPop) => "No subject found in this photo".to_owned(),
        DetectionAsked::Zone(ZoneTool::Sky)
        | DetectionAsked::Preset(Preset::EnhancedSky | Preset::DramaticSky | Preset::GoldenSky) => {
            "No sky found in this photo".to_owned()
        }
        DetectionAsked::Preset(Preset::BrightEyes) => "No eyes found in this photo".to_owned(),
        DetectionAsked::Preset(Preset::SkinGlow) => "No skin found in this photo".to_owned(),
        DetectionAsked::Preset(Preset::LipColor) => "No lips found in this photo".to_owned(),
        DetectionAsked::Preset(Preset::HairShine) => "No hair found in this photo".to_owned(),
        DetectionAsked::Preset(Preset::WhiterTeeth) => "No teeth found in this photo".to_owned(),
        DetectionAsked::Persons => "No person found in this photo".to_owned(),
        DetectionAsked::PersonParts { parts, .. } => parts_not_found_label(parts),
    }
}

/// Names the parts that were found on none of the chosen persons.
pub fn parts_not_found_label(missing: &[PersonPart]) -> String {
    let names: Vec<String> = missing
        .iter()
        .map(|part| part.name().to_lowercase())
        .collect();
    format!("Not found on the chosen persons: {}", names.join(", "))
}

/// Floats over the bottom of `over` while a detection runs.
pub fn detection_status(ui: &egui::Ui, over: egui::Rect, label: &str) {
    floating_pill(ui, ("detection status", over), |ui| {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(RichText::new(label).color(color::TEXT));
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zones::domain::zone_models::SUBJECT_MODEL;

    #[test]
    fn download_says_how_far_it_is() {
        let step = DetectionStep::Downloading {
            model: &SUBJECT_MODEL,
            received: 120_400_000,
        };

        assert_eq!(
            detection_label(&DetectionAsked::Zone(ZoneTool::Subject), Some(step)),
            "Downloading the subject model, 120 of 224 MB"
        );
    }

    #[test]
    fn detection_names_what_was_asked() {
        let background = DetectionAsked::Zone(ZoneTool::Background);

        assert_eq!(
            detection_label(&background, Some(DetectionStep::Detecting)),
            "Detecting the background…"
        );
        assert_eq!(
            detection_label(&DetectionAsked::Persons, None),
            "Looking for people…"
        );
    }

    #[test]
    fn preset_says_which_zone_it_looks_for_and_which_it_did_not_find() {
        let whiter_teeth = DetectionAsked::Preset(Preset::WhiterTeeth);

        assert_eq!(detection_label(&whiter_teeth, None), "Detecting the teeth…");
        assert_eq!(
            nothing_found_label(&whiter_teeth),
            "No teeth found in this photo"
        );
    }

    #[test]
    fn parts_found_on_nobody_are_named() {
        assert_eq!(
            parts_not_found_label(&[PersonPart::Eyes, PersonPart::Lips]),
            "Not found on the chosen persons: eyes, lips"
        );
    }
}
