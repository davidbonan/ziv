use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use super::mask::PhotoPoint;
use super::radial_gradient::SMALLEST_EXTENT;

pub const SIZE_RANGE: RangeInclusive<f32> = 1.0..=100.0;
pub const FLOW_RANGE: RangeInclusive<f32> = 1.0..=100.0;
/// The radius of a brush of size 100, in photo units.
const LARGEST_RADIUS: f32 = 0.25;
/// What `[` and `]` change the size by.
pub const SIZE_STEP: f32 = 2.0;

/// The brush as the user set it; each stroke keeps the brush it was painted with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Brush {
    pub size: f32,
    pub feather: f32,
    pub flow: f32,
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            size: 20.0,
            feather: 50.0,
            flow: 100.0,
        }
    }
}

impl Brush {
    /// In photo units.
    pub fn radius(&self) -> f32 {
        self.size / SIZE_RANGE.end() * LARGEST_RADIUS
    }

    pub fn resized_by(self, change: f32) -> Self {
        Self {
            size: (self.size + change).clamp(*SIZE_RANGE.start(), *SIZE_RANGE.end()),
            ..self
        }
    }

    pub fn stroke_from(&self, point: PhotoPoint) -> Stroke {
        Stroke {
            points: vec![point],
            radius: self.radius(),
            feather: self.feather,
            flow: self.flow,
            is_erasing: false,
        }
    }
}

/// One press-drag-release of the brush.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    pub points: Vec<PhotoPoint>,
    /// In photo units.
    pub radius: f32,
    pub feather: f32,
    pub flow: f32,
    #[serde(default)]
    pub is_erasing: bool,
}

pub fn distance_squared_to_segment(point: PhotoPoint, from: PhotoPoint, to: PhotoPoint) -> f32 {
    let along = [to[0] - from[0], to[1] - from[1]];
    let to_point = [point[0] - from[0], point[1] - from[1]];
    let length_squared = along[0] * along[0] + along[1] * along[1];
    let progress = match length_squared > 0.0 {
        true => {
            ((to_point[0] * along[0] + to_point[1] * along[1]) / length_squared).clamp(0.0, 1.0)
        }
        false => 0.0,
    };
    let apart = [
        to_point[0] - along[0] * progress,
        to_point[1] - along[1] * progress,
    ];
    apart[0] * apart[0] + apart[1] * apart[1]
}

impl Stroke {
    /// The path as segments; a stroke of one point is a segment of no length.
    pub fn segments(&self) -> impl Iterator<Item = [PhotoPoint; 2]> + '_ {
        let first = self.points.first().map(|point| [*point, *point]);
        let later = self.points.windows(2).map(|pair| [pair[0], pair[1]]);
        first.into_iter().chain(later)
    }

    fn distance_to(&self, point: PhotoPoint) -> f32 {
        self.segments()
            .map(|[from, to]| distance_squared_to_segment(point, from, to))
            .fold(f32::MAX, f32::min)
            .sqrt()
    }

    /// How much the stroke paints at `distance` from its path: its flow within
    /// the heart of the brush, falling to nothing at its radius.
    pub fn opacity_at(&self, distance: f32) -> f32 {
        let feather = (self.feather / 100.0).max(SMALLEST_EXTENT);
        let reach = distance / self.radius.max(SMALLEST_EXTENT);
        let progress = ((reach - (1.0 - feather)) / feather).clamp(0.0, 1.0);
        self.flow / 100.0 * (1.0 - progress * progress * (3.0 - 2.0 * progress))
    }

    /// The coverage after this stroke went over a point covered by `coverage`.
    pub fn over(&self, coverage: f32, distance: f32) -> f32 {
        let opacity = self.opacity_at(distance);
        match self.is_erasing {
            true => coverage * (1.0 - opacity),
            false => coverage + (1.0 - coverage) * opacity,
        }
    }
}

/// What the brush painted: its strokes, in the order they were made.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BrushMask {
    pub strokes: Vec<Stroke>,
}

impl BrushMask {
    pub fn coverage(&self, point: PhotoPoint) -> f32 {
        self.strokes.iter().fold(0.0, |coverage, stroke| {
            stroke.over(coverage, stroke.distance_to(point))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stroke(points: &[PhotoPoint]) -> Stroke {
        Stroke {
            points: points.to_vec(),
            radius: 0.1,
            feather: 50.0,
            flow: 100.0,
            is_erasing: false,
        }
    }

    fn painted(strokes: Vec<Stroke>) -> BrushMask {
        BrushMask { strokes }
    }

    #[test]
    fn stroke_covers_along_its_path_and_fades_to_its_radius() {
        let mask = painted(vec![stroke(&[[0.2, 0.5], [0.8, 0.5]])]);

        assert_eq!(mask.coverage([0.5, 0.5]), 1.0);
        assert_eq!(mask.coverage([0.5, 0.54]), 1.0);
        assert!((mask.coverage([0.5, 0.575]) - 0.5).abs() < 1e-4);
        assert_eq!(mask.coverage([0.5, 0.61]), 0.0);
        assert_eq!(mask.coverage([0.95, 0.5]), 0.0);
    }

    #[test]
    fn single_point_stroke_is_a_dab() {
        let mask = painted(vec![stroke(&[[0.5, 0.5]])]);

        assert_eq!(mask.coverage([0.52, 0.5]), 1.0);
        assert_eq!(mask.coverage([0.7, 0.5]), 0.0);
    }

    #[test]
    fn flow_limits_one_stroke_and_strokes_build_up() {
        let light = Stroke {
            flow: 50.0,
            ..stroke(&[[0.2, 0.5], [0.8, 0.5]])
        };
        let once = painted(vec![light.clone()]);
        let twice = painted(vec![light.clone(), light]);

        assert_eq!(once.coverage([0.5, 0.5]), 0.5);
        assert_eq!(twice.coverage([0.5, 0.5]), 0.75);
    }

    #[test]
    fn erasing_stroke_removes_what_was_painted_under_it() {
        let eraser = Stroke {
            is_erasing: true,
            ..stroke(&[[0.5, 0.3], [0.5, 0.7]])
        };
        let mask = painted(vec![stroke(&[[0.2, 0.5], [0.8, 0.5]]), eraser]);

        assert_eq!(mask.coverage([0.5, 0.5]), 0.0);
        assert_eq!(mask.coverage([0.25, 0.5]), 1.0);
    }

    #[test]
    fn brush_size_sets_the_radius_and_stays_in_its_range() {
        let brush = Brush::default();

        assert_eq!(brush.radius(), 0.05);
        assert_eq!(brush.resized_by(500.0).size, 100.0);
        assert_eq!(brush.resized_by(-500.0).size, 1.0);
    }
}
