use serde::{Deserialize, Serialize};

use super::mask::PhotoPoint;

pub const FEATHER_RANGE: std::ops::RangeInclusive<f32> = 0.0..=100.0;
pub const DEFAULT_FEATHER: f32 = 50.0;
/// Keeps a radius or a feather of zero finite.
pub const SMALLEST_EXTENT: f32 = 1e-5;
/// How far outside the ellipse the rotation handle sits, in photo units.
const ROTATION_HANDLE_GAP: f32 = 0.04;

pub const CENTRE_HANDLE: usize = 0;
const RIGHT_HANDLE: usize = 1;
const BOTTOM_HANDLE: usize = 2;
const LEFT_HANDLE: usize = 3;
const TOP_HANDLE: usize = 4;
pub const ROTATION_HANDLE: usize = 5;
/// Not shown: the corner of the box a drag draws the ellipse in.
pub const DRAWING_HANDLE: usize = 6;

/// An ellipse covered inside. `feather` is the share of the radius, from the
/// edge inward, over which coverage rises from none to full.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RadialGradient {
    pub centre: PhotoPoint,
    pub radii: [f32; 2],
    /// In radians, clockwise on screen.
    pub rotation: f32,
    pub feather: f32,
}

impl RadialGradient {
    /// An ellipse of no size yet, to be drawn by dragging `DRAWING_HANDLE`.
    pub fn starting_at(centre: PhotoPoint) -> Self {
        Self {
            centre,
            radii: [0.0; 2],
            rotation: 0.0,
            feather: DEFAULT_FEATHER,
        }
    }

    /// The directions of the two radii.
    fn axes(&self) -> [[f32; 2]; 2] {
        let (sine, cosine) = self.rotation.sin_cos();
        [[cosine, sine], [-sine, cosine]]
    }

    /// `point` seen from the centre, along the two radii.
    fn along_axes(&self, point: PhotoPoint) -> [f32; 2] {
        let from_centre = [point[0] - self.centre[0], point[1] - self.centre[1]];
        self.axes()
            .map(|axis| from_centre[0] * axis[0] + from_centre[1] * axis[1])
    }

    fn at(&self, along_axes: [f32; 2]) -> PhotoPoint {
        let [first, second] = self.axes();
        [0, 1].map(|xy| self.centre[xy] + along_axes[0] * first[xy] + along_axes[1] * second[xy])
    }

    pub fn coverage(&self, point: PhotoPoint) -> f32 {
        let along = self.along_axes(point);
        let reach = [0, 1].map(|axis| along[axis] / self.radii[axis].max(SMALLEST_EXTENT));
        let distance = (reach[0] * reach[0] + reach[1] * reach[1]).sqrt();
        let feather = (self.feather / 100.0).max(SMALLEST_EXTENT);
        let progress = ((distance - (1.0 - feather)) / feather).clamp(0.0, 1.0);
        1.0 - progress * progress * (3.0 - 2.0 * progress)
    }

    /// The points of the outline, for drawing it.
    pub fn outline(&self, points: usize) -> Vec<PhotoPoint> {
        (0..points)
            .map(|index| {
                let (sine, cosine) =
                    (index as f32 / points as f32 * std::f32::consts::TAU).sin_cos();
                self.at([self.radii[0] * cosine, self.radii[1] * sine])
            })
            .collect()
    }

    pub fn handles(&self) -> [PhotoPoint; 6] {
        let [across, down] = self.radii;
        [
            self.centre,
            self.at([across, 0.0]),
            self.at([0.0, down]),
            self.at([-across, 0.0]),
            self.at([0.0, -down]),
            self.at([0.0, -down - ROTATION_HANDLE_GAP]),
        ]
    }

    pub fn with_handle_at(self, handle: usize, point: PhotoPoint) -> Self {
        let along = self.along_axes(point);
        let [across, down] = self.radii;
        match handle {
            CENTRE_HANDLE => Self {
                centre: point,
                ..self
            },
            RIGHT_HANDLE | LEFT_HANDLE => Self {
                radii: [along[0].abs(), down],
                ..self
            },
            BOTTOM_HANDLE | TOP_HANDLE => Self {
                radii: [across, along[1].abs()],
                ..self
            },
            ROTATION_HANDLE => {
                let from_centre = [point[0] - self.centre[0], point[1] - self.centre[1]];
                let upward = from_centre[1].atan2(from_centre[0]);
                Self {
                    rotation: upward + std::f32::consts::FRAC_PI_2,
                    ..self
                }
            }
            _ => Self {
                radii: along.map(f32::abs),
                ..self
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIDE: RadialGradient = RadialGradient {
        centre: [0.5, 0.3],
        radii: [0.4, 0.2],
        rotation: 0.0,
        feather: 50.0,
    };

    fn assert_near(actual: PhotoPoint, expected: PhotoPoint) {
        let is_near =
            (actual[0] - expected[0]).abs() < 1e-5 && (actual[1] - expected[1]).abs() < 1e-5;
        assert!(is_near, "{actual:?} is not {expected:?}");
    }

    #[test]
    fn coverage_is_full_at_the_heart_and_none_from_the_edge() {
        assert_eq!(WIDE.coverage([0.5, 0.3]), 1.0);
        assert_eq!(WIDE.coverage([0.69, 0.3]), 1.0);
        assert_eq!(WIDE.coverage([0.9, 0.3]), 0.0);
        assert_eq!(WIDE.coverage([0.5, 0.55]), 0.0);
    }

    #[test]
    fn coverage_is_half_in_the_middle_of_the_feather() {
        assert!((WIDE.coverage([0.8, 0.3]) - 0.5).abs() < 1e-5);
        assert!((WIDE.coverage([0.5, 0.15]) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn without_feather_the_edge_is_sharp() {
        let sharp = RadialGradient {
            feather: 0.0,
            ..WIDE
        };

        assert_eq!(sharp.coverage([0.89, 0.3]), 1.0);
        assert_eq!(sharp.coverage([0.91, 0.3]), 0.0);
    }

    #[test]
    fn rotated_ellipse_covers_along_its_turned_radii() {
        let upright = RadialGradient {
            rotation: std::f32::consts::FRAC_PI_2,
            feather: 0.0,
            ..WIDE
        };

        assert_eq!(upright.coverage([0.5, 0.65]), 1.0);
        assert_eq!(upright.coverage([0.8, 0.3]), 0.0);
    }

    #[test]
    fn drawing_sets_both_radii_from_the_dragged_corner() {
        let drawn =
            RadialGradient::starting_at([0.5, 0.5]).with_handle_at(DRAWING_HANDLE, [0.8, 0.4]);

        assert_near(drawn.radii, [0.3, 0.1]);
        assert_eq!(drawn.centre, [0.5, 0.5]);
    }

    #[test]
    fn edge_handles_resize_one_radius_and_the_centre_moves_the_ellipse() {
        let wider = WIDE.with_handle_at(LEFT_HANDLE, [0.0, 0.3]);
        let moved = WIDE.with_handle_at(CENTRE_HANDLE, [0.2, 0.2]);

        assert_near(wider.radii, [0.5, 0.2]);
        assert_eq!((moved.centre, moved.radii), ([0.2, 0.2], WIDE.radii));
    }

    #[test]
    fn rotation_handle_turns_the_ellipse_toward_the_pointer() {
        let turned = WIDE.with_handle_at(ROTATION_HANDLE, [0.9, 0.3]);

        assert!((turned.rotation - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
        assert_near(turned.handles()[ROTATION_HANDLE], [0.5 + 0.2 + 0.04, 0.3]);
    }
}
