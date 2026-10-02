use serde::{Deserialize, Serialize};

use super::feathered_edge::feathered_coverage;
use super::linear_gradient::SHORTEST_GRADIENT_SQUARED;
use super::mask::PhotoPoint;

pub const FEWEST_CORNERS: usize = 3;
pub const MOST_CORNERS: usize = 16;
pub const CENTRE_HANDLE: usize = 0;

/// A closed outline covered inside, feathered inward from its edge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Polygon {
    pub corners: Vec<PhotoPoint>,
    pub feather: f32,
}

fn minus(a: PhotoPoint, b: PhotoPoint) -> [f32; 2] {
    [a[0] - b[0], a[1] - b[1]]
}

fn dot(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}

impl Polygon {
    /// `None` unless `corners` can enclose something and fit the engine.
    pub fn closing(corners: Vec<PhotoPoint>) -> Option<Self> {
        (FEWEST_CORNERS..=MOST_CORNERS)
            .contains(&corners.len())
            .then_some(Self {
                corners,
                feather: 0.0,
            })
    }

    /// How deep `point` lies inside the outline; negative outside.
    fn depth(&self, point: PhotoPoint) -> f32 {
        let mut nearest_squared = f32::MAX;
        let mut is_inside = false;
        let mut previous = self.corners[self.corners.len() - 1];
        for corner in self.corners.iter().copied() {
            let edge = minus(previous, corner);
            let to_point = minus(point, corner);
            let along = dot(to_point, edge) / dot(edge, edge).max(SHORTEST_GRADIENT_SQUARED);
            let along = along.clamp(0.0, 1.0);
            let to_edge = [to_point[0] - edge[0] * along, to_point[1] - edge[1] * along];
            nearest_squared = nearest_squared.min(dot(to_edge, to_edge));
            let crossing = [
                point[1] >= corner[1],
                point[1] < previous[1],
                edge[0] * to_point[1] > edge[1] * to_point[0],
            ];
            if crossing == [true; 3] || crossing == [false; 3] {
                is_inside = !is_inside;
            }
            previous = corner;
        }
        match is_inside {
            true => nearest_squared.sqrt(),
            false => -nearest_squared.sqrt(),
        }
    }

    pub fn coverage(&self, point: PhotoPoint) -> f32 {
        feathered_coverage(self.depth(point), self.feather)
    }

    fn centre(&self) -> PhotoPoint {
        let count = self.corners.len() as f32;
        let sum = self.corners.iter().fold([0.0; 2], |sum, corner| {
            [sum[0] + corner[0], sum[1] + corner[1]]
        });
        [sum[0] / count, sum[1] / count]
    }

    /// The centre, then each corner.
    pub fn handles(&self) -> Vec<PhotoPoint> {
        std::iter::once(self.centre())
            .chain(self.corners.iter().copied())
            .collect()
    }

    pub fn with_handle_at(&self, handle: usize, point: PhotoPoint) -> Self {
        let mut moved = self.clone();
        if handle == CENTRE_HANDLE {
            let carried = minus(point, self.centre());
            for corner in &mut moved.corners {
                *corner = [corner[0] + carried[0], corner[1] + carried[1]];
            }
            return moved;
        }
        if let Some(corner) = moved.corners.get_mut(handle - 1) {
            *corner = point;
        }
        moved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> Polygon {
        Polygon {
            corners: vec![[0.2, 0.2], [0.8, 0.2], [0.2, 0.8]],
            feather: 0.0,
        }
    }

    #[test]
    fn sharp_polygon_covers_its_inside_only() {
        assert_eq!(triangle().coverage([0.3, 0.3]), 1.0);
        assert_eq!(triangle().coverage([0.6, 0.6]), 0.0);
        assert_eq!(triangle().coverage([0.1, 0.5]), 0.0);
    }

    #[test]
    fn corners_listed_the_other_way_round_cover_the_same() {
        let reversed = Polygon {
            corners: triangle().corners.into_iter().rev().collect(),
            feather: 0.0,
        };

        assert_eq!(reversed.coverage([0.3, 0.3]), 1.0);
        assert_eq!(reversed.coverage([0.6, 0.6]), 0.0);
    }

    #[test]
    fn feather_rises_inward_from_the_outline() {
        let soft = Polygon {
            feather: 50.0,
            ..triangle()
        };

        assert!((soft.coverage([0.25, 0.4]) - 0.5).abs() < 1e-4);
        assert!(soft.coverage([0.2, 0.4]) < 1e-6);
    }

    #[test]
    fn fewer_than_three_or_more_than_sixteen_corners_close_nothing() {
        assert_eq!(Polygon::closing(vec![[0.0, 0.0], [1.0, 0.0]]), None);
        assert_eq!(Polygon::closing(vec![[0.0, 0.0]; MOST_CORNERS + 1]), None);
        assert!(Polygon::closing(triangle().corners).is_some());
    }

    #[test]
    fn corner_handle_moves_one_corner_and_the_centre_carries_all() {
        let reshaped = triangle().with_handle_at(2, [0.9, 0.1]);
        let centre = triangle().handles()[CENTRE_HANDLE];
        let moved = triangle().with_handle_at(CENTRE_HANDLE, [centre[0] + 0.1, centre[1]]);

        assert_eq!(reshaped.corners, [[0.2, 0.2], [0.9, 0.1], [0.2, 0.8]]);
        assert!((moved.corners[1][0] - 0.9).abs() < 1e-6);
        assert!((moved.corners[1][1] - 0.2).abs() < 1e-6);
    }
}
