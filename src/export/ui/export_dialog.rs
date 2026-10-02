use std::path::{Path, PathBuf};

use egui::{Align, Button, Layout, RichText, vec2};

use crate::design::ui::adjustment_slider::{AdjustmentSlider, Track, TrackScale};
use crate::design::ui::theme::{color, medium, regular, space, type_size};
use crate::export::domain::export_settings::{
    ExportFormat, ExportSettings, ExportSize, JPEG_QUALITY_RANGE, LONG_EDGE_RANGE,
};
use crate::photo::domain::photo_name::photo_name;

pub const DIALOG_TITLE: &str = "Export";
pub const JPEG_LABEL: &str = "JPEG";
pub const PNG_LABEL: &str = "PNG";
pub const QUALITY_LABEL: &str = "Quality";
pub const FULL_RESOLUTION_LABEL: &str = "Full resolution";
pub const LONG_EDGE_CHOICE_LABEL: &str = "Long edge";
pub const LONG_EDGE_LABEL: &str = "Long edge in pixels";
pub const NO_DESTINATION_LABEL: &str = "No folder chosen yet";
pub const CHOOSE_DESTINATION_LABEL: &str = "Choose…";
pub const CHANGE_DESTINATION_LABEL: &str = "Change…";
pub const CANCEL_LABEL: &str = "Cancel";
pub const EXPORT_LABEL: &str = "Export";

pub const DIALOG_WIDTH: f32 = 340.0;
const CHOICE_LABEL_WIDTH: f32 = 96.0;
const SHORT_PATH_FOLDERS: usize = 4;

const QUALITY: AdjustmentSlider = AdjustmentSlider {
    label: QUALITY_LABEL,
    range: *JPEG_QUALITY_RANGE.start() as f32..=*JPEG_QUALITY_RANGE.end() as f32,
    default: 90.0,
    step: 1.0,
    decimals: 0,
    scale: TrackScale::Linear,
    track: Track::AccentFill,
};
const LONG_EDGE: AdjustmentSlider = AdjustmentSlider {
    label: LONG_EDGE_LABEL,
    range: *LONG_EDGE_RANGE.start() as f32..=*LONG_EDGE_RANGE.end() as f32,
    default: 2048.0,
    step: 64.0,
    decimals: 0,
    scale: TrackScale::Reciprocal,
    track: Track::AccentFill,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportDialogIntent {
    Export,
    Cancel,
    ChooseDestination,
}

pub struct ExportDialogOutput {
    pub settings: ExportSettings,
    pub intent: Option<ExportDialogIntent>,
}

/// What the dialog says is about to be exported.
pub fn scope_description(photos: &[PathBuf]) -> String {
    match photos {
        [photo] => photo_name(photo),
        _ => format!("{} photos", photos.len()),
    }
}

fn muted(text: impl Into<String>) -> RichText {
    RichText::new(text)
        .font(regular(type_size::BODY))
        .color(color::TEXT_MUTED)
}

fn header(ui: &mut egui::Ui, photos: &[PathBuf]) {
    let title = RichText::new(DIALOG_TITLE)
        .font(medium(type_size::TITLE))
        .color(color::TEXT);
    ui.label(title);
    ui.label(muted(scope_description(photos)));
    ui.add_space(space::M);
}

/// A named row of mutually exclusive buttons. Returns the choice left selected.
fn choice<Choice: Copy + PartialEq>(
    ui: &mut egui::Ui,
    name: &str,
    choices: [(Choice, &str); 2],
    selected: Choice,
) -> Choice {
    let mut selected = selected;
    ui.horizontal(|ui| {
        let name_size = vec2(CHOICE_LABEL_WIDTH, ui.spacing().interact_size.y);
        let from_the_left = Layout::left_to_right(Align::Center);
        ui.allocate_ui_with_layout(name_size, from_the_left, |ui| {
            ui.set_min_size(name_size);
            ui.label(muted(name));
        });
        for (choice, label) in choices {
            if ui
                .add(Button::new(label).selected(selected == choice))
                .clicked()
            {
                selected = choice;
            }
        }
    });
    selected
}

/// One line of the dialog laid out from its right end.
fn row_from_the_right<Output>(
    ui: &mut egui::Ui,
    content: impl FnOnce(&mut egui::Ui) -> Output,
) -> Output {
    ui.horizontal(|ui| {
        ui.with_layout(Layout::right_to_left(Align::Center), content)
            .inner
    })
    .inner
}

/// The end of a path is what tells one folder from another.
fn shown_destination(folder: &Path) -> String {
    let names: Vec<_> = folder.iter().map(|name| name.to_string_lossy()).collect();
    match names.as_slice() {
        [.., parent, name] if names.len() > SHORT_PATH_FOLDERS => format!("…/{parent}/{name}"),
        _ => folder.display().to_string(),
    }
}

fn destination_row(ui: &mut egui::Ui, destination: Option<&Path>) -> bool {
    let (shown, button) = match destination {
        Some(folder) => (shown_destination(folder), CHANGE_DESTINATION_LABEL),
        None => (NO_DESTINATION_LABEL.to_owned(), CHOOSE_DESTINATION_LABEL),
    };
    row_from_the_right(ui, |ui| {
        let is_asked = ui.button(button).clicked();
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            ui.add(egui::Label::new(muted(shown)).truncate());
        });
        is_asked
    })
}

fn footer(ui: &mut egui::Ui, can_export: bool) -> Option<ExportDialogIntent> {
    row_from_the_right(ui, |ui| {
        let export =
            Button::new(RichText::new(EXPORT_LABEL).color(color::HAIRLINE)).fill(color::ACCENT);
        if ui.add_enabled(can_export, export).clicked() {
            return Some(ExportDialogIntent::Export);
        }
        ui.button(CANCEL_LABEL)
            .clicked()
            .then_some(ExportDialogIntent::Cancel)
    })
}

/// Shows what is about to be exported and how; returns the settings as the
/// user left them this frame and what they asked for.
pub fn export_dialog(
    ui: &mut egui::Ui,
    photos: &[PathBuf],
    settings: &ExportSettings,
) -> ExportDialogOutput {
    let mut settings = settings.clone();
    ui.set_width(DIALOG_WIDTH);
    ui.spacing_mut().item_spacing.y = space::S;
    header(ui, photos);

    let formats = [
        (ExportFormat::Jpeg, JPEG_LABEL),
        (ExportFormat::Png, PNG_LABEL),
    ];
    settings.format = choice(ui, "Format", formats, settings.format);
    ui.add_enabled_ui(settings.format == ExportFormat::Jpeg, |ui| {
        settings.jpeg_quality = QUALITY.show(ui, f32::from(settings.jpeg_quality)) as u8;
    });
    ui.add_space(space::S);

    let sizes = [
        (ExportSize::FullResolution, FULL_RESOLUTION_LABEL),
        (ExportSize::LongEdge, LONG_EDGE_CHOICE_LABEL),
    ];
    settings.size = choice(ui, "Size", sizes, settings.size);
    ui.add_enabled_ui(settings.size == ExportSize::LongEdge, |ui| {
        settings.long_edge = LONG_EDGE.show(ui, settings.long_edge as f32) as u32;
    });
    ui.add_space(space::S);

    let is_destination_asked = destination_row(ui, settings.destination.as_deref());
    ui.add_space(space::M);
    let intent = footer(ui, settings.destination.is_some());
    ExportDialogOutput {
        settings,
        intent: match is_destination_asked {
            true => Some(ExportDialogIntent::ChooseDestination),
            false => intent,
        },
    }
}
