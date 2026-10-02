use std::ops::RangeInclusive;

use egui::{Key, Modifiers};

use crate::design::ui::adjustment_slider::{AdjustmentSlider, TrackScale};
use crate::design::ui::theme::{color, medium, regular, space, type_size};
use crate::develop::domain::adjustments::{ADJUSTMENT_RANGE, Adjustments, EXPOSURE_RANGE};
use crate::develop::domain::brush::{Brush, FLOW_RANGE, SIZE_RANGE};
use crate::develop::domain::edit::Edit;
use crate::develop::domain::mask::MaskKind;
use crate::develop::domain::radial_gradient::FEATHER_RANGE;
use crate::develop::domain::white_balance::{
    KELVIN_RANGE, KELVIN_TINT_RANGE, RELATIVE_RANGE, WhiteBalance,
};
use crate::develop::domain::zone::DetectionTool;
use crate::enhance::ui::detail_section::{DetailShown, detail_section};
use crate::photo::domain::photo_kind::PhotoKind;

use super::masks_section::{
    MASKS_GROUP_LABEL, MaskSelection, MasksIntent, MasksShown, masks_section,
};

pub const TEMPERATURE_LABEL: &str = "Temp";
pub const TINT_LABEL: &str = "Tint";
pub const EXPOSURE_LABEL: &str = "Exposure";
pub const CONTRAST_LABEL: &str = "Contrast";
pub const HIGHLIGHTS_LABEL: &str = "Highlights";
pub const SHADOWS_LABEL: &str = "Shadows";
pub const WHITES_LABEL: &str = "Whites";
pub const BLACKS_LABEL: &str = "Blacks";
pub const VIBRANCE_LABEL: &str = "Vibrance";
pub const SATURATION_LABEL: &str = "Saturation";
pub const PRESENCE_GROUP_LABEL: &str = "Presence";
pub const DETAIL_GROUP_LABEL: &str = "Detail";
pub const PANEL_TITLE: &str = "Develop";
pub const RESET_LABEL: &str = "Reset";
pub const BEFORE_LABEL: &str = "Before";
pub const INVERT_LABEL: &str = "Invert";
pub const DONE_LABEL: &str = "Done";
pub const OVERLAY_LABEL: &str = "Overlay";
pub const FEATHER_LABEL: &str = "Feather";
pub const BRUSH_SIZE_LABEL: &str = "Size";
pub const BRUSH_FLOW_LABEL: &str = "Flow";

const OVERLAY_KEY: Key = Key::O;
pub const WHITE_BALANCE_GROUP_LABEL: &str = "White balance";
pub const TONE_GROUP_LABEL: &str = "Tone";

const KELVIN_STEP: f32 = 50.0;

const EXPOSURE: AdjustmentSlider = AdjustmentSlider {
    label: EXPOSURE_LABEL,
    range: EXPOSURE_RANGE,
    default: 0.0,
    step: 0.1,
    decimals: 2,
    scale: TrackScale::Linear,
};

const fn whole_number_adjustment(
    label: &'static str,
    range: RangeInclusive<f32>,
) -> AdjustmentSlider {
    AdjustmentSlider {
        label,
        range,
        default: 0.0,
        step: 1.0,
        decimals: 0,
        scale: TrackScale::Linear,
    }
}

const FEATHER: AdjustmentSlider = whole_number_adjustment(FEATHER_LABEL, FEATHER_RANGE);
const BRUSH_SIZE: AdjustmentSlider = AdjustmentSlider {
    default: 20.0,
    ..whole_number_adjustment(BRUSH_SIZE_LABEL, SIZE_RANGE)
};
const BRUSH_FLOW: AdjustmentSlider = AdjustmentSlider {
    default: 100.0,
    ..whole_number_adjustment(BRUSH_FLOW_LABEL, FLOW_RANGE)
};
const CONTRAST: AdjustmentSlider = whole_number_adjustment(CONTRAST_LABEL, ADJUSTMENT_RANGE);
const HIGHLIGHTS: AdjustmentSlider = whole_number_adjustment(HIGHLIGHTS_LABEL, ADJUSTMENT_RANGE);
const SHADOWS: AdjustmentSlider = whole_number_adjustment(SHADOWS_LABEL, ADJUSTMENT_RANGE);
const WHITES: AdjustmentSlider = whole_number_adjustment(WHITES_LABEL, ADJUSTMENT_RANGE);
const BLACKS: AdjustmentSlider = whole_number_adjustment(BLACKS_LABEL, ADJUSTMENT_RANGE);
const VIBRANCE: AdjustmentSlider = whole_number_adjustment(VIBRANCE_LABEL, ADJUSTMENT_RANGE);
const SATURATION: AdjustmentSlider = whole_number_adjustment(SATURATION_LABEL, ADJUSTMENT_RANGE);

fn temperature_slider(kind: &PhotoKind) -> AdjustmentSlider {
    match kind {
        PhotoKind::Raw { .. } => AdjustmentSlider {
            step: KELVIN_STEP,
            scale: TrackScale::Reciprocal,
            ..whole_number_adjustment(TEMPERATURE_LABEL, KELVIN_RANGE)
        },
        PhotoKind::StandardImage => whole_number_adjustment(TEMPERATURE_LABEL, RELATIVE_RANGE),
    }
}

fn tint_slider(kind: &PhotoKind) -> AdjustmentSlider {
    match kind {
        PhotoKind::Raw { .. } => whole_number_adjustment(TINT_LABEL, KELVIN_TINT_RANGE),
        PhotoKind::StandardImage => whole_number_adjustment(TINT_LABEL, RELATIVE_RANGE),
    }
}

/// `None` when the sliders are left at the as-shot white balance.
fn white_balance_sliders(
    ui: &mut egui::Ui,
    kind: &PhotoKind,
    chosen: Option<WhiteBalance>,
) -> Option<WhiteBalance> {
    let as_shot = WhiteBalance::as_shot(kind);
    let shown = chosen.unwrap_or(as_shot);
    let temperature = AdjustmentSlider {
        default: as_shot.temperature,
        ..temperature_slider(kind)
    };
    let tint = AdjustmentSlider {
        default: as_shot.tint,
        ..tint_slider(kind)
    };
    let left = WhiteBalance {
        temperature: temperature.show(ui, shown.temperature),
        tint: tint.show(ui, shown.tint),
    };
    (left != as_shot).then_some(left)
}

fn group_title(ui: &mut egui::Ui, title: &str) {
    let text = egui::RichText::new(title)
        .font(regular(type_size::CAPTION))
        .color(color::TEXT_MUTED);
    ui.label(text);
    ui.add_space(space::XS);
}

/// What the panel shows, and what the user made of it this frame.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DevelopPanelState {
    pub edit: Edit,
    pub mask_selection: MaskSelection,
    pub is_before_shown: bool,
    /// A detection runs: the zone tools wait for it.
    pub is_detecting: bool,
    /// The zone tool the user asked for this frame.
    pub asked_detection: Option<DetectionTool>,
    /// The photo has an enhancement: its Intensity can be dosed.
    pub is_enhanced: bool,
    /// An enhancement runs: Enhance waits for it.
    pub is_enhancing: bool,
    /// The user asked for the photo to be enhanced this frame.
    pub is_enhancement_asked: bool,
}

impl DevelopPanelState {
    fn selected_mask(&self) -> Option<usize> {
        self.mask_selection
            .selected
            .filter(|index| *index < self.edit.masks.len())
    }

    fn selecting(self, selected: Option<usize>) -> Self {
        Self {
            mask_selection: self.mask_selection.selecting(selected),
            ..self
        }
    }

    fn without_mask(mut self, removed: usize) -> Self {
        let selected = match self.selected_mask() {
            Some(selected) if selected == removed => None,
            Some(selected) if selected > removed => Some(selected - 1),
            selected => selected,
        };
        self.edit.masks.remove(removed);
        self.selecting(selected)
    }

    fn after(mut self, intent: MasksIntent) -> Self {
        match intent {
            MasksIntent::Arm(tool) => {
                let is_armed = self.mask_selection.armed_tool == Some(tool);
                self.mask_selection = self.mask_selection.arming((!is_armed).then_some(tool));
                self
            }
            MasksIntent::Detect(tool) => {
                self.asked_detection = Some(tool);
                self
            }
            MasksIntent::Select(index) => {
                let is_selected = self.selected_mask() == Some(index);
                self.selecting((!is_selected).then_some(index))
            }
            MasksIntent::ToggleVisibility(index) => {
                self.edit.masks[index].is_hidden = !self.edit.masks[index].is_hidden;
                self
            }
            MasksIntent::Remove(index) => self.without_mask(index),
        }
    }
}

struct HeaderOutput {
    is_reset_asked: bool,
    is_before_toggled: bool,
}

fn panel_header(ui: &mut egui::Ui, state: &DevelopPanelState) -> HeaderOutput {
    let title = egui::RichText::new(PANEL_TITLE)
        .font(medium(type_size::TITLE))
        .color(color::TEXT);
    let is_edited = state.edit != Edit::default();
    let output = ui
        .horizontal(|ui| {
            ui.label(title);
            let to_the_right = egui::Layout::right_to_left(egui::Align::Center);
            ui.with_layout(to_the_right, |ui| {
                let reset = ui.add_enabled(is_edited, egui::Button::new(RESET_LABEL));
                let before = egui::Button::new(BEFORE_LABEL).selected(state.is_before_shown);
                HeaderOutput {
                    is_reset_asked: reset.clicked(),
                    is_before_toggled: ui.add(before).clicked(),
                }
            })
            .inner
        })
        .inner;
    ui.add_space(space::M);
    output
}

/// Says why a photo's edits cannot be changed: what is stored for it is unusable.
pub fn edits_left_alone_warning(ui: &mut egui::Ui, reason: &str) {
    ui.add_space(space::M);
    let warning = format!("The edits file of this photo is left untouched: {reason}.");
    ui.label(
        egui::RichText::new(warning)
            .font(regular(type_size::CAPTION))
            .color(color::DANGER),
    );
}

fn adjusted(ui: &mut egui::Ui, kind: &PhotoKind, adjustments: Adjustments) -> Adjustments {
    let mut adjustments = adjustments;
    group_title(ui, WHITE_BALANCE_GROUP_LABEL);
    adjustments.white_balance = white_balance_sliders(ui, kind, adjustments.white_balance);
    ui.add_space(space::M);
    group_title(ui, TONE_GROUP_LABEL);
    adjustments.exposure = EXPOSURE.show(ui, adjustments.exposure);
    adjustments.contrast = CONTRAST.show(ui, adjustments.contrast);
    adjustments.highlights = HIGHLIGHTS.show(ui, adjustments.highlights);
    adjustments.shadows = SHADOWS.show(ui, adjustments.shadows);
    adjustments.whites = WHITES.show(ui, adjustments.whites);
    adjustments.blacks = BLACKS.show(ui, adjustments.blacks);
    ui.add_space(space::M);
    group_title(ui, PRESENCE_GROUP_LABEL);
    adjustments.vibrance = VIBRANCE.show(ui, adjustments.vibrance);
    adjustments.saturation = SATURATION.show(ui, adjustments.saturation);
    adjustments
}

/// The name of the mask being adjusted and Done. Whether Done was asked.
fn mask_title(ui: &mut egui::Ui, name: &str) -> bool {
    let name = egui::RichText::new(name)
        .font(medium(type_size::BODY))
        .color(color::ACCENT);
    ui.horizontal(|ui| {
        ui.label(name);
        let to_the_right = egui::Layout::right_to_left(egui::Align::Center);
        ui.with_layout(to_the_right, |ui| ui.button(DONE_LABEL).clicked())
            .inner
    })
    .inner
}

struct MaskOptions {
    is_inverted: bool,
    is_overlaid: bool,
}

/// Invert and Overlay, as left this frame.
fn mask_options(ui: &mut egui::Ui, options: MaskOptions) -> MaskOptions {
    let toggled = |ui: &mut egui::Ui, label: &str, is_on: bool| {
        is_on != ui.add(egui::Button::new(label).selected(is_on)).clicked()
    };
    let options = ui
        .horizontal(|ui| MaskOptions {
            is_inverted: toggled(ui, INVERT_LABEL, options.is_inverted),
            is_overlaid: toggled(ui, OVERLAY_LABEL, options.is_overlaid),
        })
        .inner;
    ui.add_space(space::S);
    options
}

fn brush_sliders(ui: &mut egui::Ui, brush: Brush) -> Brush {
    Brush {
        size: BRUSH_SIZE.show(ui, brush.size),
        feather: FEATHER.show(ui, brush.feather),
        flow: BRUSH_FLOW.show(ui, brush.flow),
    }
}

/// The sliders of the selected mask instead of the photo's.
fn mask_adjusted(
    ui: &mut egui::Ui,
    state: DevelopPanelState,
    selected: usize,
) -> DevelopPanelState {
    let mut state = state;
    let name = state.edit.mask_names().swap_remove(selected);
    let is_done = mask_title(ui, &name);
    let is_overlaid = state.mask_selection.overlaid_mask(&state.edit).is_some();
    let shown = MaskOptions {
        is_inverted: state.edit.masks[selected].is_inverted,
        is_overlaid,
    };
    let options = mask_options(ui, shown);
    if options.is_overlaid != is_overlaid {
        state.mask_selection = state.mask_selection.with_overlay_toggled(&state.edit);
    }
    let mask = &mut state.edit.masks[selected];
    mask.is_inverted = options.is_inverted;
    if mask.shape.kind() == MaskKind::Brush {
        state.mask_selection.brush = brush_sliders(ui, state.mask_selection.brush);
        ui.add_space(space::M);
    }
    if let Some(feather) = mask.shape.feather() {
        let slider = AdjustmentSlider {
            default: mask.shape.kind().default_feather(),
            ..FEATHER
        };
        mask.shape = mask.shape.with_feather(slider.show(ui, feather));
        ui.add_space(space::M);
    }
    mask.adjustments = ui
        .push_id(selected, |ui| {
            adjusted(ui, &PhotoKind::StandardImage, mask.adjustments)
        })
        .inner;
    match is_done {
        true => state.selecting(None),
        false => state,
    }
}

/// The sliders of the photo itself, then its Detail section.
fn photo_adjusted(
    ui: &mut egui::Ui,
    kind: &PhotoKind,
    state: DevelopPanelState,
) -> DevelopPanelState {
    let mut state = state;
    state.edit.adjustments = adjusted(ui, kind, state.edit.adjustments);
    ui.add_space(space::M);
    group_title(ui, DETAIL_GROUP_LABEL);
    let shown = DetailShown {
        is_enhanced: state.is_enhanced,
        is_enhancing: state.is_enhancing,
        intensity: state.edit.enhancement_intensity,
    };
    let detail = detail_section(ui, &shown);
    state.edit.enhancement_intensity = detail.intensity;
    state.is_enhancement_asked = detail.is_enhancement_asked;
    state
}

fn masks_and_adjustments(
    ui: &mut egui::Ui,
    kind: &PhotoKind,
    state: DevelopPanelState,
) -> DevelopPanelState {
    group_title(ui, MASKS_GROUP_LABEL);
    let intent = ui
        .scope(|ui| {
            let shown = MasksShown {
                edit: &state.edit,
                selection: state.mask_selection,
                is_detecting: state.is_detecting,
            };
            masks_section(ui, &shown)
        })
        .inner;
    let state = match intent {
        Some(intent) => state.after(intent),
        None => state,
    };
    ui.add_space(space::M);
    match state.selected_mask() {
        Some(selected) => mask_adjusted(ui, state, selected),
        None => photo_adjusted(ui, kind, state),
    }
}

/// `Esc` leaves the armed tool or the selected mask, `Delete` removes the
/// selected mask, `O` shows or hides its overlay. A typed value keeps its keys.
fn after_mask_keys(ui: &egui::Ui, state: DevelopPanelState) -> DevelopPanelState {
    if ui.ctx().text_edit_focused() {
        return state;
    }
    let is_pressed = |key| ui.input_mut(|input| input.consume_key(Modifiers::NONE, key));
    let selection = state.mask_selection;
    let is_working_on_masks = selection.selected.is_some() || selection.armed_tool.is_some();
    if is_working_on_masks && is_pressed(Key::Escape) {
        return state.selecting(None);
    }
    let Some(selected) = state.selected_mask() else {
        return state;
    };
    if is_pressed(Key::Delete) || is_pressed(Key::Backspace) {
        return state.without_mask(selected);
    }
    if is_pressed(OVERLAY_KEY) {
        return DevelopPanelState {
            mask_selection: state.mask_selection.with_overlay_toggled(&state.edit),
            ..state
        };
    }
    state
}

/// Shows the edit and returns it as the user left it this frame.
/// Changing the edit leaves Before.
pub fn develop_panel(
    ui: &mut egui::Ui,
    kind: &PhotoKind,
    state: DevelopPanelState,
) -> DevelopPanelState {
    ui.spacing_mut().item_spacing.y = space::S;
    let header = panel_header(ui, &state);
    if header.is_reset_asked {
        return DevelopPanelState::default();
    }
    let shown = (state.edit.clone(), state.is_before_shown);
    let state = after_mask_keys(ui, state);
    // Room for the scroll bar: it would otherwise cover the values.
    ui.spacing_mut().scroll.floating_allocated_width = ui.spacing().scroll.bar_width;
    let left = egui::ScrollArea::vertical()
        .show(ui, |ui| masks_and_adjustments(ui, kind, state))
        .inner;
    let (shown_edit, was_before_shown) = shown;
    let is_edit_changed = left.edit != shown_edit;
    DevelopPanelState {
        is_before_shown: !is_edit_changed && (was_before_shown != header.is_before_toggled),
        ..left
    }
}
