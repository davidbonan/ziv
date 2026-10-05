use egui::{Color32, CursorIcon, Key, Modifiers, Pos2, Rect, Sense, Stroke, Vec2};

use crate::develop::domain::brush::{Brush, BrushMask, SIZE_STEP, Stroke as BrushStroke};
use crate::develop::domain::framing::Framing;
use crate::develop::domain::linear_gradient::LinearGradient;
use crate::develop::domain::mask::{Mask, MaskKind, MaskShape, PhotoPoint};
use crate::develop::domain::polygon::{FEWEST_CORNERS, MOST_CORNERS, Polygon};

use super::masks_section::MaskSelection;

const HANDLE_RADIUS: f32 = 5.0;
const HANDLE_REACH: f32 = 12.0;
const GUIDE: Stroke = Stroke {
    width: 1.0,
    color: Color32::WHITE,
};
// Under every white line, so that it shows on a white sky too.
const GUIDE_SHADE: Stroke = Stroke {
    width: 3.0,
    color: Color32::from_black_alpha(90),
};
const PAN_KEY: Key = Key::Space;
const OUTLINE_POINTS: usize = 96;
/// In screen points: closer pointer positions add nothing to a stroke.
const STROKE_POINT_SPACING: f32 = 2.0;

/// Where the photo lies on screen, to go between screen and photo points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhotoOnScreen {
    /// The screen area the photo is looked at through.
    pub area: Rect,
    /// The top-left corner of the whole picture, out of sight or out of the frame maybe.
    picture_origin: Pos2,
    /// The top and the left side of the whole picture, from that corner.
    picture_across: Vec2,
    picture_down: Vec2,
}

impl PhotoOnScreen {
    /// A photo shown as its picture is, `whole_picture` being where it lies.
    pub fn upright(area: Rect, whole_picture: Rect) -> Self {
        Self {
            area,
            picture_origin: whole_picture.min,
            picture_across: Vec2::new(whole_picture.width(), 0.0),
            picture_down: Vec2::new(0.0, whole_picture.height()),
        }
    }

    /// A photo framed by `framing`, `framed_photo` being where the whole
    /// framed photo lies and `picture` the size of its picture.
    pub fn framed(
        area: Rect,
        framed_photo: Rect,
        (framing, picture): (&Framing, [u32; 2]),
    ) -> Self {
        let [width, height] = picture.map(|side| side as f32);
        let on_screen = |point: [f32; 2]| {
            let share = Vec2::from(framing.share_of_framed_photo(picture, point));
            framed_photo.min + share * framed_photo.size()
        };
        let picture_origin = on_screen([0.0, 0.0]);
        Self {
            area,
            picture_origin,
            picture_across: on_screen([width, 0.0]) - picture_origin,
            picture_down: on_screen([0.0, height]) - picture_origin,
        }
    }

    fn long_edge(&self) -> f32 {
        self.picture_across.length().max(self.picture_down.length())
    }

    pub fn on_screen(&self, point: PhotoPoint) -> Pos2 {
        let [across, down] = point.map(|reach| reach * self.long_edge());
        self.picture_origin
            + self.picture_across.normalized() * across
            + self.picture_down.normalized() * down
    }

    pub fn in_photo(&self, position: Pos2) -> PhotoPoint {
        let from_origin = position - self.picture_origin;
        [self.picture_across, self.picture_down]
            .map(|side| from_origin.dot(side.normalized()) / self.long_edge())
    }

    /// Where a point of the picture, in shares of its width and height, is on screen.
    pub fn share_on_screen(&self, [across, down]: [f32; 2]) -> Pos2 {
        self.picture_origin + self.picture_across * across + self.picture_down * down
    }
}

/// The masks of the photo and what is being done to them.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MaskCanvasState {
    pub masks: Vec<Mask>,
    pub selection: MaskSelection,
}

fn guide(painter: &egui::Painter, from: Pos2, to: Pos2) {
    painter.line_segment([from, to], GUIDE_SHADE);
    painter.line_segment([from, to], GUIDE);
}

fn paint_handle(painter: &egui::Painter, centre: Pos2) {
    painter.circle_filled(centre, HANDLE_RADIUS + 1.5, GUIDE_SHADE.color);
    painter.circle_filled(centre, HANDLE_RADIUS, Color32::WHITE);
}

// The two lines the gradient runs between, and the segment joining its ends.
fn paint_linear_gradient(
    painter: &egui::Painter,
    photo: &PhotoOnScreen,
    gradient: &LinearGradient,
) {
    let [full, none] = [gradient.full, gradient.none].map(|end| photo.on_screen(end));
    let across = (none - full).rot90().normalized() * photo.area.size().length();
    guide(painter, full, none);
    if across.is_finite() {
        guide(painter, full - across, full + across);
        guide(painter, none - across, none + across);
    }
}

fn paint_outline(painter: &egui::Painter, photo: &PhotoOnScreen, outline: &[PhotoPoint]) {
    let on_screen: Vec<Pos2> = outline
        .iter()
        .map(|point| photo.on_screen(*point))
        .collect();
    painter.add(egui::Shape::closed_line(on_screen.clone(), GUIDE_SHADE));
    painter.add(egui::Shape::closed_line(on_screen, GUIDE));
}

fn paint_shape(painter: &egui::Painter, photo: &PhotoOnScreen, shape: &MaskShape) {
    match shape {
        MaskShape::LinearGradient(gradient) => paint_linear_gradient(painter, photo, gradient),
        MaskShape::RadialGradient(gradient) => {
            let [.., top, rotation] = gradient.handles();
            guide(painter, photo.on_screen(top), photo.on_screen(rotation));
            paint_outline(painter, photo, &gradient.outline(OUTLINE_POINTS));
        }
        MaskShape::Rectangle(rectangle) => paint_outline(painter, photo, &rectangle.corners()),
        MaskShape::Polygon(polygon) => paint_outline(painter, photo, &polygon.corners),
        MaskShape::Brush(_) | MaskShape::Zone(_) => {}
    }
    for handle in shape.handles() {
        paint_handle(painter, photo.on_screen(handle));
    }
}

fn handle_under(photo: &PhotoOnScreen, shape: &MaskShape, pointer: Pos2) -> Option<usize> {
    let distance_to = |handle: &PhotoPoint| photo.on_screen(*handle).distance(pointer);
    let handles = shape.handles();
    let nearest = (0..handles.len())
        .min_by(|a, b| distance_to(&handles[*a]).total_cmp(&distance_to(&handles[*b])))?;
    (distance_to(&handles[nearest]) <= HANDLE_REACH).then_some(nearest)
}

#[derive(Debug, Clone, Copy)]
struct Grab {
    mask: usize,
    handle: usize,
}

/// What a drag starting at `pointer` takes hold of: the shape the armed tool
/// starts there, else a handle of the selected mask.
fn grabbed(photo: &PhotoOnScreen, state: &mut MaskCanvasState, pointer: Pos2) -> Option<Grab> {
    if let Some(tool) = state.selection.armed_tool {
        let (shape, handle) = MaskShape::drawn_from(tool, photo.in_photo(pointer))?;
        state.masks.push(Mask::of(shape));
        let mask = state.masks.len() - 1;
        state.selection = state.selection.selecting(Some(mask));
        return Some(Grab { mask, handle });
    }
    let mask = state.selection.selected?;
    let handle = handle_under(photo, &state.masks.get(mask)?.shape, pointer)?;
    Some(Grab { mask, handle })
}

fn is_panning(ui: &egui::Ui) -> bool {
    !ui.ctx().text_edit_focused() && ui.input(|input| input.key_down(PAN_KEY))
}

fn is_closing(ui: &egui::Ui, photo: &PhotoOnScreen, corners: &[PhotoPoint]) -> bool {
    let clicked = ui.input(|input| input.pointer.interact_pos());
    let is_first_corner_clicked = match (corners.first(), clicked) {
        (Some(first), Some(clicked)) if corners.len() >= FEWEST_CORNERS => {
            photo.on_screen(*first).distance(clicked) <= HANDLE_REACH
        }
        _ => false,
    };
    is_first_corner_clicked || corners.len() == MOST_CORNERS
}

fn paint_polygon_being_drawn(ui: &egui::Ui, photo: &PhotoOnScreen, corners: &[PhotoPoint]) {
    let painter = ui.painter_at(photo.area);
    let pointer = ui.input(|input| input.pointer.hover_pos());
    let mut line: Vec<Pos2> = corners
        .iter()
        .map(|corner| photo.on_screen(*corner))
        .collect();
    line.extend(pointer);
    painter.add(egui::Shape::line(line.clone(), GUIDE_SHADE));
    painter.add(egui::Shape::line(line, GUIDE));
    for corner in corners {
        paint_handle(&painter, photo.on_screen(*corner));
    }
}

/// The polygon tool: each click adds a corner; a click on the first corner, a
/// double-click or Enter closes the outline into a mask.
fn polygon_drawn(
    ui: &mut egui::Ui,
    photo: &PhotoOnScreen,
    state: MaskCanvasState,
    response: &egui::Response,
) -> MaskCanvasState {
    let mut state = state;
    let corners_id = response.id.with("polygon corners");
    let mut corners: Vec<PhotoPoint> = ui
        .data(|data| data.get_temp(corners_id))
        .unwrap_or_default();
    let is_enter_pressed = ui.input_mut(|input| input.consume_key(Modifiers::NONE, Key::Enter));
    let clicked_at = response
        .interact_pointer_pos()
        .filter(|_| response.clicked());
    let is_closed = is_enter_pressed
        || response.double_clicked()
        || (clicked_at.is_some() && is_closing(ui, photo, &corners));
    if let (Some(clicked), false) = (clicked_at, is_closed) {
        corners.push(photo.in_photo(clicked));
    }
    let closed = is_closed
        .then(|| corners.clone())
        .and_then(Polygon::closing);
    if let Some(polygon) = closed {
        state.masks.push(Mask::of(MaskShape::Polygon(polygon)));
        state.selection = state.selection.selecting(Some(state.masks.len() - 1));
        corners.clear();
    }
    paint_polygon_being_drawn(ui, photo, &corners);
    ui.data_mut(|data| data.insert_temp(corners_id, corners));
    state
}

fn brush_mask_painted_on(state: &mut MaskCanvasState) -> Option<&mut BrushMask> {
    match &mut state.masks.get_mut(state.selection.selected?)?.shape {
        MaskShape::Brush(brush) => Some(brush),
        _ => None,
    }
}

fn is_brush_at_work(state: &MaskCanvasState) -> bool {
    let selected = state.selection.selected_shape(&state.masks);
    state.selection.armed_tool == Some(MaskKind::Brush)
        || matches!(selected, Some(MaskShape::Brush(_)))
}

// The size and the heart of the brush around the pointer.
fn paint_brush(ui: &egui::Ui, photo: &PhotoOnScreen, brush: &Brush) {
    let Some(pointer) = ui.input(|input| input.pointer.hover_pos()) else {
        return;
    };
    let painter = ui.painter_at(photo.area);
    let radius = brush.radius() * photo.long_edge();
    let heart = radius * (1.0 - brush.feather / 100.0);
    painter.circle_stroke(pointer, radius, GUIDE_SHADE);
    painter.circle_stroke(pointer, radius, GUIDE);
    painter.circle_stroke(
        pointer,
        heart,
        Stroke::new(1.0, Color32::from_white_alpha(110)),
    );
}

fn begin_stroke(state: &mut MaskCanvasState, stroke: BrushStroke) {
    if state.selection.armed_tool == Some(MaskKind::Brush) {
        state
            .masks
            .push(Mask::of(MaskShape::Brush(BrushMask::default())));
        state.selection = state.selection.selecting(Some(state.masks.len() - 1));
    }
    if let Some(painted) = brush_mask_painted_on(state) {
        painted.strokes.push(stroke);
    }
}

fn continue_stroke(photo: &PhotoOnScreen, state: &mut MaskCanvasState, pointer: Pos2) {
    let last_stroke = brush_mask_painted_on(state).and_then(|mask| mask.strokes.last_mut());
    let Some(stroke) = last_stroke else {
        return;
    };
    let is_further = stroke
        .points
        .last()
        .is_none_or(|last| photo.on_screen(*last).distance(pointer) >= STROKE_POINT_SPACING);
    if is_further {
        stroke.points.push(photo.in_photo(pointer));
    }
}

/// The brush: each press-drag-release adds a stroke, erasing while Alt is held;
/// `[` and `]` change its size.
fn brushed(
    ui: &mut egui::Ui,
    photo: &PhotoOnScreen,
    state: MaskCanvasState,
    response: &egui::Response,
) -> MaskCanvasState {
    let mut state = state;
    let is_pressed = |key| ui.input_mut(|input| input.consume_key(Modifiers::NONE, key));
    let steps = i8::from(is_pressed(Key::CloseBracket)) - i8::from(is_pressed(Key::OpenBracket));
    state.selection.brush = state
        .selection
        .brush
        .resized_by(f32::from(steps) * SIZE_STEP);

    let pressed_at = ui.input(|input| input.pointer.press_origin());
    let began_at = match (response.drag_started(), response.clicked()) {
        (true, _) => pressed_at,
        (false, true) => response.interact_pointer_pos(),
        (false, false) => None,
    };
    if let Some(began_at) = began_at {
        let stroke = BrushStroke {
            is_erasing: ui.input(|input| input.modifiers.alt),
            ..state.selection.brush.stroke_from(photo.in_photo(began_at))
        };
        begin_stroke(&mut state, stroke);
    }
    if let Some(pointer) = response
        .interact_pointer_pos()
        .filter(|_| response.dragged())
    {
        continue_stroke(photo, &mut state, pointer);
    }
    paint_brush(ui, photo, &state.selection.brush);
    state
}

/// The masks after this frame's drag: a grabbed handle follows the pointer.
fn dragged(ui: &mut egui::Ui, photo: &PhotoOnScreen, state: MaskCanvasState) -> MaskCanvasState {
    let mut state = state;
    let canvas = ui.id().with("mask canvas");
    let response = ui.interact(photo.area, canvas, Sense::click_and_drag());
    if state.selection.armed_tool.is_some() {
        ui.ctx().set_cursor_icon(CursorIcon::Crosshair);
    }
    if state.selection.armed_tool == Some(MaskKind::Polygon) {
        return polygon_drawn(ui, photo, state, &response);
    }
    ui.data_mut(|data| data.remove_temp::<Vec<PhotoPoint>>(response.id.with("polygon corners")));
    if is_brush_at_work(&state) {
        return brushed(ui, photo, state, &response);
    }
    let Some(pointer) = response.interact_pointer_pos() else {
        return state;
    };
    let grab_id = canvas.with("grab");
    if response.drag_started() {
        // A drag is known only once the pointer has moved: it began at the press.
        let pressed_at = ui.input(|input| input.pointer.press_origin());
        let grab = grabbed(photo, &mut state, pressed_at.unwrap_or(pointer));
        ui.data_mut(|data| data.insert_temp(grab_id, grab));
    }
    let grab: Option<Grab> = ui.data(|data| data.get_temp(grab_id)).flatten();
    let grabbed_mask = grab.and_then(|grab| Some((state.masks.get_mut(grab.mask)?, grab.handle)));
    if let Some((mask, handle)) = grabbed_mask.filter(|_| response.dragged()) {
        mask.shape = mask.shape.with_handle_at(handle, photo.in_photo(pointer));
    }
    if response.drag_stopped() {
        ui.data_mut(|data| data.insert_temp(grab_id, None::<Grab>));
    }
    state
}

/// Draws the selected mask's shape over the photo and lets the pointer draw a
/// new mask or reshape the selected one. Holding Space leaves the pointer to
/// the photo underneath.
pub fn mask_canvas(
    ui: &mut egui::Ui,
    photo: &PhotoOnScreen,
    state: MaskCanvasState,
) -> MaskCanvasState {
    if state.selection.armed_tool.is_none() && state.selection.selected.is_none() {
        return state;
    }
    let state = match is_panning(ui) {
        true => state,
        false => dragged(ui, photo, state),
    };
    let selected = state
        .selection
        .selected
        .and_then(|index| state.masks.get(index));
    if let Some(mask) = selected {
        paint_shape(&ui.painter_at(photo.area), photo, &mask.shape);
    }
    state
}
