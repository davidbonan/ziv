use egui::{Align, Layout, RichText};

use crate::design::ui::icon_button::{Icon, icon_toggle};
use crate::design::ui::primary_button::primary_button;
use crate::design::ui::theme::{color, medium, regular, space, type_size};
use crate::library::domain::series_filter::SeriesFilter;
use crate::library::ui::series_filter_switch::series_filter_switch;
use crate::shell::domain::window_mode::WindowMode;

pub const TOP_BAR_HEIGHT: f32 = 40.0;
pub const SIDEBAR_TOGGLE_LABEL: &str = "Show or hide the series";
pub const BEFORE_LABEL: &str = "Before";
pub const EXPORT_BUTTON_LABEL: &str = "Export…";
pub const CULL_LABEL: &str = "Cull";
pub const DEVELOP_LABEL: &str = "Develop";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TopBarShown<'a> {
    pub is_sidebar_shown: bool,
    pub series_name: Option<&'a str>,
    pub photo_name: Option<&'a str>,
    pub mode: WindowMode,
    /// The filter of the open series; `None` without one.
    pub filter: Option<SeriesFilter>,
    /// `None` without a ready photo.
    pub zoom_readout: Option<&'a str>,
    pub is_before_shown: bool,
    pub can_before_be_shown: bool,
    pub can_export: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopBarIntent {
    ToggleSidebar,
    SwitchTo(WindowMode),
    Filter(SeriesFilter),
    ToggleBefore,
    Export,
}

fn names(ui: &mut egui::Ui, shown: &TopBarShown<'_>) {
    let Some(series) = shown.series_name else {
        return;
    };
    ui.spacing_mut().item_spacing.x = space::XS;
    let muted = |text: &str| {
        RichText::new(text)
            .font(regular(type_size::BODY))
            .color(color::TEXT_MUTED)
    };
    ui.label(muted(series));
    if let Some(photo) = shown.photo_name {
        ui.label(muted("/"));
        ui.label(RichText::new(photo).font(medium(type_size::BODY)));
    }
}

fn mode_switch(ui: &mut egui::Ui, mode: WindowMode) -> Option<TopBarIntent> {
    ui.spacing_mut().item_spacing.x = space::XS;
    let modes = [
        (CULL_LABEL, WindowMode::Cull),
        (DEVELOP_LABEL, WindowMode::Develop),
    ];
    let clicked = modes.into_iter().find(|(label, of_button)| {
        let button = egui::Button::new(*label)
            .frame_when_inactive(false)
            .selected(mode == *of_button);
        ui.add(button).clicked()
    });
    clicked.map(|(_, mode)| TopBarIntent::SwitchTo(mode))
}

fn actions(ui: &mut egui::Ui, shown: &TopBarShown<'_>) -> Option<TopBarIntent> {
    let mut intent = None;
    let export = ui.add_enabled_ui(shown.can_export, |ui| {
        primary_button(ui, EXPORT_BUTTON_LABEL)
    });
    if export.inner.clicked() {
        intent = Some(TopBarIntent::Export);
    }
    let before = egui::Button::new(BEFORE_LABEL)
        .frame_when_inactive(false)
        .selected(shown.is_before_shown);
    if ui.add_enabled(shown.can_before_be_shown, before).clicked() {
        intent = Some(TopBarIntent::ToggleBefore);
    }
    if let Some(zoom) = shown.zoom_readout {
        let text = RichText::new(zoom)
            .font(regular(type_size::BODY))
            .color(color::TEXT_MUTED);
        ui.label(text);
    }
    intent
}

/// The band across the top of the window. Returns what the user asked for this frame, if anything.
pub fn top_bar(ui: &mut egui::Ui, shown: &TopBarShown<'_>) -> Option<TopBarIntent> {
    ui.horizontal_centered(|ui| {
        let sidebar = icon_toggle(
            ui,
            Icon::Sidebar,
            SIDEBAR_TOGGLE_LABEL,
            shown.is_sidebar_shown,
        );
        let toggled = sidebar.clicked().then_some(TopBarIntent::ToggleSidebar);
        ui.scope(|ui| names(ui, shown));
        ui.add_space(space::M);
        let switched = ui.scope(|ui| mode_switch(ui, shown.mode)).inner;
        ui.add_space(space::M);
        let filtered = shown
            .filter
            .and_then(|filter| ui.scope(|ui| series_filter_switch(ui, filter)).inner)
            .map(TopBarIntent::Filter);
        let asked = ui
            .with_layout(Layout::right_to_left(Align::Center), |ui| {
                actions(ui, shown)
            })
            .inner;
        toggled.or(switched).or(filtered).or(asked)
    })
    .inner
}
