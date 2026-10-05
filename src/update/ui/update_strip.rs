use egui::RichText;

use super::update_status::update_status_label;
use crate::design::ui::floating_pill::floating_pill;
use crate::design::ui::primary_button::primary_button;
use crate::design::ui::theme::{color, space};
use crate::update::domain::update_state::UpdateState;

pub const INSTALL_LABEL: &str = "Install and relaunch";
pub const LATER_LABEL: &str = "Later";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateStripIntent {
    Install,
    Later,
}

pub struct UpdateStripShown<'a> {
    pub state: &'a UpdateState,
    /// Nothing else is running that a relaunch would cut short.
    pub can_install: bool,
}

fn offer(ui: &mut egui::Ui, can_install: bool) -> Option<UpdateStripIntent> {
    ui.add_space(space::S);
    let install = ui.add_enabled_ui(can_install, |ui| primary_button(ui, INSTALL_LABEL));
    if install.inner.clicked() {
        return Some(UpdateStripIntent::Install);
    }
    ui.button(LATER_LABEL)
        .clicked()
        .then_some(UpdateStripIntent::Later)
}

/// Floats over the bottom of `over` while an update is available or being
/// installed. Returns what the user asked for this frame, if anything.
pub fn update_strip(
    ui: &egui::Ui,
    over: egui::Rect,
    shown: &UpdateStripShown<'_>,
) -> Option<UpdateStripIntent> {
    let is_offered = matches!(shown.state, UpdateState::Available(_));
    if !is_offered && !shown.state.is_installing() {
        return None;
    }
    let label = update_status_label(shown.state)?;
    floating_pill(ui, ("update", over), |ui| {
        ui.horizontal(|ui| {
            if !is_offered {
                ui.spinner();
            }
            ui.label(RichText::new(label).color(color::TEXT));
            match is_offered {
                true => offer(ui, shown.can_install),
                false => None,
            }
        })
        .inner
    })
}
