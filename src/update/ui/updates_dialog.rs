use egui::RichText;
use egui_commonmark::CommonMarkCache;

use super::release_notes_text::release_notes_text;
use super::running_version_button::running_version_label;
use super::update_status::update_status_label;
use super::update_strip::INSTALL_LABEL;
use crate::design::ui::primary_button::primary_button;
use crate::design::ui::theme::{color, medium, regular, space, type_size};
use crate::update::domain::update_state::UpdateState;
use crate::update::domain::version::Version;

pub const DIALOG_TITLE: &str = "Updates";
pub const CHECK_LABEL: &str = "Check for updates";
pub const CLOSE_LABEL: &str = "Close";
pub const UNBUNDLED_LABEL: &str = "Running outside an app bundle: updates are disabled";
pub const DIALOG_WIDTH: f32 = 440.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdatesDialogIntent {
    Check,
    Install,
    Close,
}

pub struct UpdatesShown<'a> {
    pub running: Version,
    /// `None` for an app that cannot update itself: it runs outside an app bundle.
    pub state: Option<&'a UpdateState>,
    /// Nothing else is running that a relaunch would cut short.
    pub can_install: bool,
}

fn muted(text: impl Into<String>) -> RichText {
    RichText::new(text)
        .font(regular(type_size::BODY))
        .color(color::TEXT_MUTED)
}

fn status(ui: &mut egui::Ui, state: &UpdateState) {
    let Some(label) = update_status_label(state) else {
        return;
    };
    if state.is_busy() {
        ui.spinner();
    }
    let ink = match state {
        UpdateState::Failed(_) => color::DANGER,
        _ => color::TEXT,
    };
    ui.add(egui::Label::new(RichText::new(label).color(ink)).wrap());
}

fn update_row(
    ui: &mut egui::Ui,
    state: &UpdateState,
    can_install: bool,
) -> Option<UpdatesDialogIntent> {
    let is_offered = matches!(state, UpdateState::Available(_));
    ui.horizontal_wrapped(|ui| {
        let check = ui.add_enabled(!state.is_busy(), egui::Button::new(CHECK_LABEL));
        if check.clicked() {
            return Some(UpdatesDialogIntent::Check);
        }
        status(ui, state);
        if !is_offered {
            return None;
        }
        let install = ui.add_enabled_ui(can_install, |ui| primary_button(ui, INSTALL_LABEL));
        install
            .inner
            .clicked()
            .then_some(UpdatesDialogIntent::Install)
    })
    .inner
}

/// The running version, the update and the release notes. Returns what the
/// user asked for this frame, if anything.
pub fn updates_dialog(
    ui: &mut egui::Ui,
    shown: &UpdatesShown<'_>,
    notes_layout: &mut CommonMarkCache,
) -> Option<UpdatesDialogIntent> {
    ui.set_width(DIALOG_WIDTH);
    ui.spacing_mut().item_spacing.y = space::S;
    ui.label(RichText::new(DIALOG_TITLE).font(medium(type_size::TITLE)));
    ui.label(muted(running_version_label(shown.running)));
    let asked = match shown.state {
        Some(state) => update_row(ui, state, shown.can_install),
        None => {
            ui.label(muted(UNBUNDLED_LABEL));
            None
        }
    };
    ui.add_space(space::S);
    release_notes_text(ui, notes_layout);
    ui.add_space(space::S);
    let is_closed = ui.button(CLOSE_LABEL).clicked();
    asked.or(is_closed.then_some(UpdatesDialogIntent::Close))
}
