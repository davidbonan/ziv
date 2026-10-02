use serde::{Deserialize, Serialize};

use super::mask::PhotoPoint;

/// Shorter than any gradient a hand can draw; keeps a zero-length one finite.
pub const SHORTEST_GRADIENT_SQUARED: f32 = 1e-12;

/// Full coverage up to the line through `full`, none from the line through
/// `none`, progressive in between. Both lines are perpendicular to the segment.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LinearGradient {
    pub full: PhotoPoint,
    pub none: PhotoPoint,
}

/// Handles, in the order a canvas shows them.
pub const FULL_HANDLE: usize = 0;
pub const NONE_HANDLE: usize = 1;
pub const MIDDLE_HANDLE: usize = 2;

impl LinearGradient {
    /// A gradient of no length yet, to be drawn by dragging its `none` end.
    pub fn starting_at(point: PhotoPoint) -> Self {
        Self {
            full: point,
            none: point,
        }
    }

    pub fn handles(&self) -> [PhotoPoint; 3] {
        let middle = [0, 1].map(|axis| (self.full[axis] + self.none[axis]) / 2.0);
        [self.full, self.none, middle]
    }

    /// The gradient after `handle` was dragged to `point`: an end moves alone,
    /// the middle carries both ends.
    pub fn with_handle_at(self, handle: usize, point: PhotoPoint) -> Self {
        let middle = self.handles()[MIDDLE_HANDLE];
        let carried = |end: PhotoPoint| [0, 1].map(|axis| end[axis] + point[axis] - middle[axis]);
        match handle {
            FULL_HANDLE => Self {
                full: point,
                ..self
            },
            NONE_HANDLE => Self {
                none: point,
                ..self
            },
            _ => Self {
                full: carried(self.full),
                none: carried(self.none),
            },
        }
    }

    pub fn coverage(&self, point: PhotoPoint) -> f32 {
        let along = [self.none[0] - self.full[0], self.none[1] - self.full[1]];
        let to_point = [point[0] - self.full[0], point[1] - self.full[1]];
        let length_squared =
            (along[0] * along[0] + along[1] * along[1]).max(SHORTEST_GRADIENT_SQUARED);
        let progress = (to_point[0] * along[0] + to_point[1] * along[1]) / length_squared;
        let progress = progress.clamp(0.0, 1.0);
        1.0 - progress * progress * (3.0 - 2.0 * progress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOP_TO_BOTTOM: LinearGradient = LinearGradient {
        full: [0.5, 0.2],
        none: [0.5, 0.6],
    };

    #[test]
    fn coverage_is_full_before_the_gradient_and_none_after() {
        assert_eq!(TOP_TO_BOTTOM.coverage([0.1, 0.0]), 1.0);
        assert_eq!(TOP_TO_BOTTOM.coverage([0.9, 0.2]), 1.0);
        assert_eq!(TOP_TO_BOTTOM.coverage([0.3, 0.6]), 0.0);
        assert_eq!(TOP_TO_BOTTOM.coverage([0.3, 1.0]), 0.0);
    }

    #[test]
    fn coverage_is_half_in_the_middle_and_falls_along_the_gradient() {
        let sampled: Vec<f32> = [0.3, 0.4, 0.5]
            .iter()
            .map(|y| TOP_TO_BOTTOM.coverage([0.0, *y]))
            .collect();

        assert!((sampled[1] - 0.5).abs() < 1e-6);
        assert!(sampled[0] > sampled[1] && sampled[1] > sampled[2]);
    }

    #[test]
    fn coverage_follows_a_slanted_gradient() {
        let diagonal = LinearGradient {
            full: [0.0, 0.0],
            none: [1.0, 1.0],
        };

        assert!((diagonal.coverage([1.0, 0.0]) - 0.5).abs() < 1e-6);
        assert!((diagonal.coverage([0.0, 1.0]) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn dragging_an_end_moves_that_end_alone() {
        let moved = TOP_TO_BOTTOM.with_handle_at(NONE_HANDLE, [0.7, 0.9]);

        assert_eq!(moved.full, TOP_TO_BOTTOM.full);
        assert_eq!(moved.none, [0.7, 0.9]);
    }

    #[test]
    fn dragging_the_middle_carries_both_ends() {
        let moved = TOP_TO_BOTTOM.with_handle_at(MIDDLE_HANDLE, [0.6, 0.5]);

        assert_eq!(moved.handles()[MIDDLE_HANDLE], [0.6, 0.5]);
        assert!((moved.full[1] - 0.3).abs() < 1e-6 && (moved.none[1] - 0.7).abs() < 1e-6);
        assert!((moved.full[0] - 0.6).abs() < 1e-6);
    }
}
