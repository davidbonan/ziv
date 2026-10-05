use egui::{Color32, CursorIcon, Pos2, Rect, Sense, Stroke, Vec2, WidgetInfo, WidgetType, vec2};

use crate::develop::domain::crop_view::CropView;
use crate::develop::domain::framing::{
    ANGLE_RANGE, FRAME_HANDLES, FrameHandle, Framing, PicturePoint,
};
use crate::photo::domain::picture_region::PictureRegion;

use super::crop_section::CropTools;

pub const CROP_FRAME_LABEL: &str = "Crop frame";

/// Kept around the picture, so that a handle on its edge can be reached.
pub const CANVAS_MARGIN: f32 = 24.0;
const HANDLE_REACH: f32 = 12.0;
const CORNER_HANDLE_SIZE: Vec2 = vec2(9.0, 9.0);
const SIDE_HANDLE_LENGTH: f32 = 18.0;
const SIDE_HANDLE_THICKNESS: f32 = 5.0;
const OUTSIDE_VEIL: Color32 = Color32::from_black_alpha(150);
const FRAME_LINE: Stroke = Stroke {
    width: 1.0,
    color: Color32::WHITE,
};
// Under every light line, so that it shows on a white sky too.
const LINE_SHADE: Stroke = Stroke {
    width: 3.0,
    color: Color32::from_black_alpha(90),
};
/// How many cells across and down the grid shown while the picture turns has.
const TURNING_GRID_CELLS: usize = 9;
const THIRDS_CELLS: usize = 3;
const GRID_LINE: Stroke = Stroke {
    width: 1.0,
    color: Color32::from_rgba_premultiplied(90, 90, 90, 90),
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CropCanvasShown {
    /// The size of the picture, in pixels.
    pub picture: [u32; 2],
    pub framing: Framing,
    pub tools: CropTools,
}

pub struct CropCanvasOutput {
    /// The framing after this frame's gesture.
    pub framing: Framing,
    /// The user asked to leave crop mode, keeping the framing.
    pub is_done: bool,
    /// The line of the level tool was drawn: the tool has done its work.
    pub is_level_drawn: bool,
}

#[derive(Debug, Clone, Copy)]
enum Hold {
    Handle(FrameHandle),
    Frame,
    /// Outside the frame: the picture turns with the pointer around `centre`.
    Turn {
        centre: Pos2,
    },
}

/// What a drag took hold of, with the framing and the pointer as they were then.
#[derive(Debug, Clone, Copy)]
struct Grab {
    hold: Hold,
    framing: Framing,
    pressed_at: PicturePoint,
    pressed_on_screen: Pos2,
}

/// The picture laid out in the canvas, to go between screen points and the picture.
struct PictureOnScreen {
    view: CropView,
    area: Rect,
    pixels_per_point: f32,
    picture: [u32; 2],
}

impl PictureOnScreen {
    fn on_screen(&self, point: PicturePoint) -> Pos2 {
        self.area.min + Vec2::from(self.view.on_screen(point)) / self.pixels_per_point
    }

    fn in_picture(&self, position: Pos2) -> PicturePoint {
        self.view
            .in_picture(((position - self.area.min) * self.pixels_per_point).into())
    }

    /// The upright rectangle holding the turned picture.
    fn bounds(&self) -> Rect {
        Rect::from_min_size(
            self.area.min + Vec2::from(self.view.bounds_min()) / self.pixels_per_point,
            Vec2::from(self.view.bounds_size()) / self.pixels_per_point,
        )
    }

    fn frame_rect(&self, framing: &Framing) -> Rect {
        let [top_left, bottom_right] = [FrameHandle([-1.0, -1.0]), FrameHandle([1.0, 1.0])]
            .map(|corner| self.on_screen(framing.handle_position(self.picture, corner)));
        Rect::from_two_pos(top_left, bottom_right)
    }

    fn handle_on_screen(&self, framing: &Framing, handle: FrameHandle) -> Pos2 {
        self.on_screen(framing.handle_position(self.picture, handle))
    }

    fn hold_under(&self, framing: &Framing, pointer: Pos2) -> Hold {
        let distance_to =
            |handle: &FrameHandle| self.handle_on_screen(framing, *handle).distance(pointer);
        let nearest = FRAME_HANDLES
            .into_iter()
            .min_by(|a, b| distance_to(a).total_cmp(&distance_to(b)))
            .unwrap_or(FRAME_HANDLES[0]);
        if distance_to(&nearest) <= HANDLE_REACH {
            return Hold::Handle(nearest);
        }
        if framing.holds(self.picture, self.in_picture(pointer)) {
            return Hold::Frame;
        }
        Hold::Turn {
            centre: self.on_screen(framing.frame_centre(self.picture)),
        }
    }

    /// The cursor of what is held, a handle resizing the way it lies on screen.
    fn cursor_of(&self, framing: &Framing, hold: Hold) -> CursorIcon {
        let handle = match hold {
            Hold::Handle(handle) => handle,
            Hold::Frame => return CursorIcon::Grab,
            Hold::Turn { .. } => return CursorIcon::Alias,
        };
        let centre = self.on_screen(framing.frame_centre(self.picture));
        resize_cursor(self.handle_on_screen(framing, handle) - centre)
    }
}

fn resize_cursor(from_centre: Vec2) -> CursorIcon {
    let [across, down] = [from_centre.x, from_centre.y].map(|reach| match reach.abs() < 0.5 {
        true => 0.0,
        false => reach.signum(),
    });
    match (across != 0.0, down != 0.0) {
        (true, false) => CursorIcon::ResizeHorizontal,
        (false, true) => CursorIcon::ResizeVertical,
        _ if across == down => CursorIcon::ResizeNwSe,
        _ => CursorIcon::ResizeNeSw,
    }
}

fn shaded_line(painter: &egui::Painter, ends: [Pos2; 2], line: Stroke) {
    painter.line_segment(ends, LINE_SHADE);
    painter.line_segment(ends, line);
}

// Over the whole canvas: the corners a turned picture leaves empty get no edge of their own.
fn paint_outside_veil(painter: &egui::Painter, frame: Rect) {
    let canvas = painter.clip_rect();
    let beside_frame = frame.y_range();
    let around = [
        Rect::from_x_y_ranges(canvas.x_range(), canvas.top()..=frame.top()),
        Rect::from_x_y_ranges(canvas.x_range(), frame.bottom()..=canvas.bottom()),
        Rect::from_x_y_ranges(canvas.left()..=frame.left(), beside_frame),
        Rect::from_x_y_ranges(frame.right()..=canvas.right(), beside_frame),
    ];
    for part in around {
        painter.rect_filled(part, 0.0, OUTSIDE_VEIL);
    }
}

fn paint_grid(painter: &egui::Painter, frame: Rect, cells: usize) {
    for line in 1..cells {
        let share = line as f32 / cells as f32;
        let across = egui::lerp(frame.x_range(), share);
        let down = egui::lerp(frame.y_range(), share);
        painter.vline(across, frame.y_range(), GRID_LINE);
        painter.hline(frame.x_range(), down, GRID_LINE);
    }
}

fn paint_handle(painter: &egui::Painter, centre: Pos2, FrameHandle([across, down]): FrameHandle) {
    let size = match (across != 0.0, down != 0.0) {
        (true, false) => vec2(SIDE_HANDLE_THICKNESS, SIDE_HANDLE_LENGTH),
        (false, true) => vec2(SIDE_HANDLE_LENGTH, SIDE_HANDLE_THICKNESS),
        _ => CORNER_HANDLE_SIZE,
    };
    let shape = Rect::from_center_size(centre, size);
    painter.rect_filled(shape.expand(1.5), 1.0, LINE_SHADE.color);
    painter.rect_filled(shape, 1.0, Color32::WHITE);
}

fn paint_frame(painter: &egui::Painter, frame: Rect, grid_cells: usize) {
    paint_outside_veil(painter, frame);
    paint_grid(painter, frame, grid_cells);
    let corners = [
        frame.left_top(),
        frame.right_top(),
        frame.right_bottom(),
        frame.left_bottom(),
    ];
    for side in 0..corners.len() {
        let ends = [corners[side], corners[(side + 1) % corners.len()]];
        shaded_line(painter, ends, FRAME_LINE);
    }
    for handle in FRAME_HANDLES {
        let from_centre = Vec2::from(handle.0) * frame.size() / 2.0;
        paint_handle(painter, frame.center() + from_centre, handle);
    }
}

/// The angle a pointer gives a picture held outside its frame: it turns as
/// the pointer does around the centre of the frame.
fn turned_angle(grab: &Grab, centre: Pos2, pointer: Pos2) -> f32 {
    let turn = (pointer - centre).angle() - (grab.pressed_on_screen - centre).angle();
    let degrees =
        egui::emath::normalized_angle(turn).to_degrees() * grab.framing.turn.angle_sense();
    (grab.framing.angle + degrees).clamp(*ANGLE_RANGE.start(), *ANGLE_RANGE.end())
}

/// What the drag of this frame holds, taken at its press and let go at its release.
fn grab_of(
    ui: &egui::Ui,
    on_screen: &PictureOnScreen,
    framing: Framing,
    response: &egui::Response,
) -> Option<Grab> {
    let grab_id = response.id.with("grab");
    if response.drag_started() {
        // A drag is known only once the pointer has moved: it began at the press.
        let pressed_at = ui.input(|input| input.pointer.press_origin());
        let grab = pressed_at.map(|pressed_at| Grab {
            hold: on_screen.hold_under(&framing, pressed_at),
            framing,
            pressed_at: on_screen.in_picture(pressed_at),
            pressed_on_screen: pressed_at,
        });
        ui.data_mut(|data| data.insert_temp(grab_id, grab));
    }
    let grab: Option<Grab> = ui.data(|data| data.get_temp(grab_id)).flatten();
    if response.drag_stopped() {
        ui.data_mut(|data| data.insert_temp(grab_id, None::<Grab>));
    }
    grab
}

/// The framing after this frame's drag: a held handle follows the pointer, a
/// held frame moves with it, a picture held outside its frame turns.
fn dragged(
    on_screen: &PictureOnScreen,
    shown: &CropCanvasShown,
    grab: &Grab,
    pointer: Pos2,
) -> Framing {
    let CropCanvasShown {
        picture,
        framing,
        tools,
    } = *shown;
    let in_picture = on_screen.in_picture(pointer);
    match grab.hold {
        Hold::Handle(handle) if tools.ratio.is_locked() => {
            framing.with_handle_at_keeping_ratio(picture, handle, in_picture)
        }
        Hold::Handle(handle) => framing.with_handle_at(picture, handle, in_picture),
        Hold::Frame => {
            let shift = [0, 1].map(|axis| in_picture[axis] - grab.pressed_at[axis]);
            grab.framing.moved_by(picture, shift)
        }
        Hold::Turn { centre } => grab
            .framing
            .straightened(picture, turned_angle(grab, centre, pointer)),
    }
}

/// The level tool: a drag draws a line on the picture; at its release the
/// picture is turned so that the line is level or upright. Returns the
/// framing and whether the line was drawn.
fn levelled(
    ui: &egui::Ui,
    on_screen: &PictureOnScreen,
    framing: Framing,
    response: &egui::Response,
) -> (Framing, bool) {
    ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
    let line_start = response.id.with("level line start");
    if response.drag_started() {
        // Kept: the press is forgotten by the time the drag is released.
        let pressed_at = ui.input(|input| input.pointer.press_origin());
        ui.data_mut(|data| data.insert_temp(line_start, pressed_at));
    }
    let from: Option<Pos2> = ui.data(|data| data.get_temp(line_start)).flatten();
    let pointer = ui.input(|input| input.pointer.latest_pos());
    let (Some(from), Some(to)) = (from, pointer) else {
        return (framing, false);
    };
    if response.dragged() {
        shaded_line(&ui.painter_at(response.rect), [from, to], FRAME_LINE);
    }
    if !response.drag_stopped() {
        return (framing, false);
    }
    let line = [from, to].map(|end| on_screen.in_picture(end));
    (framing.levelled(on_screen.picture, line), true)
}

/// Whether the angle is not the one this canvas showed at the frame before.
fn is_angle_changing(ui: &egui::Ui, canvas: egui::Id, angle: f32) -> bool {
    let shown_id = canvas.with("angle shown");
    let shown: Option<f32> = ui.data(|data| data.get_temp(shown_id));
    ui.data_mut(|data| data.insert_temp(shown_id, angle));
    shown.is_some_and(|shown| shown != angle)
}

fn paint_picture(
    ui: &egui::Ui,
    canvas: Rect,
    on_screen: &PictureOnScreen,
    display_texture: impl FnOnce(PictureRegion, [u32; 2]) -> egui::TextureId,
) {
    let whole_texture = Rect::from_min_max(Pos2::ZERO, egui::pos2(1.0, 1.0));
    let rendered_size = on_screen.view.bounds_size().map(|side| side as u32);
    ui.painter_at(canvas).image(
        display_texture(on_screen.view.region(), rendered_size),
        on_screen.bounds(),
        whole_texture,
        Color32::WHITE,
    );
}

/// Shows the whole picture, turned so that its crop frame is upright, and
/// lets the pointer resize and move the frame and turn the picture.
/// `display_texture` returns a region of the picture rendered at a size in pixels.
pub fn crop_canvas(
    ui: &mut egui::Ui,
    shown: &CropCanvasShown,
    display_texture: impl FnOnce(PictureRegion, [u32; 2]) -> egui::TextureId,
) -> CropCanvasOutput {
    let CropCanvasShown {
        picture, framing, ..
    } = *shown;
    let pixels_per_point = ui.pixels_per_point();
    let canvas = ui.available_rect_before_wrap();
    let response = ui.allocate_rect(canvas, Sense::click_and_drag());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Image, true, CROP_FRAME_LABEL));
    let area = canvas.shrink(CANVAS_MARGIN);
    let laid_out = |framing: &Framing| PictureOnScreen {
        view: CropView::fitting(picture, framing, (area.size() * pixels_per_point).into()),
        area,
        pixels_per_point,
        picture,
    };

    if shown.tools.is_level_armed {
        let on_screen = laid_out(&framing);
        paint_picture(ui, canvas, &on_screen, display_texture);
        let frame = on_screen.frame_rect(&framing);
        paint_frame(&ui.painter_at(canvas), frame, THIRDS_CELLS);
        let (framing, is_level_drawn) = levelled(ui, &on_screen, framing, &response);
        return CropCanvasOutput {
            framing,
            is_done: false,
            is_level_drawn,
        };
    }

    let grab = grab_of(ui, &laid_out(&framing), framing, &response);
    let framing = match (&grab, response.interact_pointer_pos()) {
        (Some(grab), Some(pointer)) => dragged(&laid_out(&grab.framing), shown, grab, pointer),
        _ => framing,
    };
    let on_screen = laid_out(&framing);
    let hovered = ui
        .input(|input| input.pointer.hover_pos())
        .filter(|_| response.contains_pointer())
        .map(|pointer| on_screen.hold_under(&framing, pointer));
    if let Some(hold) = grab.map(|grab| grab.hold).or(hovered) {
        ui.ctx()
            .set_cursor_icon(on_screen.cursor_of(&framing, hold));
    }
    let is_done = response.double_clicked() && matches!(hovered, Some(Hold::Frame));
    let is_held_to_turn = grab.is_some_and(|grab| matches!(grab.hold, Hold::Turn { .. }));
    let is_turning = is_held_to_turn || is_angle_changing(ui, response.id, framing.angle);

    paint_picture(ui, canvas, &on_screen, display_texture);
    let grid_cells = match is_turning {
        true => TURNING_GRID_CELLS,
        false => THIRDS_CELLS,
    };
    let frame = on_screen.frame_rect(&framing);
    paint_frame(&ui.painter_at(canvas), frame, grid_cells);
    CropCanvasOutput {
        framing,
        is_done,
        is_level_drawn: false,
    }
}
