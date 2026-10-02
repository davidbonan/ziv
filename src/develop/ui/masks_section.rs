use egui::{Align, Align2, Layout, Sense, WidgetInfo, WidgetType, vec2};

use crate::design::ui::icon_button::{Icon, icon_button};
use crate::design::ui::theme::{CONTROL_RADIUS, color, regular, space, type_size};
use crate::develop::domain::brush::Brush;
use crate::develop::domain::edit::Edit;
use crate::develop::domain::mask::{Mask, MaskKind, MaskShape};
use crate::develop::domain::zone::{DetectionTool, ZoneTool};

pub const MASKS_GROUP_LABEL: &str = "Masks";
pub const LINEAR_GRADIENT_TOOL_LABEL: &str = "Linear";
pub const RADIAL_GRADIENT_TOOL_LABEL: &str = "Radial";
pub const RECTANGLE_TOOL_LABEL: &str = "Rectangle";
pub const POLYGON_TOOL_LABEL: &str = "Polygon";
pub const BRUSH_TOOL_LABEL: &str = "Brush";

pub const SUBJECT_TOOL_LABEL: &str = "Subject";
pub const BACKGROUND_TOOL_LABEL: &str = "Background";
pub const SKY_TOOL_LABEL: &str = "Sky";
pub const PEOPLE_TOOL_LABEL: &str = "People";

const ZONE_TOOLS: [(DetectionTool, &str); 4] = [
    (DetectionTool::Zone(ZoneTool::Subject), SUBJECT_TOOL_LABEL),
    (
        DetectionTool::Zone(ZoneTool::Background),
        BACKGROUND_TOOL_LABEL,
    ),
    (DetectionTool::Zone(ZoneTool::Sky), SKY_TOOL_LABEL),
    (DetectionTool::People, PEOPLE_TOOL_LABEL),
];

const TOOLS: [(MaskKind, &str); 5] = [
    (MaskKind::LinearGradient, LINEAR_GRADIENT_TOOL_LABEL),
    (MaskKind::RadialGradient, RADIAL_GRADIENT_TOOL_LABEL),
    (MaskKind::Rectangle, RECTANGLE_TOOL_LABEL),
    (MaskKind::Polygon, POLYGON_TOOL_LABEL),
    (MaskKind::Brush, BRUSH_TOOL_LABEL),
];

/// What the masks of the photo are being worked on with: the mask whose
/// adjustments the panel shows, or the tool about to draw a new one.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MaskSelection {
    pub selected: Option<usize>,
    pub armed_tool: Option<MaskKind>,
    /// What `O` or the Overlay button last asked for; `None`: the overlay
    /// shows as long as the mask has no adjustment.
    pub is_overlay_asked: Option<bool>,
    /// What the next stroke is painted with.
    pub brush: Brush,
}

impl MaskSelection {
    pub fn of(selected: usize) -> Self {
        Self::default().selecting(Some(selected))
    }

    /// Leaves the armed tool and forgets what was asked of the overlay.
    pub fn selecting(self, selected: Option<usize>) -> Self {
        Self {
            selected,
            brush: self.brush,
            ..Self::default()
        }
    }

    pub fn arming(self, tool: Option<MaskKind>) -> Self {
        Self {
            armed_tool: tool,
            ..self.selecting(None)
        }
    }

    pub fn selected_mask<'a>(&self, edit: &'a Edit) -> Option<&'a Mask> {
        edit.masks.get(self.selected?)
    }

    pub fn selected_shape<'a>(&self, masks: &'a [Mask]) -> Option<&'a MaskShape> {
        Some(&masks.get(self.selected?)?.shape)
    }

    /// The mask whose coverage the overlay shows now.
    pub fn overlaid_mask<'a>(&self, edit: &'a Edit) -> Option<&'a Mask> {
        let mask = self.selected_mask(edit)?;
        self.is_overlay_asked
            .unwrap_or(!mask.has_adjustments())
            .then_some(mask)
    }

    pub fn with_overlay_toggled(self, edit: &Edit) -> Self {
        Self {
            is_overlay_asked: Some(self.overlaid_mask(edit).is_none()),
            ..self
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MasksIntent {
    /// Asking for the armed tool disarms it.
    Arm(MaskKind),
    Detect(DetectionTool),
    /// Asking for the selected mask deselects it.
    Select(usize),
    ToggleVisibility(usize),
    Remove(usize),
}

pub fn hide_label(mask_name: &str) -> String {
    format!("Hide {mask_name}")
}

pub fn show_label(mask_name: &str) -> String {
    format!("Show {mask_name}")
}

pub fn remove_label(mask_name: &str) -> String {
    format!("Remove {mask_name}")
}

fn tools(ui: &mut egui::Ui, edit: &Edit, armed: Option<MaskKind>) -> Option<MaskKind> {
    ui.add_enabled_ui(edit.has_room_for_a_mask(), |ui| {
        ui.horizontal_wrapped(|ui| {
            TOOLS.into_iter().find(|(kind, label)| {
                let tool = egui::Button::new(*label).selected(armed == Some(*kind));
                ui.add(tool).clicked()
            })
        })
        .inner
    })
    .inner
    .map(|(kind, _)| kind)
}

fn zone_tools(ui: &mut egui::Ui, shown: &MasksShown<'_>) -> Option<DetectionTool> {
    let can_detect = shown.edit.has_room_for_a_mask() && !shown.is_detecting;
    ui.add_enabled_ui(can_detect, |ui| {
        ui.horizontal_wrapped(|ui| {
            ZONE_TOOLS
                .into_iter()
                .find(|(_, label)| ui.button(*label).clicked())
        })
        .inner
    })
    .inner
    .map(|(tool, _)| tool)
}

struct MaskRow<'a> {
    index: usize,
    name: &'a str,
    is_selected: bool,
    is_hidden: bool,
}

fn mask_name(ui: &mut egui::Ui, row: &MaskRow<'_>, width: f32) -> egui::Response {
    let size = vec2(width, ui.spacing().interact_size.y);
    let (area, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| {
        WidgetInfo::selected(WidgetType::SelectableLabel, true, row.is_selected, row.name)
    });
    if row.is_selected || response.hovered() {
        ui.painter()
            .rect_filled(area, CONTROL_RADIUS, color::RAISED);
    }
    let ink = match (row.is_selected, row.is_hidden) {
        (true, _) => color::ACCENT,
        (false, true) => color::TEXT_DISABLED,
        (false, false) => color::TEXT,
    };
    ui.painter().text(
        area.left_center() + vec2(space::S, 0.0),
        Align2::LEFT_CENTER,
        row.name,
        regular(type_size::BODY),
        ink,
    );
    response
}

fn mask_row(ui: &mut egui::Ui, row: &MaskRow<'_>) -> Option<MasksIntent> {
    ui.horizontal(|ui| {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = space::XS;
            let (eye, eye_label) = match row.is_hidden {
                true => (Icon::Hidden, show_label(row.name)),
                false => (Icon::Shown, hide_label(row.name)),
            };
            if icon_button(ui, Icon::Remove, &remove_label(row.name)).clicked() {
                return Some(MasksIntent::Remove(row.index));
            }
            if icon_button(ui, eye, &eye_label).clicked() {
                return Some(MasksIntent::ToggleVisibility(row.index));
            }
            mask_name(ui, row, ui.available_width())
                .clicked()
                .then_some(MasksIntent::Select(row.index))
        })
        .inner
    })
    .inner
}

pub struct MasksShown<'a> {
    pub edit: &'a Edit,
    pub selection: MaskSelection,
    /// A detection runs: no other can be asked for.
    pub is_detecting: bool,
}

/// The mask tools, the zone tools and the list of the photo's masks.
pub fn masks_section(ui: &mut egui::Ui, shown: &MasksShown<'_>) -> Option<MasksIntent> {
    let MasksShown {
        edit, selection, ..
    } = shown;
    let armed = tools(ui, edit, selection.armed_tool).map(MasksIntent::Arm);
    let detected = zone_tools(ui, shown).map(MasksIntent::Detect);
    let mut intent = armed.or(detected);
    ui.spacing_mut().item_spacing.y = space::XS;
    for (index, (mask, name)) in edit.masks.iter().zip(edit.mask_names()).enumerate() {
        let row = MaskRow {
            index,
            name: &name,
            is_selected: selection.selected == Some(index),
            is_hidden: mask.is_hidden,
        };
        intent = intent.or(mask_row(ui, &row));
    }
    intent
}
