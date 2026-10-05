use egui::collapsing_header::CollapsingState;
use egui::{Align2, Sense, Stroke, WidgetInfo, WidgetType, pos2, vec2};

use super::theme::{color, medium, regular, space, type_size};

const TITLE_HEIGHT: f32 = 28.0;
const ARROW_REACH: f32 = 3.0;

pub struct SectionTitle<'a> {
    pub text: &'a str,
    /// How many things the section holds, when that is worth telling.
    pub count: Option<usize>,
}

impl<'a> SectionTitle<'a> {
    pub fn of(text: &'a str) -> Self {
        Self { text, count: None }
    }
}

/// What assistive technology and tests read for the title of a section.
pub fn section_label(title: &SectionTitle<'_>) -> String {
    match title.count.filter(|count| *count > 0) {
        Some(count) => format!("{} ({count})", title.text),
        None => title.text.to_owned(),
    }
}

fn paint_arrow(painter: &egui::Painter, centre: egui::Pos2, openness: f32) {
    let turned = egui::emath::Rot2::from_angle(openness * std::f32::consts::FRAC_PI_2);
    let points = [
        vec2(-ARROW_REACH * 0.7, -ARROW_REACH),
        vec2(ARROW_REACH * 0.8, 0.0),
        vec2(-ARROW_REACH * 0.7, ARROW_REACH),
    ];
    let points = points.map(|point| centre + turned * point);
    painter.add(egui::Shape::convex_polygon(
        points.to_vec(),
        color::TEXT_MUTED,
        Stroke::NONE,
    ));
}

fn title_row(ui: &mut egui::Ui, title: &SectionTitle<'_>, openness: f32) -> egui::Response {
    let size = vec2(ui.available_width(), TITLE_HEIGHT);
    let (area, response) = ui.allocate_exact_size(size, Sense::click());
    response.widget_info(|| {
        WidgetInfo::labeled(WidgetType::CollapsingHeader, true, section_label(title))
    });
    let painter = ui.painter();
    let hairline = Stroke::new(1.0, color::HAIRLINE);
    painter.hline(area.x_range(), area.top(), hairline);
    paint_arrow(
        painter,
        pos2(area.left() + ARROW_REACH, area.center().y),
        openness,
    );
    let ink = match response.hovered() {
        true => color::TEXT,
        false => color::TEXT_MUTED,
    };
    painter.text(
        pos2(area.left() + space::M, area.center().y),
        Align2::LEFT_CENTER,
        title.text,
        medium(type_size::CAPTION),
        ink,
    );
    if let Some(count) = title.count.filter(|count| *count > 0) {
        painter.text(
            area.right_center(),
            Align2::RIGHT_CENTER,
            count,
            regular(type_size::CAPTION),
            color::ACCENT,
        );
    }
    response
}

/// A titled group that folds when its title is clicked. Returns what
/// `add_contents` returned, `None` while the section is folded.
pub fn section<Shown>(
    ui: &mut egui::Ui,
    title: &SectionTitle<'_>,
    add_contents: impl FnOnce(&mut egui::Ui) -> Shown,
) -> Option<Shown> {
    let id = ui.make_persistent_id(("section", title.text));
    let mut state = CollapsingState::load_with_default_open(ui.ctx(), id, true);
    let openness = state.openness(ui.ctx());
    if title_row(ui, title, openness).clicked() {
        state.toggle(ui);
    }
    // Its own scope of ids: two sections may each hold a control of the same name.
    let body = state.show_body_unindented(ui, |ui| {
        ui.push_id(id, |ui| {
            let shown = add_contents(ui);
            ui.add_space(space::S);
            shown
        })
        .inner
    });
    body.map(|body| body.inner)
}
