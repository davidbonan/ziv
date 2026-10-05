use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2, WidgetInfo, WidgetType, pos2};

use crate::design::ui::theme::{CONTROL_RADIUS, color};
use crate::develop::domain::tone_curve::{CurvePoint, ToneCurve};
use crate::histogram::domain::histogram::Histogram;
use crate::histogram::ui::histogram_plot::paint_histogram;

pub const TONE_CURVE_GRAPH_LABEL: &str = "Tone curve graph";

const POINT_RADIUS: f32 = 4.0;
const GRAB_REACH: f32 = 10.0;
/// How far outside the graph a dragged point is let go of, and removed.
const REMOVAL_REACH: f32 = 24.0;
const GRID_DIVISIONS: usize = 4;
const HISTOGRAM_OPACITY: f32 = 0.5;

/// The point the pointer holds, and where it holds it from.
#[derive(Clone, Copy)]
struct Grab {
    index: usize,
    offset: Vec2,
}

fn on_screen(area: Rect, [tone, result]: CurvePoint) -> Pos2 {
    pos2(
        area.left() + tone * area.width(),
        area.bottom() - result * area.height(),
    )
}

fn in_graph(area: Rect, position: Pos2) -> CurvePoint {
    [
        (position.x - area.left()) / area.width(),
        (area.bottom() - position.y) / area.height(),
    ]
}

fn point_near(area: Rect, curve: &ToneCurve, position: Pos2) -> Option<usize> {
    let distance_to = |point: &CurvePoint| on_screen(area, *point).distance(position);
    let (index, nearest) = curve
        .points()
        .iter()
        .enumerate()
        .min_by(|(_, one), (_, other)| distance_to(one).total_cmp(&distance_to(other)))?;
    (distance_to(nearest) <= GRAB_REACH).then_some(index)
}

/// The curve with the point under a press grabbed, a new one when there is none.
fn grabbed_at(area: Rect, curve: &ToneCurve, press: Pos2) -> Option<(ToneCurve, Grab)> {
    let (curve, index) = match point_near(area, curve, press) {
        Some(index) => (curve.clone(), index),
        None => curve.with_point_added(in_graph(area, press))?,
    };
    let offset = on_screen(area, curve.points()[index]) - press;
    Some((curve, Grab { index, offset }))
}

/// The curve with its grabbed point following the pointer: `None` for the
/// grab once the point was dragged out of the graph, and removed.
fn dragged_to(
    area: Rect,
    curve: &ToneCurve,
    grab: Grab,
    pointer: Pos2,
) -> (ToneCurve, Option<Grab>) {
    let is_dragged_out = !area.expand(REMOVAL_REACH).contains(pointer);
    if is_dragged_out && !curve.is_end_point(grab.index) {
        return (curve.without_point(grab.index), None);
    }
    let target = in_graph(area, pointer + grab.offset);
    (curve.with_point_moved(grab.index, target), Some(grab))
}

fn paint_grid(painter: &egui::Painter, area: Rect) {
    let line = Stroke::new(1.0, color::RAISED);
    for division in 1..GRID_DIVISIONS {
        let along = division as f32 / GRID_DIVISIONS as f32;
        painter.vline(area.left() + along * area.width(), area.y_range(), line);
        painter.hline(area.x_range(), area.top() + along * area.height(), line);
    }
    painter.line_segment([area.left_bottom(), area.right_top()], line);
}

/// A curve as painted: its ink, and the point shown as held.
struct CurveLook {
    ink: Color32,
    held: Option<usize>,
}

fn paint_curve(painter: &egui::Painter, area: Rect, curve: &ToneCurve, look: CurveLook) {
    let lookup = curve.lookup();
    let last_level = (lookup.len() - 1) as f32;
    let line = lookup
        .iter()
        .enumerate()
        .map(|(level, result)| on_screen(area, [level as f32 / last_level, *result]))
        .collect();
    painter.line(line, Stroke::new(1.5, look.ink));
    for (index, point) in curve.points().iter().enumerate() {
        let fill = match look.held == Some(index) {
            true => color::ACCENT,
            false => color::PANEL,
        };
        let outline = Stroke::new(1.5, look.ink);
        painter.circle(on_screen(area, *point), POINT_RADIUS, fill, outline);
    }
}

/// What the graph shows.
pub struct GraphShown<'a> {
    pub curve: &'a ToneCurve,
    /// What the curve is drawn in.
    pub ink: Color32,
    /// The histogram of the photo as developed, drawn behind the curve.
    pub histogram: Option<&'a Histogram>,
}

/// The curve on its square graph, and the curve as the user left it this
/// frame: a press away from a point adds one, a drag moves the point held,
/// a double click or a drag out of the graph removes it.
pub fn tone_curve_graph(ui: &mut egui::Ui, shown: &GraphShown<'_>) -> ToneCurve {
    let curve = shown.curve;
    let side = ui.available_width();
    let (area, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::click_and_drag());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Other, true, TONE_CURVE_GRAPH_LABEL));
    let grab_id = response.id.with("grab");
    let held: Option<Grab> = ui.data(|data| data.get_temp(grab_id));
    let held = held.filter(|grab| grab.index < curve.points().len());
    let pointer = response.interact_pointer_pos();
    let is_pressed = ui.input(|input| input.pointer.primary_pressed());

    let (left, held) = match pointer {
        Some(pointer) if response.double_clicked() => {
            let on_point = point_near(area, curve, pointer);
            let without = on_point.map(|index| curve.without_point(index));
            (without.unwrap_or_else(|| curve.clone()), None)
        }
        Some(press) if is_pressed && response.is_pointer_button_down_on() => {
            match grabbed_at(area, curve, press) {
                Some((curve, grab)) => (curve, Some(grab)),
                None => (curve.clone(), None),
            }
        }
        Some(pointer) if response.is_pointer_button_down_on() => match held {
            Some(grab) => dragged_to(area, curve, grab, pointer),
            None => (curve.clone(), None),
        },
        _ => (curve.clone(), None),
    };
    ui.data_mut(|data| match held {
        Some(grab) => {
            data.insert_temp(grab_id, grab);
        }
        None => data.remove::<Grab>(grab_id),
    });

    let painter = ui.painter();
    painter.rect_filled(area, CONTROL_RADIUS, color::CANVAS);
    if let Some(histogram) = shown.histogram {
        let mut faint = painter.with_clip_rect(area);
        faint.set_opacity(HISTOGRAM_OPACITY);
        paint_histogram(&faint, area, histogram);
    }
    paint_grid(painter, area);
    let hovered = response
        .hover_pos()
        .and_then(|pointer| point_near(area, &left, pointer));
    let look = CurveLook {
        ink: shown.ink,
        held: held.map(|grab| grab.index).or(hovered),
    };
    paint_curve(painter, area, &left, look);
    if response.has_focus() {
        let ring = Stroke::new(1.0, color::ACCENT);
        painter.rect_stroke(area, CONTROL_RADIUS, ring, egui::StrokeKind::Outside);
    }
    left
}
