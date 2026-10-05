use egui::{Color32, Rect, Sense, WidgetInfo, WidgetType, pos2, vec2};

use crate::design::ui::theme::{CONTROL_RADIUS, color};
use crate::histogram::domain::histogram::{Histogram, LEVEL_COUNT, LevelHeights};

pub const HISTOGRAM_LABEL: &str = "Histogram";
pub const HISTOGRAM_HEIGHT: f32 = 64.0;

const CHANNELS_FILL: Color32 = Color32::from_rgba_premultiplied(76, 76, 76, 140);
const LUMINANCE_FILL: Color32 = Color32::from_rgba_premultiplied(72, 72, 72, 89);

fn shape(area: Rect, heights: &LevelHeights, fill: Color32) -> egui::Mesh {
    let mut mesh = egui::Mesh::default();
    let last_level = (LEVEL_COUNT - 1) as f32;
    for (level, height) in heights.iter().enumerate() {
        let x = area.left() + area.width() * level as f32 / last_level;
        mesh.colored_vertex(pos2(x, area.bottom() - area.height() * height), fill);
        mesh.colored_vertex(pos2(x, area.bottom()), fill);
    }
    for level in 0..LEVEL_COUNT as u32 - 1 {
        let top = level * 2;
        mesh.add_triangle(top, top + 1, top + 2);
        mesh.add_triangle(top + 1, top + 2, top + 3);
    }
    mesh
}

/// The channels shape and, over it, the luminance shape, across `area`.
pub fn paint_histogram(painter: &egui::Painter, area: Rect, histogram: &Histogram) {
    painter.add(shape(area, &histogram.channel_heights(), CHANNELS_FILL));
    painter.add(shape(area, &histogram.luminance_heights(), LUMINANCE_FILL));
}

/// The place of the histogram, empty while there is none to show.
pub fn histogram_plot(ui: &mut egui::Ui, histogram: Option<&Histogram>) {
    let size = vec2(ui.available_width(), HISTOGRAM_HEIGHT);
    let (area, response) = ui.allocate_exact_size(size, Sense::hover());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Image, true, HISTOGRAM_LABEL));
    let painter = ui.painter().with_clip_rect(area);
    painter.rect_filled(area, CONTROL_RADIUS, color::CANVAS);
    let Some(histogram) = histogram else {
        return;
    };
    paint_histogram(&painter, area, histogram);
}
