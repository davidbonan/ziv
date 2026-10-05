use egui::{Color32, Pos2, Sense, Stroke, Vec2, WidgetInfo, WidgetType, vec2};

use crate::design::ui::theme::color;
use crate::develop::domain::color_grading::{GRADE_SATURATION_RANGE, ZoneGrade, wheel_color};

const RIM_STEPS: usize = 72;
const POINT_RADIUS: f32 = 4.0;
const CENTRE: Color32 = Color32::from_gray(0x80);
const DISABLED_OPACITY: f32 = 0.35;

pub struct Wheel<'a> {
    /// What assistive technology reads.
    pub name: &'a str,
    pub diameter: f32,
}

/// The disc a point is placed in: the wheel without the room its point needs at the rim.
struct Disc {
    centre: Pos2,
    radius: f32,
}

impl Disc {
    fn direction(hue: f32) -> Vec2 {
        let (sine, cosine) = hue.to_radians().sin_cos();
        vec2(cosine, -sine)
    }

    fn place_of(&self, grade: &ZoneGrade) -> Pos2 {
        let full = *GRADE_SATURATION_RANGE.end();
        self.centre + Self::direction(grade.hue) * self.radius * (grade.saturation / full)
    }

    /// The grade with its point under the pointer: the angle is the hue, the
    /// distance from the centre the saturation, full at the rim and beyond.
    fn placed(&self, pointer: Pos2, grade: ZoneGrade) -> ZoneGrade {
        let from_centre = pointer - self.centre;
        let full = *GRADE_SATURATION_RANGE.end();
        ZoneGrade {
            hue: (-from_centre.y)
                .atan2(from_centre.x)
                .to_degrees()
                .rem_euclid(360.0),
            saturation: (from_centre.length() / self.radius).min(1.0) * full,
            ..grade
        }
    }

    fn paint(&self, painter: &egui::Painter) {
        let mut mesh = egui::Mesh::default();
        mesh.colored_vertex(self.centre, CENTRE);
        for step in 0..=RIM_STEPS {
            let hue = 360.0 * step as f32 / RIM_STEPS as f32;
            let [red, green, blue] = wheel_color(hue).map(|channel| (channel * 255.0) as u8);
            let rim = self.centre + Self::direction(hue) * self.radius;
            mesh.colored_vertex(rim, Color32::from_rgb(red, green, blue));
        }
        for step in 1..=RIM_STEPS as u32 {
            mesh.add_triangle(0, step, step + 1);
        }
        painter.add(mesh);
    }
}

/// A disc of all hues, grey at its centre, and the point of a tonal zone in
/// it. Returns the grade as the user left it this frame: a press or a drag
/// places the point, a double click returns it to the centre.
pub fn color_wheel(ui: &mut egui::Ui, wheel: &Wheel<'_>, grade: ZoneGrade) -> ZoneGrade {
    let (area, response) =
        ui.allocate_exact_size(Vec2::splat(wheel.diameter), Sense::click_and_drag());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Other, ui.is_enabled(), wheel.name));
    let disc = Disc {
        centre: area.center(),
        radius: wheel.diameter / 2.0 - POINT_RADIUS,
    };
    let held = response
        .interact_pointer_pos()
        .filter(|_| response.is_pointer_button_down_on());
    let left = match held {
        _ if response.double_clicked() => ZoneGrade {
            hue: 0.0,
            saturation: 0.0,
            ..grade
        },
        Some(pointer) => disc.placed(pointer, grade),
        None => grade,
    };

    let mut painter = ui.painter().clone();
    if !ui.is_enabled() {
        painter.set_opacity(DISABLED_OPACITY);
    }
    disc.paint(&painter);
    let point_fill = match held.is_some() || response.hovered() {
        true => color::ACCENT,
        false => color::PANEL,
    };
    let outline = Stroke::new(1.5, color::TEXT);
    painter.circle(disc.place_of(&left), POINT_RADIUS, point_fill, outline);
    if response.has_focus() {
        let ring = Stroke::new(1.0, color::ACCENT);
        painter.circle_stroke(disc.centre, wheel.diameter / 2.0, ring);
    }
    left
}
