use egui::{Rect, Sense, Stroke, Vec2, WidgetInfo, WidgetType, vec2};

use super::theme::{CONTROL_RADIUS, color};

const BUTTON_SIZE: Vec2 = vec2(24.0, 24.0);
const STROKE_WIDTH: f32 = 1.4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Shown,
    Hidden,
    Remove,
}

fn paint_eye(painter: &egui::Painter, area: Rect, stroke: Stroke) {
    let centre = area.center();
    let half_width = 6.5;
    let corners = [-1.0, 1.0].map(|side| centre + vec2(side * half_width, 0.0));
    for lid in [-1.0, 1.0] {
        let bulge = centre + vec2(0.0, lid * 6.0);
        let lid = egui::epaint::QuadraticBezierShape::from_points_stroke(
            [corners[0], bulge, corners[1]],
            false,
            egui::Color32::TRANSPARENT,
            stroke,
        );
        painter.add(lid);
    }
    painter.circle_filled(centre, 1.8, stroke.color);
}

fn paint_icon(painter: &egui::Painter, icon: Icon, area: Rect, stroke: Stroke) {
    let centre = area.center();
    match icon {
        Icon::Shown => paint_eye(painter, area, stroke),
        Icon::Hidden => {
            paint_eye(painter, area, stroke);
            let reach = vec2(5.5, 5.5);
            painter.line_segment([centre - reach, centre + reach], stroke);
        }
        Icon::Remove => {
            for rising in [-1.0, 1.0] {
                let reach = vec2(4.0, 4.0 * rising);
                painter.line_segment([centre - reach, centre + reach], stroke);
            }
        }
    }
}

/// A small button drawn as an icon; `label` is what assistive technology reads.
pub fn icon_button(ui: &mut egui::Ui, icon: Icon, label: &str) -> egui::Response {
    let (area, response) = ui.allocate_exact_size(BUTTON_SIZE, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), label));
    if response.hovered() {
        ui.painter()
            .rect_filled(area, CONTROL_RADIUS, color::RAISED_HOVERED);
    }
    let ink = match (ui.is_enabled(), response.hovered(), icon) {
        (false, _, _) => color::TEXT_DISABLED,
        (true, true, _) => color::TEXT,
        (true, false, Icon::Hidden) => color::TEXT_DISABLED,
        (true, false, _) => color::TEXT_MUTED,
    };
    paint_icon(ui.painter(), icon, area, Stroke::new(STROKE_WIDTH, ink));
    response.on_hover_text(label)
}
