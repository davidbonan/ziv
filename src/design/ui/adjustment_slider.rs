use std::ops::RangeInclusive;

use egui::{
    Align, Align2, Color32, Key, Modifiers, Rect, Sense, Stroke, StrokeKind, TextEdit, WidgetInfo,
    pos2, vec2,
};

use egui::text::{CCursor, CCursorRange};
use egui::text_edit::TextEditState;

use super::theme::{CONTROL_RADIUS, color, hue, regular, space, type_size};

const NAME_LINE_HEIGHT: f32 = 18.0;
const TRACK_LINE_HEIGHT: f32 = 16.0;
const VALUE_WIDTH: f32 = 56.0;
const TRACK_THICKNESS: f32 = 2.0;
const HUE_TRACK_THICKNESS: f32 = 4.0;
const DEFAULT_TICK_HEIGHT: f32 = 7.0;
const HANDLE_RADIUS: f32 = 5.0;
const HANDLE_RADIUS_ENGAGED: f32 = 6.5;
const HANDLE_GAP: f32 = 2.0;
const ENGAGE_SECONDS: f32 = 0.12;
const MINUS: char = '−';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackScale {
    Linear,
    /// Even steps of 1 / value: a kelvin track that gives warm lights their room.
    Reciprocal,
}

impl TrackScale {
    fn position(&self, value: f32) -> f32 {
        match self {
            Self::Linear => value,
            Self::Reciprocal => -1.0 / value,
        }
    }

    fn value(&self, position: f32) -> f32 {
        match self {
            Self::Linear => position,
            Self::Reciprocal => -1.0 / position,
        }
    }
}

/// What the track of a slider is painted with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Track {
    /// Grey, filled with the accent from the default to the value.
    AccentFill,
    /// From one hue to the other through neutral: where each end takes the photo.
    Hues { start: Color32, end: Color32 },
}

pub struct AdjustmentSlider {
    pub label: &'static str,
    pub range: RangeInclusive<f32>,
    pub default: f32,
    pub step: f32,
    pub decimals: usize,
    pub scale: TrackScale,
    pub track: Track,
}

pub fn value_field_label(label: &str) -> String {
    format!("{label} value")
}

fn select_all(context: &egui::Context, editor: egui::Id, text: &str) {
    let mut state = TextEditState::load(context, editor).unwrap_or_default();
    let everything = CCursorRange::two(CCursor::new(0), CCursor::new(text.chars().count()));
    state.cursor.set_char_range(Some(everything));
    state.store(context, editor);
}

/// A band going from the first color to the last through the middle one.
fn paint_hues(painter: &egui::Painter, band: Rect, colors: [Color32; 3]) {
    let mut mesh = egui::Mesh::default();
    let stops = [band.left(), band.center().x, band.right()];
    for (x, color) in stops.into_iter().zip(colors) {
        mesh.colored_vertex(pos2(x, band.top()), color);
        mesh.colored_vertex(pos2(x, band.bottom()), color);
    }
    for stop in [0, 2] {
        mesh.add_triangle(stop, stop + 1, stop + 2);
        mesh.add_triangle(stop + 1, stop + 2, stop + 3);
    }
    painter.add(mesh);
}

struct Shown {
    value: f32,
    enabled: bool,
    is_changed: bool,
    is_typing: bool,
    has_focus: bool,
    engagement: f32,
}

struct Layout {
    name: Rect,
    value: Rect,
    track_line: Rect,
}

impl AdjustmentSlider {
    fn is_signed(&self) -> bool {
        *self.range.start() < 0.0
    }

    fn clamped(&self, value: f32) -> f32 {
        let quantum = 10f64.powi(-(self.decimals as i32));
        let rounded = ((f64::from(value) / quantum).round() * quantum) as f32;
        rounded.clamp(*self.range.start(), *self.range.end())
    }

    fn fraction(&self, value: f32) -> f32 {
        let [start, end, value] =
            [*self.range.start(), *self.range.end(), value].map(|value| self.scale.position(value));
        (value - start) / (end - start)
    }

    fn value_at(&self, fraction: f32) -> f32 {
        let [start, end] =
            [*self.range.start(), *self.range.end()].map(|value| self.scale.position(value));
        self.scale
            .value(start + fraction.clamp(0.0, 1.0) * (end - start))
    }

    pub fn formatted(&self, value: f32) -> String {
        let magnitude = format!("{:.*}", self.decimals, value.abs());
        let is_zero = magnitude
            .chars()
            .all(|character| character == '0' || character == '.');
        match (self.is_signed(), is_zero, value < 0.0) {
            (_, false, true) => format!("{MINUS}{magnitude}"),
            (true, false, false) => format!("+{magnitude}"),
            _ => magnitude,
        }
    }

    pub fn parsed(&self, typed: &str) -> Option<f32> {
        let ascii = typed.trim().replace(MINUS, "-").replace(',', ".");
        ascii.parse::<f32>().ok().map(|value| self.clamped(value))
    }

    fn layout(&self, ui: &mut egui::Ui) -> Layout {
        let size = vec2(ui.available_width(), NAME_LINE_HEIGHT + TRACK_LINE_HEIGHT);
        let (row, _) = ui.allocate_exact_size(size, Sense::hover());
        let (name_line, track_line) = row.split_top_bottom_at_y(row.top() + NAME_LINE_HEIGHT);
        let (name, value) = name_line.split_left_right_at_x(name_line.right() - VALUE_WIDTH);
        Layout {
            name,
            value,
            track_line,
        }
    }

    fn typing_id(&self, ui: &egui::Ui) -> egui::Id {
        ui.id().with((self.label, "typing"))
    }

    fn typed_value(&self, ui: &mut egui::Ui, area: Rect, value: f32) -> f32 {
        let typing_id = self.typing_id(ui);
        let typing: Option<String> = ui.data(|data| data.get_temp(typing_id));
        let Some(mut text) = typing else {
            let field = ui.interact(area, typing_id, Sense::click());
            field.widget_info(|| {
                WidgetInfo::labeled(
                    egui::WidgetType::Button,
                    ui.is_enabled(),
                    value_field_label(self.label),
                )
            });
            if field.clicked() {
                ui.data_mut(|data| data.insert_temp(typing_id, self.formatted(value)));
            }
            return value;
        };

        let editor = TextEdit::singleline(&mut text)
            .id(typing_id.with("editor"))
            .font(regular(type_size::BODY))
            .text_color(color::TEXT)
            .horizontal_align(Align::RIGHT)
            .margin(vec2(space::XS, 0.0));
        let field = ui.put(area, editor);
        if !field.has_focus() && !field.lost_focus() {
            field.request_focus();
            select_all(ui.ctx(), field.id, &text);
        }
        if !field.lost_focus() {
            ui.data_mut(|data| data.insert_temp(typing_id, text));
            return value;
        }
        ui.data_mut(|data| data.remove_temp::<String>(typing_id));
        if ui.input(|input| input.key_pressed(Key::Escape)) {
            return value;
        }
        self.parsed(&text).unwrap_or(value)
    }

    fn stepped_by_arrow_keys(&self, ui: &egui::Ui, value: f32) -> f32 {
        let pressed = |key| ui.input_mut(|input| input.consume_key(Modifiers::NONE, key));
        let mut value = value;
        if pressed(Key::ArrowLeft) {
            value -= self.step;
        }
        if pressed(Key::ArrowRight) {
            value += self.step;
        }
        value
    }

    fn handle_travel(track_line: Rect) -> f32 {
        track_line.width() - 2.0 * HANDLE_RADIUS_ENGAGED
    }

    // Relative drag: pressing the track never makes the value jump. The exact
    // handle position is remembered so that rounding the value loses no travel.
    fn dragged_value(&self, ui: &egui::Ui, track: &egui::Response, value: f32) -> f32 {
        let position_id = track.id.with("dragged position");
        let remembered: Option<f32> = ui.data(|data| data.get_temp(position_id));
        let before = match remembered {
            Some(position) if !track.drag_started() => position,
            _ => self.fraction(value),
        };
        let travelled = track.drag_delta().x / Self::handle_travel(track.rect);
        let position = (before + travelled).clamp(0.0, 1.0);
        ui.data_mut(|data| data.insert_temp(position_id, position));
        self.value_at(position)
    }

    fn moved_on_track(&self, ui: &egui::Ui, track: &egui::Response, value: f32) -> f32 {
        if track.double_clicked() {
            return self.default;
        }
        if track.dragged() {
            return self.clamped(self.dragged_value(ui, track, value));
        }
        if track.has_focus() {
            // Otherwise egui spends the arrows on moving the focus to a neighbour.
            let arrows = egui::EventFilter {
                horizontal_arrows: true,
                ..Default::default()
            };
            ui.memory_mut(|memory| memory.set_focus_lock_filter(track.id, arrows));
            return self.clamped(self.stepped_by_arrow_keys(ui, value));
        }
        self.clamped(value)
    }

    fn interact(&self, ui: &mut egui::Ui, layout: &Layout, value: f32) -> Shown {
        let id = ui.id().with(self.label);
        let enabled = ui.is_enabled();
        let name = ui.interact(layout.name, id.with("name"), Sense::click());
        let typed = self.typed_value(ui, layout.value, value);
        let track = ui.interact(layout.track_line, id, Sense::click_and_drag());
        track.widget_info(|| WidgetInfo::slider(enabled, f64::from(typed), self.label));

        let value = if name.double_clicked() {
            self.default
        } else {
            self.moved_on_track(ui, &track, typed)
        };
        let is_engaged = enabled && (track.hovered() || track.dragged());
        Shown {
            value,
            enabled,
            is_changed: self.clamped(value) != self.clamped(self.default),
            is_typing: ui.data(|data| data.get_temp::<String>(self.typing_id(ui)).is_some()),
            has_focus: track.has_focus(),
            engagement: ui.ctx().animate_bool_with_time(
                id.with("engaged"),
                is_engaged,
                ENGAGE_SECONDS,
            ),
        }
    }

    fn paint_name_and_value(&self, painter: &egui::Painter, layout: &Layout, shown: &Shown) {
        let (name_color, value_color) = match (shown.enabled, shown.is_changed) {
            (false, _) => (color::TEXT_DISABLED, color::TEXT_DISABLED),
            (true, false) => (color::TEXT_MUTED, color::TEXT_MUTED),
            (true, true) => (color::TEXT, color::ACCENT),
        };
        painter.text(
            layout.name.left_center(),
            Align2::LEFT_CENTER,
            self.label,
            regular(type_size::BODY),
            name_color,
        );
        if shown.is_typing {
            return;
        }
        painter.text(
            layout.value.right_center(),
            Align2::RIGHT_CENTER,
            self.formatted(shown.value),
            regular(type_size::BODY),
            value_color,
        );
    }

    fn paint_track(&self, painter: &egui::Painter, track_line: Rect, shown: &Shown) {
        let center_y = track_line.center().y;
        let x_of = |value: f32| {
            track_line.left()
                + HANDLE_RADIUS_ENGAGED
                + self.fraction(value) * Self::handle_travel(track_line)
        };
        let segment = |from: f32, to: f32| {
            Rect::from_min_max(
                pos2(from.min(to), center_y - TRACK_THICKNESS / 2.0),
                pos2(from.max(to), center_y + TRACK_THICKNESS / 2.0),
            )
        };
        let (start_x, end_x) = (x_of(*self.range.start()), x_of(*self.range.end()));
        let (default_x, handle_x) = (x_of(self.default), x_of(shown.value));
        if default_x > start_x && default_x < end_x {
            let tick =
                Rect::from_center_size(pos2(default_x, center_y), vec2(1.0, DEFAULT_TICK_HEIGHT));
            painter.rect_filled(tick, 0.0, color::TRACK);
        }
        match self.track {
            Track::Hues { start, end } if shown.enabled => {
                let grow = (HUE_TRACK_THICKNESS - TRACK_THICKNESS) / 2.0;
                let band = segment(start_x, end_x).expand2(vec2(0.0, grow));
                paint_hues(painter, band, [start, hue::NEUTRAL, end]);
            }
            _ => {
                painter.rect_filled(segment(start_x, end_x), TRACK_THICKNESS / 2.0, color::TRACK);
            }
        }
        if shown.is_changed && shown.enabled && self.track == Track::AccentFill {
            painter.rect_filled(
                segment(default_x, handle_x),
                TRACK_THICKNESS / 2.0,
                color::ACCENT,
            );
        }
        let resting_color = match (shown.enabled, shown.is_changed) {
            (false, _) => color::TEXT_DISABLED,
            (true, false) => color::TEXT_MUTED,
            (true, true) => color::TEXT,
        };
        let radius = egui::lerp(HANDLE_RADIUS..=HANDLE_RADIUS_ENGAGED, shown.engagement);
        let handle = pos2(handle_x, center_y);
        painter.circle_filled(handle, radius + HANDLE_GAP, color::PANEL);
        painter.circle_filled(
            handle,
            radius,
            resting_color.lerp_to_gamma(egui::Color32::WHITE, shown.engagement),
        );
    }

    fn paint_focus_ring(painter: &egui::Painter, layout: &Layout) {
        let ring = layout
            .name
            .union(layout.track_line)
            .expand2(vec2(space::XS, 2.0));
        painter.rect_stroke(
            ring,
            CONTROL_RADIUS,
            Stroke::new(1.0, color::ACCENT),
            StrokeKind::Outside,
        );
    }

    pub fn show(&self, ui: &mut egui::Ui, value: f32) -> f32 {
        let layout = self.layout(ui);
        let shown = self.interact(ui, &layout, value);
        let painter = ui.painter();
        self.paint_name_and_value(painter, &layout, &shown);
        self.paint_track(painter, layout.track_line, &shown);
        if shown.has_focus {
            Self::paint_focus_ring(painter, &layout);
        }
        shown.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPOSURE: AdjustmentSlider = AdjustmentSlider {
        label: "Exposure",
        range: -5.0..=5.0,
        default: 0.0,
        step: 0.1,
        decimals: 2,
        scale: TrackScale::Linear,
        track: Track::AccentFill,
    };
    const TEMPERATURE: AdjustmentSlider = AdjustmentSlider {
        label: "Temp",
        range: 2000.0..=50000.0,
        default: 5500.0,
        step: 50.0,
        decimals: 0,
        scale: TrackScale::Reciprocal,
        track: Track::AccentFill,
    };

    #[test]
    fn signed_values_carry_their_sign_and_zero_carries_none() {
        assert_eq!(EXPOSURE.formatted(1.25), "+1.25");
        assert_eq!(EXPOSURE.formatted(-0.5), "−0.50");
        assert_eq!(EXPOSURE.formatted(0.0), "0.00");
        assert_eq!(EXPOSURE.formatted(-0.001), "0.00");
    }

    #[test]
    fn unsigned_values_carry_no_sign() {
        assert_eq!(TEMPERATURE.formatted(5500.0), "5500");
    }

    #[test]
    fn typed_text_is_read_whatever_the_minus_or_decimal_mark() {
        assert_eq!(EXPOSURE.parsed("-1,5"), Some(-1.5));
        assert_eq!(EXPOSURE.parsed(" −1.5 "), Some(-1.5));
        assert_eq!(EXPOSURE.parsed("+2"), Some(2.0));
    }

    #[test]
    fn typed_value_outside_the_range_is_brought_back_into_it() {
        assert_eq!(EXPOSURE.parsed("12"), Some(5.0));
        assert_eq!(EXPOSURE.parsed("-12"), Some(-5.0));
    }

    #[test]
    fn reciprocal_track_gives_warm_lights_most_of_the_room() {
        assert!(TEMPERATURE.fraction(5000.0) > 0.6);
        assert!((TEMPERATURE.value_at(TEMPERATURE.fraction(5000.0)) - 5000.0).abs() < 0.5);
    }

    #[test]
    fn text_that_is_not_a_number_is_rejected() {
        assert_eq!(EXPOSURE.parsed("bright"), None);
    }
}
