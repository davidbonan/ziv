use egui::{RichText, vec2};

use crate::design::ui::theme::{color, medium, space, type_size};
use crate::develop::domain::edit::Edit;
use crate::develop::domain::preset::{Preset, PresetGroup};

pub const PRESETS_GROUP_LABEL: &str = "Presets";

const COLUMNS: usize = 2;

pub struct PresetsShown<'a> {
    pub edit: &'a Edit,
    /// A detection runs: no preset can be applied.
    pub is_detecting: bool,
}

/// The presets by group, two buttons a row; the preset asked for this frame.
pub fn presets_section(ui: &mut egui::Ui, shown: &PresetsShown<'_>) -> Option<Preset> {
    let mut asked = None;
    for group in PresetGroup::ALL {
        let title = RichText::new(group.name())
            .font(medium(type_size::CAPTION))
            .color(color::TEXT_MUTED);
        ui.label(title);
        asked = asked.or(preset_grid(ui, shown, group.presets()));
        ui.add_space(space::S);
    }
    asked
}

fn preset_grid(ui: &mut egui::Ui, shown: &PresetsShown<'_>, presets: &[Preset]) -> Option<Preset> {
    let gap = ui.spacing().item_spacing.x;
    let width = (ui.available_width() - gap) / COLUMNS as f32;
    let size = vec2(width, ui.spacing().interact_size.y);
    let mut asked = None;
    for row in presets.chunks(COLUMNS) {
        ui.horizontal(|ui| {
            for preset in row {
                let can_apply = !shown.is_detecting && shown.edit.has_room_for(*preset);
                let button = egui::Button::new(preset.name()).min_size(size);
                if ui.add_enabled(can_apply, button).clicked() {
                    asked = Some(*preset);
                }
            }
        });
    }
    asked
}
