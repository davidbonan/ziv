use egui::{Align2, Sense, Vec2, WidgetInfo, WidgetType, vec2};

use crate::design::ui::theme::{color, regular, space, type_size};
use crate::library::domain::mark::{MOST_STARS, Mark, Rating};

pub const MARK_LINE_HEIGHT: f32 = 20.0;
pub const REJECTED_LABEL: &str = "Rejected";
pub const STAR: &str = "★";
const STAR_SIZE: Vec2 = vec2(18.0, MARK_LINE_HEIGHT);

/// What assistive technology and tests read for the star that gives `stars`.
pub fn star_label(stars: u8) -> String {
    match stars {
        1 => "1 star".to_owned(),
        stars => format!("{stars} stars"),
    }
}

/// A star to click; `label` is what assistive technology reads.
pub fn star_button(ui: &mut egui::Ui, label: String, is_lit: bool) -> egui::Response {
    let (area, response) = ui.allocate_exact_size(STAR_SIZE, Sense::click());
    response.widget_info(|| WidgetInfo::selected(WidgetType::Button, true, is_lit, label.as_str()));
    let ink = match (is_lit, response.hovered()) {
        (true, _) => color::TEXT,
        (false, true) => color::TEXT_MUTED,
        (false, false) => color::TRACK,
    };
    let font = regular(type_size::TITLE);
    ui.painter()
        .text(area.center(), Align2::CENTER_CENTER, STAR, font, ink);
    response
}

/// The rating as five stars, then the rejected mark. Returns the rating a clicked star asks for.
pub fn mark_line(ui: &mut egui::Ui, mark: Mark) -> Option<Rating> {
    ui.horizontal(|ui| {
        ui.set_height(MARK_LINE_HEIGHT);
        ui.spacing_mut().item_spacing.x = 0.0;
        let clicked = (1..=MOST_STARS)
            .filter(|stars| {
                let is_lit = *stars <= mark.rating.stars();
                star_button(ui, star_label(*stars), is_lit).clicked()
            })
            .last();
        if mark.is_rejected {
            ui.add_space(space::M);
            ui.colored_label(color::DANGER, REJECTED_LABEL);
        }
        clicked.map(Rating::of)
    })
    .inner
}
