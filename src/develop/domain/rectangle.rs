use serde::{Deserialize, Serialize};

use super::feathered_edge::feathered_coverage;
use super::mask::PhotoPoint;

pub const CENTRE_HANDLE: usize = 0;
/// The corner a drag pulls to draw the rectangle.
pub const DRAWING_HANDLE: usize = 3;

// Which way each corner handle lies from the centre, along both half sizes.
const CORNERS: [[f32; 2]; 4] = [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];

/// A rectangle covered inside, feathered inward from its edge. A half size may
/// be negative while a corner is dragged past the opposite one.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rectangle {
    pub centre: PhotoPoint,
    pub half_size: [f32; 2],
    pub feather: f32,
}

impl Rectangle {
    /// A rectangle of no size yet, to be drawn by dragging `DRAWING_HANDLE`.
    pub fn starting_at(corner: PhotoPoint) -> Self {
        Self {
            centre: corner,
            half_size: [0.0; 2],
            feather: 0.0,
        }
    }

    fn corner(&self, toward: [f32; 2]) -> PhotoPoint {
        [0, 1].map(|axis| self.centre[axis] + toward[axis] * self.half_size[axis])
    }

    pub fn corners(&self) -> [PhotoPoint; 4] {
        CORNERS.map(|toward| self.corner(toward))
    }

    pub fn coverage(&self, point: PhotoPoint) -> f32 {
        let depths =
            [0, 1].map(|axis| self.half_size[axis].abs() - (point[axis] - self.centre[axis]).abs());
        feathered_coverage(depths[0].min(depths[1]), self.feather)
    }

    pub fn handles(&self) -> [PhotoPoint; 5] {
        let [first, second, third, fourth] = self.corners();
        [self.centre, first, second, third, fourth]
    }

    /// A corner moves while the opposite one stays; the centre carries all.
    pub fn with_handle_at(self, handle: usize, point: PhotoPoint) -> Self {
        if handle == CENTRE_HANDLE {
            return Self {
                centre: point,
                ..self
            };
        }
        let toward = CORNERS[(handle - 1) % CORNERS.len()];
        let opposite = self.corner(toward.map(|side| -side));
        Self {
            centre: [0, 1].map(|axis| (point[axis] + opposite[axis]) / 2.0),
            half_size: [0, 1].map(|axis| (point[axis] - opposite[axis]) / 2.0 * toward[axis]),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOX: Rectangle = Rectangle {
        centre: [0.5, 0.4],
        half_size: [0.3, 0.2],
        feather: 0.0,
    };

    fn assert_near(actual: PhotoPoint, expected: PhotoPoint) {
        let is_near =
            (actual[0] - expected[0]).abs() < 1e-6 && (actual[1] - expected[1]).abs() < 1e-6;
        assert!(is_near, "{actual:?} is not {expected:?}");
    }

    #[test]
    fn sharp_rectangle_covers_its_inside_only() {
        assert_eq!(BOX.coverage([0.5, 0.4]), 1.0);
        assert_eq!(BOX.coverage([0.79, 0.59]), 1.0);
        assert_eq!(BOX.coverage([0.81, 0.4]), 0.0);
        assert_eq!(BOX.coverage([0.5, 0.61]), 0.0);
    }

    #[test]
    fn feather_rises_inward_from_the_edge() {
        let soft = Rectangle {
            feather: 50.0,
            ..BOX
        };

        assert_eq!(soft.coverage([0.8, 0.4]), 0.0);
        assert!((soft.coverage([0.75, 0.4]) - 0.5).abs() < 1e-4);
        assert_eq!(soft.coverage([0.5, 0.4]), 1.0);
    }

    #[test]
    fn drawing_spans_from_the_start_to_the_dragged_corner_in_any_direction() {
        let start = Rectangle::starting_at([0.5, 0.5]);
        let down_right = start.with_handle_at(DRAWING_HANDLE, [0.9, 0.7]);
        let up_left = start.with_handle_at(DRAWING_HANDLE, [0.1, 0.3]);

        assert_eq!(down_right.coverage([0.7, 0.6]), 1.0);
        assert_eq!(up_left.coverage([0.3, 0.4]), 1.0);
        assert_eq!(up_left.coverage([0.7, 0.6]), 0.0);
        assert_near(up_left.handles()[DRAWING_HANDLE], [0.1, 0.3]);
    }

    #[test]
    fn dragging_a_corner_keeps_the_opposite_one() {
        let resized = BOX.with_handle_at(1, [0.1, 0.1]);

        assert_near(resized.handles()[1], [0.1, 0.1]);
        assert_near(resized.handles()[3], [0.8, 0.6]);
    }
}
