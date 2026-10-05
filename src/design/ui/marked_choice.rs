use egui::{Align2, Sense, WidgetInfo, WidgetType, vec2};

use super::theme::{CONTROL_RADIUS, color, regular, space, type_size};

const MARK_RADIUS: f32 = 2.0;

/// One choice of a selector: the one chosen is raised, one holding an edit is marked.
pub struct MarkedChoice<'a> {
    pub text: &'a str,
    /// What assistive technology reads, before its mark.
    pub name: &'a str,
    pub is_chosen: bool,
    pub is_edited: bool,
}

/// What assistive technology and tests read for a choice.
pub fn marked_choice_label(name: &str, is_edited: bool) -> String {
    match is_edited {
        true => format!("{name}, edited"),
        false => name.to_owned(),
    }
}

pub fn marked_choice(ui: &mut egui::Ui, choice: &MarkedChoice<'_>) -> egui::Response {
    let font = regular(type_size::BODY);
    let text_size = ui
        .painter()
        .layout_no_wrap(choice.text.to_owned(), font.clone(), color::TEXT)
        .size();
    let size = vec2(text_size.x + 2.0 * space::S, ui.spacing().interact_size.y);
    let (area, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| {
        let label = marked_choice_label(choice.name, choice.is_edited);
        WidgetInfo::selected(WidgetType::Button, ui.is_enabled(), choice.is_chosen, label)
    });
    let painter = ui.painter();
    let is_raised = choice.is_chosen || response.hovered();
    if is_raised {
        painter.rect_filled(area, CONTROL_RADIUS, color::RAISED);
    }
    let ink = match (ui.is_enabled(), is_raised) {
        (false, _) => color::TEXT_DISABLED,
        (true, true) => color::TEXT,
        (true, false) => color::TEXT_MUTED,
    };
    painter.text(area.center(), Align2::CENTER_CENTER, choice.text, font, ink);
    if choice.is_edited {
        let mark = area.right_top() + vec2(-space::XS, space::XS + MARK_RADIUS);
        painter.circle_filled(mark, MARK_RADIUS, color::ACCENT);
    }
    response
}
