use egui::RichText;

use crate::design::ui::adjustment_slider::{AdjustmentSlider, Track, TrackScale};
use crate::design::ui::theme::{color, regular, type_size};
use crate::develop::domain::edit::{FULL_INTENSITY, INTENSITY_RANGE};

pub const ENHANCE_LABEL: &str = "Enhance";
pub const INTENSITY_LABEL: &str = "Intensity";
const ENHANCE_HINT: &str = "Removes noise and strengthens detail.";

const INTENSITY: AdjustmentSlider = AdjustmentSlider {
    label: INTENSITY_LABEL,
    range: INTENSITY_RANGE,
    default: FULL_INTENSITY,
    step: 1.0,
    decimals: 0,
    scale: TrackScale::Linear,
    track: Track::AccentFill,
};

pub struct DetailShown {
    pub is_enhanced: bool,
    /// An enhancement runs, of this photo or another: Enhance waits for it.
    pub is_enhancing: bool,
    pub intensity: f32,
}

pub struct DetailOutput {
    pub intensity: f32,
    pub is_enhancement_asked: bool,
}

/// The Intensity of an enhanced photo, or Enhance on one that is not.
pub fn detail_section(ui: &mut egui::Ui, shown: &DetailShown) -> DetailOutput {
    if shown.is_enhanced {
        return DetailOutput {
            intensity: INTENSITY.show(ui, shown.intensity),
            is_enhancement_asked: false,
        };
    }
    let enhance = ui.add_enabled(!shown.is_enhancing, egui::Button::new(ENHANCE_LABEL));
    let hint = RichText::new(ENHANCE_HINT)
        .font(regular(type_size::CAPTION))
        .color(color::TEXT_MUTED);
    ui.label(hint);
    DetailOutput {
        intensity: shown.intensity,
        is_enhancement_asked: enhance.clicked(),
    }
}
