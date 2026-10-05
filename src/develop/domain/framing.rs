use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::photo::domain::picture_region::PictureRegion;

/// A point of the picture, in pixels from its top-left.
pub type PicturePoint = [f32; 2];

/// The shortest a side of the crop frame gets, in shares of the picture's long edge.
const SHORTEST_SIDE: f32 = 1.0 / 50.0;

/// The rectangle of the picture that is kept, in shares of the picture's
/// width and height.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CropFrame {
    pub centre: [f32; 2],
    pub size: [f32; 2],
}

impl Default for CropFrame {
    fn default() -> Self {
        Self {
            centre: [0.5; 2],
            size: [1.0; 2],
        }
    }
}

/// A handle of the crop frame, named by the side it holds on each axis:
/// −1 the left or top one, 1 the right or bottom one, 0 neither.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameHandle(pub [f32; 2]);

pub const FRAME_HANDLES: [FrameHandle; 8] = [
    FrameHandle([-1.0, -1.0]),
    FrameHandle([0.0, -1.0]),
    FrameHandle([1.0, -1.0]),
    FrameHandle([1.0, 0.0]),
    FrameHandle([1.0, 1.0]),
    FrameHandle([0.0, 1.0]),
    FrameHandle([-1.0, 1.0]),
    FrameHandle([-1.0, 0.0]),
];

/// The shortest line the level tool reads, in shares of the picture's long edge.
const SHORTEST_LEVEL_LINE: f32 = 1.0 / 100.0;

/// How far the picture is turned under the frame, in degrees.
pub const ANGLE_RANGE: RangeInclusive<f32> = -45.0..=45.0;

/// One of the eight ways a framed photo is turned: mirrored left to right
/// first, then turned clockwise by quarter turns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Turn {
    pub is_mirrored: bool,
    /// 0 to 3.
    pub quarter_turns: u8,
}

impl Turn {
    fn turned_by(self, quarter_turns: u8) -> Self {
        Self {
            quarter_turns: (self.quarter_turns + quarter_turns) % 4,
            ..self
        }
    }

    pub fn turned_right(self) -> Self {
        self.turned_by(1)
    }

    pub fn turned_left(self) -> Self {
        self.turned_by(3)
    }

    /// Mirrored left to right, as it is seen once turned.
    pub fn flipped_horizontally(self) -> Self {
        Self {
            is_mirrored: !self.is_mirrored,
            quarter_turns: (4 - self.quarter_turns % 4) % 4,
        }
    }

    /// Mirrored top to bottom, as it is seen once turned.
    pub fn flipped_vertically(self) -> Self {
        self.flipped_horizontally().turned_by(2)
    }

    pub fn swaps_width_and_height(self) -> bool {
        self.quarter_turns % 2 == 1
    }

    /// 1, or −1 when the mirror makes a clockwise turn of the picture show anticlockwise.
    pub fn angle_sense(self) -> f32 {
        match self.is_mirrored {
            true => -1.0,
            false => 1.0,
        }
    }

    /// Where a step to the right and a step down on the turned photo lead on
    /// the photo before its turn.
    fn steps(self) -> [[f32; 2]; 2] {
        [[1.0, 0.0], [0.0, 1.0]].map(|step| {
            let [across, down] =
                (0..self.quarter_turns % 4).fold(step, |[across, down], _| [down, -across]);
            [across * self.angle_sense(), down]
        })
    }
}

/// What a photo keeps of its picture and how it is turned. The default keeps
/// the whole picture.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Framing {
    pub frame: CropFrame,
    /// Degrees the picture is turned by, clockwise, under the frame.
    pub angle: f32,
    /// How the framed photo is turned, last.
    pub turn: Turn,
}

fn in_pixels(picture: [u32; 2]) -> [f32; 2] {
    picture.map(|side| side as f32)
}

/// The frame in pixels of its picture: its sides run along `across` and
/// `down`, which the angle tilts.
#[derive(Debug, Clone, Copy)]
struct FrameInPicture {
    picture: [f32; 2],
    centre: PicturePoint,
    half_size: [f32; 2],
    across: [f32; 2],
    down: [f32; 2],
}

impl FrameInPicture {
    fn of(framing: &Framing, picture: [u32; 2]) -> Self {
        let picture = in_pixels(picture);
        let (sine, cosine) = framing.angle.to_radians().sin_cos();
        Self {
            picture,
            centre: [0, 1].map(|axis| framing.frame.centre[axis] * picture[axis]),
            half_size: [0, 1].map(|axis| framing.frame.size[axis] * picture[axis] / 2.0),
            across: [cosine, -sine],
            down: [sine, cosine],
        }
    }

    fn crop_frame(&self) -> CropFrame {
        CropFrame {
            centre: [0, 1].map(|axis| self.centre[axis] / self.picture[axis]),
            size: [0, 1].map(|axis| 2.0 * self.half_size[axis] / self.picture[axis]),
        }
    }

    /// Where the point `across` and `down` pixels from the centre, along the
    /// sides of the frame, is in the picture.
    fn at(&self, [across, down]: [f32; 2]) -> PicturePoint {
        [0, 1].map(|axis| self.centre[axis] + across * self.across[axis] + down * self.down[axis])
    }

    /// `point` from the centre, in pixels along the sides of the frame.
    fn along_sides(&self, point: PicturePoint) -> [f32; 2] {
        let from_centre = [0, 1].map(|axis| point[axis] - self.centre[axis]);
        [self.across, self.down].map(|side| from_centre[0] * side[0] + from_centre[1] * side[1])
    }

    /// How far the frame reaches from its centre along each axis of the picture.
    fn reach(&self) -> [f32; 2] {
        [0, 1].map(|axis| {
            (self.half_size[0] * self.across[axis]).abs()
                + (self.half_size[1] * self.down[axis]).abs()
        })
    }

    fn shortest_side(&self) -> f32 {
        SHORTEST_SIDE * self.picture[0].max(self.picture[1])
    }

    /// The frame with the side `towards` (−1 or 1) of `axis` at `position`
    /// along that axis, the opposite side staying.
    fn with_side_at(self, axis: usize, towards: f32, position: f32) -> Self {
        let opposite = -towards * self.half_size[axis];
        let length = (towards * (position - opposite)).max(self.shortest_side());
        let mut middle = [0.0; 2];
        middle[axis] = opposite + towards * length / 2.0;
        let mut half_size = self.half_size;
        half_size[axis] = length / 2.0;
        Self {
            centre: self.at(middle),
            half_size,
            ..self
        }
    }

    /// The frame resized by `handle` dragged to `position` (along its sides),
    /// keeping its proportions: around the opposite corner for a corner
    /// handle, around the middle of the opposite side for a side handle.
    fn with_handle_at_keeping_ratio(self, handle: FrameHandle, position: [f32; 2]) -> Self {
        let anchor = [0, 1].map(|axis| -handle.0[axis] * self.half_size[axis]);
        let wanted = [0, 1].map(|axis| {
            let towards = handle.0[axis];
            towards * (position[axis] - anchor[axis]) / (2.0 * self.half_size[axis])
        });
        let held = [0, 1].map(|axis| handle.0[axis] != 0.0);
        let smallest = self.shortest_side() / (2.0 * self.half_size[0].min(self.half_size[1]));
        let share = match held {
            [true, true] => wanted[0].max(wanted[1]),
            [true, false] => wanted[0],
            _ => wanted[1],
        }
        .max(smallest);
        let half_size = self.half_size.map(|half| half * share);
        Self {
            centre: self.at([0, 1].map(|axis| anchor[axis] + handle.0[axis] * half_size[axis])),
            half_size,
            ..self
        }
    }

    fn corners(&self) -> [PicturePoint; 4] {
        let [across, down] = self.half_size;
        [
            [-across, -down],
            [across, -down],
            [across, down],
            [-across, down],
        ]
        .map(|corner| self.at(corner))
    }

    /// The frame as far from this one towards `wanted` as it gets without a
    /// corner leaving the picture.
    fn towards(self, wanted: Self) -> Self {
        let mut share: f32 = 1.0;
        for (from, to) in self.corners().into_iter().zip(wanted.corners()) {
            for axis in [0, 1] {
                let moved = to[axis] - from[axis];
                let room = match moved > 0.0 {
                    true => self.picture[axis] - from[axis],
                    false => -from[axis],
                };
                if moved != 0.0 {
                    share = share.min((room / moved).max(0.0));
                }
            }
        }
        let between = |from: f32, to: f32| from + (to - from) * share;
        Self {
            centre: [0, 1].map(|axis| between(self.centre[axis], wanted.centre[axis])),
            half_size: [0, 1].map(|axis| between(self.half_size[axis], wanted.half_size[axis])),
            ..self
        }
    }

    /// The same frame around the same centre, shrunk as much as needed to hold in the picture.
    fn held_in_picture(self) -> Self {
        let reach = self.reach();
        let room = [0, 1].map(|axis| self.centre[axis].min(self.picture[axis] - self.centre[axis]));
        let share = (room[0] / reach[0]).min(room[1] / reach[1]).clamp(0.0, 1.0);
        Self {
            half_size: self.half_size.map(|half| half * share),
            ..self
        }
    }
}

impl Framing {
    fn with_frame(self, frame: &FrameInPicture) -> Self {
        Self {
            frame: frame.crop_frame(),
            ..self
        }
    }

    /// Whether the frame leaves part of the picture out or the picture is straightened.
    pub fn is_cropped(&self) -> bool {
        *self != self.without_crop()
    }

    /// The whole picture, not straightened, turned as this framing turns it.
    pub fn without_crop(&self) -> Self {
        Self {
            turn: self.turn,
            ..Self::default()
        }
    }

    /// The directions, in the picture, a step to the right and a step down
    /// on the framed photo lead to.
    pub fn steps_in_picture(&self) -> [[f32; 2]; 2] {
        let (sine, cosine) = self.angle.to_radians().sin_cos();
        let (across, down) = ([cosine, -sine], [sine, cosine]);
        self.turn
            .steps()
            .map(|step| [0, 1].map(|axis| step[0] * across[axis] + step[1] * down[axis]))
    }

    /// Half the width and the height of the framed photo, in pixels of the picture.
    fn framed_half_size(&self, frame: &FrameInPicture) -> [f32; 2] {
        match self.turn.swaps_width_and_height() {
            true => [frame.half_size[1], frame.half_size[0]],
            false => frame.half_size,
        }
    }

    /// Where a point of `picture` is on the framed photo, in shares of its
    /// width and height; outside 0 to 1, the point is out of the frame.
    pub fn share_of_framed_photo(&self, picture: [u32; 2], point: PicturePoint) -> [f32; 2] {
        let frame = FrameInPicture::of(self, picture);
        let half_size = self.framed_half_size(&frame);
        let from_centre = [0, 1].map(|axis| point[axis] - frame.centre[axis]);
        let steps = self.steps_in_picture();
        [0, 1].map(|side| {
            let along = from_centre[0] * steps[side][0] + from_centre[1] * steps[side][1];
            0.5 + along / (2.0 * half_size[side])
        })
    }

    /// The part of `picture` a render of the framed photo shows.
    pub fn region(&self, picture: [u32; 2]) -> PictureRegion {
        let frame = FrameInPicture::of(self, picture);
        let half_size = self.framed_half_size(&frame);
        let [across, down] = self.steps_in_picture();
        let origin = [0, 1].map(|axis| {
            frame.centre[axis] - half_size[0] * across[axis] - half_size[1] * down[axis]
        });
        let in_shares = |vector: [f32; 2], length: f32| {
            [0, 1].map(|axis| vector[axis] * length / frame.picture[axis])
        };
        PictureRegion {
            origin: in_shares(origin, 1.0),
            across: in_shares(across, 2.0 * half_size[0]),
            down: in_shares(down, 2.0 * half_size[1]),
        }
    }

    /// The size of the framed photo, in pixels of `picture`.
    pub fn framed_size(&self, picture: [u32; 2]) -> [u32; 2] {
        let [width, height] = [0, 1].map(|axis| {
            let side = self.frame.size[axis] * picture[axis] as f32;
            (side.round() as u32).max(1)
        });
        match self.turn.swaps_width_and_height() {
            true => [height, width],
            false => [width, height],
        }
    }

    /// Where `handle` is in `picture`.
    pub fn handle_position(&self, picture: [u32; 2], handle: FrameHandle) -> PicturePoint {
        let frame = FrameInPicture::of(self, picture);
        frame.at([0, 1].map(|axis| handle.0[axis] * frame.half_size[axis]))
    }

    /// Where the centre of the frame is in `picture`.
    pub fn frame_centre(&self, picture: [u32; 2]) -> PicturePoint {
        self.handle_position(picture, FrameHandle([0.0, 0.0]))
    }

    pub fn holds(&self, picture: [u32; 2], point: PicturePoint) -> bool {
        let frame = FrameInPicture::of(self, picture);
        let along_sides = frame.along_sides(point);
        [0, 1]
            .iter()
            .all(|axis| along_sides[*axis].abs() <= frame.half_size[*axis])
    }

    /// The framing once `handle` is dragged to `point`: the sides it holds
    /// follow, stopped at the edge of the picture and before the frame gets
    /// too small; the opposite ones stay.
    pub fn with_handle_at(
        self,
        picture: [u32; 2],
        handle: FrameHandle,
        point: PicturePoint,
    ) -> Self {
        let mut frame = FrameInPicture::of(&self, picture);
        for axis in [0, 1] {
            let towards = handle.0[axis];
            if towards != 0.0 {
                let position = frame.along_sides(point)[axis];
                frame = frame.towards(frame.with_side_at(axis, towards, position));
            }
        }
        self.with_frame(&frame)
    }

    /// The framing once `handle` is dragged to `point`, the frame keeping its
    /// proportions; stopped where a corner meets the edge of the picture.
    pub fn with_handle_at_keeping_ratio(
        self,
        picture: [u32; 2],
        handle: FrameHandle,
        point: PicturePoint,
    ) -> Self {
        let frame = FrameInPicture::of(&self, picture);
        let resized = frame.with_handle_at_keeping_ratio(handle, frame.along_sides(point));
        self.with_frame(&frame.towards(resized))
    }

    /// The long side of the frame over its short one, in pixels of `picture`.
    pub fn long_over_short(&self, picture: [u32; 2]) -> f32 {
        let [width, height] = FrameInPicture::of(self, picture).half_size;
        width.max(height) / width.min(height)
    }

    /// The largest frame of the proportions `long_over_short` held in this
    /// one, around the same centre, its long side along this one's.
    pub fn with_ratio(self, picture: [u32; 2], long_over_short: f32) -> Self {
        let frame = FrameInPicture::of(&self, picture);
        let (long, short) = match frame.half_size[0] >= frame.half_size[1] {
            true => (0, 1),
            false => (1, 0),
        };
        let mut half_size = frame.half_size;
        half_size[long] = frame.half_size[long].min(frame.half_size[short] * long_over_short);
        half_size[short] = half_size[long] / long_over_short;
        self.with_frame(&FrameInPicture { half_size, ..frame })
    }

    /// The frame turned from landscape to portrait or back around its
    /// centre, shrunk as much as needed to hold in the picture.
    pub fn with_sides_swapped(self, picture: [u32; 2]) -> Self {
        let frame = FrameInPicture::of(&self, picture);
        let [width, height] = frame.half_size;
        let swapped = FrameInPicture {
            half_size: [height, width],
            ..frame
        };
        self.with_frame(&swapped.held_in_picture())
    }

    /// The same frame `shift` pixels further over `picture`, stopped at its edge.
    pub fn moved_by(self, picture: [u32; 2], shift: [f32; 2]) -> Self {
        let frame = FrameInPicture::of(&self, picture);
        let reach = frame.reach();
        let centre = [0, 1].map(|axis| {
            let moved = frame.centre[axis] + shift[axis];
            moved
                .max(reach[axis])
                .min(frame.picture[axis] - reach[axis])
        });
        self.with_frame(&FrameInPicture { centre, ..frame })
    }

    /// The picture turned so that the line drawn on it between `from` and
    /// `to` is level or upright, whichever needs the smaller turn. A line too
    /// short to tell a direction changes nothing.
    pub fn levelled(self, picture: [u32; 2], [from, to]: [PicturePoint; 2]) -> Self {
        let along = [0, 1].map(|axis| to[axis] - from[axis]);
        let [width, height] = in_pixels(picture);
        if along[0].hypot(along[1]) < SHORTEST_LEVEL_LINE * width.max(height) {
            return self;
        }
        let tilt = along[1].atan2(along[0]).to_degrees();
        let to_the_nearest_axis = -tilt - 90.0 * (-tilt / 90.0).round();
        self.straightened(picture, to_the_nearest_axis)
    }

    /// The picture turned to `angle` under this frame, which keeps its centre
    /// and its proportions and shrinks as much as needed to hold in the picture.
    pub fn straightened(self, picture: [u32; 2], angle: f32) -> Self {
        let turned = Self {
            angle: angle.clamp(*ANGLE_RANGE.start(), *ANGLE_RANGE.end()),
            ..self
        };
        turned.with_frame(&FrameInPicture::of(&turned, picture).held_in_picture())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PICTURE: [u32; 2] = [1000, 500];
    const TOP_LEFT: FrameHandle = FrameHandle([-1.0, -1.0]);
    const RIGHT: FrameHandle = FrameHandle([1.0, 0.0]);

    fn framing(centre: [f32; 2], size: [f32; 2]) -> Framing {
        Framing {
            frame: CropFrame { centre, size },
            ..Framing::default()
        }
    }

    fn assert_close(actual: [f32; 2], expected: [f32; 2]) {
        let is_close = [0, 1]
            .iter()
            .all(|axis| (actual[*axis] - expected[*axis]).abs() < 1e-4);
        assert!(is_close, "{actual:?} differs from {expected:?}");
    }

    #[test]
    fn the_default_framing_keeps_the_whole_picture() {
        let framing = Framing::default();

        assert_eq!(framing.region(PICTURE), PictureRegion::WHOLE);
        assert_eq!(framing.framed_size(PICTURE), PICTURE);
    }

    #[test]
    fn a_frame_gives_its_part_of_the_picture_and_its_size_in_pixels() {
        let right_half = framing([0.75, 0.5], [0.5, 1.0]);

        assert_eq!(
            right_half.region(PICTURE),
            PictureRegion::upright([0.5, 0.0], [0.5, 1.0])
        );
        assert_eq!(right_half.framed_size(PICTURE), [500, 500]);
    }

    #[test]
    fn a_corner_handle_moves_its_two_sides_and_leaves_the_opposite_corner() {
        let cropped = Framing::default().with_handle_at(PICTURE, TOP_LEFT, [200.0, 100.0]);

        assert_close(cropped.frame.centre, [0.6, 0.6]);
        assert_close(cropped.frame.size, [0.8, 0.8]);
    }

    #[test]
    fn a_side_handle_moves_its_side_only() {
        let cropped = Framing::default().with_handle_at(PICTURE, RIGHT, [600.0, 20.0]);

        assert_close(cropped.frame.centre, [0.3, 0.5]);
        assert_close(cropped.frame.size, [0.6, 1.0]);
    }

    #[test]
    fn a_handle_dragged_out_of_the_picture_stops_at_its_edge() {
        let inside = framing([0.5, 0.5], [0.5, 0.5]);

        let widened = inside.with_handle_at(PICTURE, RIGHT, [4000.0, 0.0]);

        assert_close(widened.handle_position(PICTURE, RIGHT), [1000.0, 250.0]);
    }

    #[test]
    fn a_handle_dragged_past_the_opposite_side_leaves_the_shortest_frame() {
        let crossed = Framing::default().with_handle_at(PICTURE, RIGHT, [-300.0, 0.0]);

        assert_close(crossed.frame.size, [SHORTEST_SIDE, 1.0]);
        assert_close(crossed.handle_position(PICTURE, RIGHT), [20.0, 250.0]);
    }

    #[test]
    fn a_moved_frame_keeps_its_size_and_stops_at_the_edge_of_the_picture() {
        let inside = framing([0.5, 0.5], [0.5, 0.5]);

        let moved = inside.moved_by(PICTURE, [100.0, -4000.0]);

        assert_close(moved.frame.centre, [0.6, 0.25]);
        assert_eq!(moved.frame.size, inside.frame.size);
    }

    #[test]
    fn a_frame_holds_the_points_inside_it() {
        let right_half = framing([0.75, 0.5], [0.5, 1.0]);

        assert!(right_half.holds(PICTURE, [600.0, 250.0]));
        assert!(!right_half.holds(PICTURE, [400.0, 250.0]));
    }

    fn assert_holds_in_picture(framing: &Framing) {
        for handle in FRAME_HANDLES {
            let [across, down] = framing.handle_position(PICTURE, handle);
            let is_inside = (-0.01..=1000.01).contains(&across) && (-0.01..=500.01).contains(&down);
            assert!(is_inside, "{framing:?} leaves the picture at {handle:?}");
        }
    }

    #[test]
    fn a_straightened_frame_keeps_its_centre_and_proportions_and_holds_in_the_picture() {
        let straightened = Framing::default().straightened(PICTURE, 10.0);

        assert_eq!(straightened.angle, 10.0);
        assert_close(straightened.frame.centre, [0.5, 0.5]);
        assert!((straightened.frame.size[0] - straightened.frame.size[1]).abs() < 1e-4);
        assert!(straightened.frame.size[0] < 1.0);
        assert_holds_in_picture(&straightened);
        let top_right = straightened.handle_position(PICTURE, FrameHandle([1.0, -1.0]));
        assert!(
            top_right[1].abs() < 0.01,
            "{top_right:?} is not on the top edge"
        );
    }

    #[test]
    fn no_angle_makes_a_frame_leave_the_picture() {
        let frames = [
            framing([0.5, 0.5], [1.0, 1.0]),
            framing([0.2, 0.7], [0.4, 0.6]),
            framing([0.9, 0.1], [0.2, 0.2]),
        ];
        for frame in frames {
            for tenth in -450..=450 {
                assert_holds_in_picture(&frame.straightened(PICTURE, tenth as f32 / 10.0));
            }
        }
    }

    #[test]
    fn an_angle_is_kept_within_its_range() {
        assert_eq!(Framing::default().straightened(PICTURE, 80.0).angle, 45.0);
        assert_eq!(Framing::default().straightened(PICTURE, -80.0).angle, -45.0);
    }

    #[test]
    fn a_frame_small_enough_is_left_alone_by_an_angle() {
        let small = framing([0.5, 0.5], [0.2, 0.2]);

        assert_eq!(small.straightened(PICTURE, 20.0).frame, small.frame);
    }

    #[test]
    fn coming_back_to_the_starting_angle_gives_the_starting_frame_back() {
        let start = framing([0.4, 0.5], [0.8, 0.9]);

        let back = start.straightened(PICTURE, 0.0);

        assert_eq!(
            start.straightened(PICTURE, 30.0).frame.centre,
            start.frame.centre
        );
        assert_close(back.frame.size, start.frame.size);
    }

    #[test]
    fn a_straightened_frame_gives_a_tilted_part_of_the_picture() {
        let tilted = framing([0.5, 0.5], [0.2, 0.2]).straightened(PICTURE, 90.0_f32.min(45.0));

        let region = tilted.region(PICTURE);

        let top_right = region.at([1.0, 0.0]);
        assert!(
            top_right[1] < region.origin[1],
            "the top side does not rise"
        );
        assert_close(region.at([0.5, 0.5]), [0.5, 0.5]);
    }

    #[test]
    fn handles_of_a_straightened_frame_resize_it_along_its_own_sides() {
        let tilted = framing([0.5, 0.5], [0.2, 0.4]).straightened(PICTURE, 30.0);
        let right = tilted.handle_position(PICTURE, RIGHT);
        let further =
            [0, 1].map(|axis| right[axis] + (right[axis] - 250.0 * [2.0, 1.0][axis]) * 0.5);

        let widened = tilted.with_handle_at(PICTURE, RIGHT, further);

        assert_close(widened.handle_position(PICTURE, RIGHT), further);
        assert_close(
            widened.handle_position(PICTURE, FrameHandle([-1.0, 0.0])),
            tilted.handle_position(PICTURE, FrameHandle([-1.0, 0.0])),
        );
        assert!((widened.frame.size[1] - tilted.frame.size[1]).abs() < 1e-5);
    }

    #[test]
    fn a_handle_of_a_straightened_frame_stops_where_a_corner_meets_the_edge() {
        let tilted = framing([0.5, 0.5], [0.2, 0.4]).straightened(PICTURE, 30.0);

        let widened = tilted.with_handle_at(PICTURE, RIGHT, [5000.0, -2000.0]);

        assert!(widened.frame.size[0] > tilted.frame.size[0]);
        assert_holds_in_picture(&widened);
    }

    #[test]
    fn a_corner_handle_keeping_the_ratio_resizes_around_the_opposite_corner() {
        let inside = framing([0.5, 0.5], [0.4, 0.4]);

        let grown = inside.with_handle_at_keeping_ratio(PICTURE, TOP_LEFT, [200.0, 140.0]);

        assert_close(grown.frame.size, [0.5, 0.5]);
        assert_close(
            grown.handle_position(PICTURE, FrameHandle([1.0, 1.0])),
            inside.handle_position(PICTURE, FrameHandle([1.0, 1.0])),
        );
    }

    #[test]
    fn a_side_handle_keeping_the_ratio_resizes_around_the_middle_of_the_opposite_side() {
        let inside = framing([0.5, 0.5], [0.4, 0.4]);

        let shrunk = inside.with_handle_at_keeping_ratio(PICTURE, RIGHT, [500.0, 10.0]);

        assert_close(shrunk.frame.size, [0.2, 0.2]);
        assert_close(shrunk.frame.centre, [0.4, 0.5]);
    }

    #[test]
    fn a_handle_keeping_the_ratio_stops_where_the_frame_meets_the_edge() {
        let inside = framing([0.5, 0.5], [0.4, 0.4]);

        let grown = inside.with_handle_at_keeping_ratio(PICTURE, RIGHT, [5000.0, 250.0]);

        assert_close(grown.frame.size, [0.7, 0.7]);
        assert_close(grown.handle_position(PICTURE, RIGHT), [1000.0, 250.0]);
    }

    #[test]
    fn a_ratio_gives_the_largest_frame_of_these_proportions_inside_the_frame() {
        let landscape = Framing::default().with_ratio(PICTURE, 1.0);
        let portrait = framing([0.3, 0.5], [0.2, 1.0]).with_ratio(PICTURE, 1.5);

        assert_close(landscape.frame.size, [0.5, 1.0]);
        assert_close(landscape.frame.centre, [0.5, 0.5]);
        assert_close(portrait.frame.size, [0.2, 0.6]);
        assert_close(portrait.frame.centre, [0.3, 0.5]);
        assert!((portrait.long_over_short(PICTURE) - 1.5).abs() < 1e-4);
    }

    #[test]
    fn swapped_sides_turn_a_landscape_frame_into_a_portrait_one_held_in_the_picture() {
        let small = framing([0.5, 0.5], [0.4, 0.4]).with_sides_swapped(PICTURE);
        let whole = Framing::default().with_sides_swapped(PICTURE);

        assert_close(small.frame.size, [0.2, 0.8]);
        assert_close(whole.frame.size, [0.25, 1.0]);
        assert_close(whole.frame.centre, [0.5, 0.5]);
    }

    fn line_tilted_by(degrees: f32) -> [PicturePoint; 2] {
        let (sine, cosine) = degrees.to_radians().sin_cos();
        [
            [500.0, 250.0],
            [500.0 + 200.0 * cosine, 250.0 + 200.0 * sine],
        ]
    }

    #[test]
    fn a_level_line_turns_the_picture_until_it_is_horizontal_or_vertical() {
        let small = framing([0.5, 0.5], [0.2, 0.2]);
        let levelled = |tilt: f32| small.levelled(PICTURE, line_tilted_by(tilt)).angle;

        assert!((levelled(12.0) + 12.0).abs() < 1e-3);
        assert!((levelled(-30.0) - 30.0).abs() < 1e-3);
        assert!((levelled(80.0) - 10.0).abs() < 1e-3);
        assert!((levelled(192.0) + 12.0).abs() < 1e-3);
    }

    #[test]
    fn a_levelled_picture_holds_its_frame() {
        let levelled = Framing::default().levelled(PICTURE, line_tilted_by(12.0));

        assert!(levelled.frame.size[0] < 1.0);
        assert_holds_in_picture(&levelled);
    }

    #[test]
    fn a_level_line_too_short_to_tell_a_direction_changes_nothing() {
        let short = [[500.0, 250.0], [505.0, 252.0]];

        assert_eq!(
            Framing::default().levelled(PICTURE, short),
            Framing::default()
        );
    }

    #[test]
    fn four_quarter_turns_or_the_same_mirror_twice_give_the_turn_back() {
        let turn = Turn::default().turned_right().flipped_horizontally();

        assert_eq!(
            turn.turned_right()
                .turned_right()
                .turned_right()
                .turned_right(),
            turn
        );
        assert_eq!(turn.turned_left().turned_right(), turn);
        assert_eq!(turn.flipped_horizontally().flipped_horizontally(), turn);
        assert_eq!(turn.flipped_vertically().flipped_vertically(), turn);
    }

    #[test]
    fn both_mirrors_are_a_half_turn() {
        let flipped = Turn::default().flipped_horizontally().flipped_vertically();

        assert_eq!(flipped, Turn::default().turned_right().turned_right());
    }

    fn turned(turn: Turn) -> Framing {
        Framing {
            turn,
            ..framing([0.75, 0.5], [0.5, 1.0])
        }
    }

    #[test]
    fn a_quarter_turn_swaps_the_sides_of_the_framed_photo_and_keeps_its_part_of_the_picture() {
        let right = turned(Turn::default().turned_right());

        assert_eq!(right.framed_size(PICTURE), [500, 500]);
        assert_eq!(turned(Turn::default()).framed_size([1000, 400]), [500, 400]);
        assert_eq!(right.framed_size([1000, 400]), [400, 500]);
        let region = right.region(PICTURE);
        assert_close(region.at([0.5, 0.5]), [0.75, 0.5]);
        assert_close(region.at([0.0, 0.0]), [0.5, 1.0]);
        assert_close(region.at([1.0, 0.0]), [0.5, 0.0]);
    }

    #[test]
    fn a_mirror_shows_the_same_part_of_the_picture_the_other_way_round() {
        let flipped = turned(Turn::default().flipped_horizontally()).region(PICTURE);
        let upside_down = turned(Turn::default().flipped_vertically()).region(PICTURE);

        assert_close(flipped.at([0.0, 0.0]), [1.0, 0.0]);
        assert_close(flipped.at([1.0, 1.0]), [0.5, 1.0]);
        assert_close(upside_down.at([0.0, 0.0]), [0.5, 1.0]);
        assert_close(upside_down.at([1.0, 1.0]), [1.0, 0.0]);
    }

    #[test]
    fn a_flip_seen_after_a_quarter_turn_is_still_left_to_right_on_screen() {
        let turn = Turn::default().turned_right().flipped_horizontally();

        let region = turned(turn).region(PICTURE);

        assert_close(region.at([0.0, 0.0]), [0.5, 0.0]);
        assert_close(region.at([1.0, 0.0]), [0.5, 1.0]);
    }

    #[test]
    fn a_point_of_the_picture_is_found_on_the_framed_photo_where_its_region_shows_it() {
        let framing = Framing {
            turn: Turn::default().turned_right().flipped_horizontally(),
            ..framing([0.5, 0.5], [0.5, 0.5]).straightened(PICTURE, 12.0)
        };
        let region = framing.region(PICTURE);

        for share in [[0.0, 0.0], [1.0, 0.0], [0.3, 0.8]] {
            let [across, down] = region.at(share);
            let point = [across * PICTURE[0] as f32, down * PICTURE[1] as f32];
            assert_close(framing.share_of_framed_photo(PICTURE, point), share);
        }
    }

    #[test]
    fn a_framing_without_its_crop_keeps_only_its_turn() {
        let turn = Turn::default().turned_left();
        let cropped = Framing {
            turn,
            ..framing([0.5, 0.5], [0.5, 0.5]).straightened(PICTURE, 4.0)
        };

        assert!(cropped.is_cropped());
        assert!(!cropped.without_crop().is_cropped());
        assert_eq!(cropped.without_crop().turn, turn);
        assert_eq!(
            cropped.without_crop().region(PICTURE),
            turned(turn).without_crop().region(PICTURE)
        );
    }
}
