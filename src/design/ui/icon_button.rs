use egui::{Rect, Sense, Stroke, Vec2, WidgetInfo, WidgetType, vec2};

use super::theme::{CONTROL_RADIUS, color};

const BUTTON_SIZE: Vec2 = vec2(24.0, 24.0);
const TOGGLE_SIZE: Vec2 = vec2(24.0, 24.0);
const STROKE_WIDTH: f32 = 1.4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Shown,
    Hidden,
    Remove,
    Add,
    Sidebar,
    Crop,
    RotateLeft,
    RotateRight,
    FlipHorizontal,
    FlipVertical,
    LinearGradient,
    RadialGradient,
    Rectangle,
    Polygon,
    Brush,
    Subject,
    Background,
    Sky,
    People,
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

fn paint_outline(painter: &egui::Painter, points: Vec<egui::Pos2>, stroke: Stroke) {
    painter.add(egui::Shape::closed_line(points, stroke));
}

/// Points of the arc of a circle, angles in degrees, clockwise from the right.
fn arc(centre: egui::Pos2, radius: f32, degrees: [f32; 2]) -> impl Iterator<Item = egui::Pos2> {
    const STEPS: usize = 8;
    (0..=STEPS).map(move |step| {
        let turn = egui::lerp(degrees[0]..=degrees[1], step as f32 / STEPS as f32).to_radians();
        centre + radius * vec2(turn.cos(), turn.sin())
    })
}

fn paint_person(painter: &egui::Painter, centre: egui::Pos2, size: f32, stroke: Stroke) {
    let head = centre - vec2(0.0, size * 0.45);
    painter.circle_stroke(head, size * 0.36, stroke);
    let shoulders: Vec<egui::Pos2> =
        arc(centre + vec2(0.0, size), size * 0.75, [180.0, 360.0]).collect();
    painter.add(egui::Shape::line(shoulders, stroke));
}

fn paint_cloud(painter: &egui::Painter, centre: egui::Pos2, stroke: Stroke) {
    let base = 3.5;
    let left = arc(centre + vec2(-3.5, 1.0), 2.5, [90.0, 270.0]);
    let top = arc(centre + vec2(-0.5, -0.5), 3.6, [205.0, 335.0]);
    let right = arc(centre + vec2(3.8, 1.2), 2.3, [270.0, 450.0]);
    let mut outline: Vec<egui::Pos2> = left.chain(top).chain(right).collect();
    outline.push(centre + vec2(-3.5, base));
    paint_outline(painter, outline, stroke);
}

// Two set squares crossing: the corners of a frame laid over a picture.
fn paint_crop(painter: &egui::Painter, centre: egui::Pos2, stroke: Stroke) {
    let (near, far) = (4.0, 7.0);
    for side in [-1.0, 1.0] {
        let corner = centre + vec2(-near, near) * side;
        let ends = [vec2(-near, -far), vec2(far, near)].map(|end| centre + end * side);
        painter.add(egui::Shape::line(vec![ends[0], corner, ends[1]], stroke));
    }
}

// Three quarters of a circle ending in an arrow head, turning towards `side`.
fn paint_quarter_turn(painter: &egui::Painter, centre: egui::Pos2, side: f32, stroke: Stroke) {
    let radius = 5.5;
    let mirrored =
        |point: egui::Pos2| centre + vec2((point.x - centre.x) * side, point.y - centre.y);
    let circle: Vec<egui::Pos2> = arc(centre, radius, [-270.0, 0.0]).map(mirrored).collect();
    painter.add(egui::Shape::line(circle, stroke));
    let tip = centre + vec2(radius, 0.0);
    let head = [vec2(-3.0, -2.5), vec2(0.0, 0.0), vec2(2.5, -3.0)].map(|end| mirrored(tip + end));
    painter.add(egui::Shape::line(head.to_vec(), stroke));
}

// Two triangles facing each other across a dashed mirror line along `mirror`.
fn paint_flip(painter: &egui::Painter, centre: egui::Pos2, mirror: Vec2, stroke: Stroke) {
    let away = vec2(mirror.y, mirror.x);
    for dash in [-6.0, -1.5, 3.0] {
        painter.line_segment(
            [centre + mirror * dash, centre + mirror * (dash + 3.0)],
            stroke,
        );
    }
    for side in [-1.0, 1.0] {
        let foot = centre + away * side * 2.5;
        let triangle = vec![
            foot - mirror * 4.5,
            foot + mirror * 4.5,
            foot + away * side * 4.5,
        ];
        paint_outline(painter, triangle, stroke);
    }
}

fn paint_mask_tool(painter: &egui::Painter, icon: Icon, centre: egui::Pos2, stroke: Stroke) {
    match icon {
        Icon::LinearGradient => {
            for line in [-4.0, 0.0, 4.0] {
                let reach = vec2(6.0, 0.0);
                let middle = centre + vec2(0.0, line);
                painter.line_segment([middle - reach, middle + reach], stroke);
            }
            let reach = vec2(0.0, 6.5);
            painter.line_segment([centre - reach, centre + reach], stroke);
        }
        Icon::RadialGradient => {
            painter.circle_stroke(centre, 6.0, stroke);
            painter.circle_stroke(centre, 2.2, stroke);
        }
        Icon::Rectangle => {
            let shape = Rect::from_center_size(centre, vec2(12.0, 9.0));
            painter.rect_stroke(shape, 1.0, stroke, egui::StrokeKind::Middle);
        }
        Icon::Polygon => {
            let corners = (0..5).map(|corner| {
                let turn = (-90.0 + 72.0 * corner as f32).to_radians();
                centre + 6.5 * vec2(turn.cos(), turn.sin())
            });
            paint_outline(painter, corners.collect(), stroke);
        }
        Icon::Brush => {
            let tip = centre + vec2(-2.0, 2.0);
            painter.line_segment([centre + vec2(6.0, -6.0), tip], stroke);
            painter.circle_stroke(tip + vec2(-2.0, 2.0), 2.4, stroke);
        }
        Icon::Subject => paint_person(painter, centre, 6.0, stroke),
        Icon::Background => {
            let frame = Rect::from_center_size(centre, vec2(12.0, 12.0));
            for corner in [
                frame.left_top(),
                frame.right_top(),
                frame.right_bottom(),
                frame.left_bottom(),
            ] {
                let inward = (centre - corner) * 0.42;
                painter.line_segment([corner, corner + vec2(inward.x, 0.0)], stroke);
                painter.line_segment([corner, corner + vec2(0.0, inward.y)], stroke);
            }
            painter.circle_stroke(centre, 2.0, stroke);
        }
        Icon::Sky => paint_cloud(painter, centre, stroke),
        Icon::People => {
            paint_person(painter, centre + vec2(-2.5, 0.5), 5.0, stroke);
            paint_person(painter, centre + vec2(4.0, 1.5), 3.6, stroke);
        }
        _ => {}
    }
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
        Icon::Add => {
            for reach in [vec2(5.0, 0.0), vec2(0.0, 5.0)] {
                painter.line_segment([centre - reach, centre + reach], stroke);
            }
        }
        Icon::Sidebar => {
            let window = Rect::from_center_size(centre, vec2(13.0, 11.0));
            painter.rect_stroke(window, 1.5, stroke, egui::StrokeKind::Middle);
            let divider = window.left() + 4.5;
            painter.line_segment(
                [
                    egui::pos2(divider, window.top()),
                    egui::pos2(divider, window.bottom()),
                ],
                stroke,
            );
        }
        Icon::Crop => paint_crop(painter, centre, stroke),
        Icon::RotateLeft => paint_quarter_turn(painter, centre, -1.0, stroke),
        Icon::RotateRight => paint_quarter_turn(painter, centre, 1.0, stroke),
        Icon::FlipHorizontal => paint_flip(painter, centre, vec2(0.0, 1.0), stroke),
        Icon::FlipVertical => paint_flip(painter, centre, vec2(1.0, 0.0), stroke),
        tool => paint_mask_tool(painter, tool, centre, stroke),
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

/// An icon button that stays on: `is_on` shows it in the accent.
pub fn icon_toggle(ui: &mut egui::Ui, icon: Icon, label: &str, is_on: bool) -> egui::Response {
    let (area, response) = ui.allocate_exact_size(TOGGLE_SIZE, Sense::click());
    response
        .widget_info(|| WidgetInfo::selected(WidgetType::Button, ui.is_enabled(), is_on, label));
    if is_on || response.hovered() {
        ui.painter()
            .rect_filled(area, CONTROL_RADIUS, color::RAISED);
    }
    let ink = match (ui.is_enabled(), is_on, response.hovered()) {
        (false, _, _) => color::TEXT_DISABLED,
        (true, true, _) => color::ACCENT,
        (true, false, true) => color::TEXT,
        (true, false, false) => color::TEXT_MUTED,
    };
    paint_icon(ui.painter(), icon, area, Stroke::new(STROKE_WIDTH, ink));
    response.on_hover_text(label)
}
