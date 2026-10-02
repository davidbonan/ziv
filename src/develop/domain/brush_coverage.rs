use std::ops::Range;

use super::brush::{BrushMask, Stroke, distance_squared_to_segment};
use super::mask::{PhotoPoint, photo_extent};

/// Columns and rows of a coverage image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Texels {
    pub columns: Range<u32>,
    pub rows: Range<u32>,
}

impl Texels {
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty() || self.rows.is_empty()
    }

    fn joined(&self, other: &Self) -> Self {
        if self.is_empty() {
            return other.clone();
        }
        if other.is_empty() {
            return self.clone();
        }
        Self {
            columns: self.columns.start.min(other.columns.start)
                ..self.columns.end.max(other.columns.end),
            rows: self.rows.start.min(other.rows.start)..self.rows.end.max(other.rows.end),
        }
    }
}

/// A brush mask as an image of its coverage, 0 … 255. Kept up to date stroke
/// by stroke: showing a mask that only grew repaints only where it grew.
pub struct BrushCoverage {
    size: [u32; 2],
    shown: BrushMask,
    values: Vec<u8>,
    before_last_stroke: Vec<u8>,
    // For each texel, how far the last stroke's path passes, squared.
    last_stroke_distances: Vec<f32>,
}

impl BrushCoverage {
    pub fn of_size(size: [u32; 2]) -> Self {
        let texels = size[0] as usize * size[1] as usize;
        Self {
            size,
            shown: BrushMask::default(),
            values: vec![0; texels],
            before_last_stroke: vec![0; texels],
            last_stroke_distances: vec![f32::MAX; texels],
        }
    }

    pub fn size(&self) -> [u32; 2] {
        self.size
    }

    pub fn values(&self) -> &[u8] {
        &self.values
    }

    pub fn everything(&self) -> Texels {
        Texels {
            columns: 0..self.size[0],
            rows: 0..self.size[1],
        }
    }

    fn texel_centre(&self, column: u32, row: u32) -> PhotoPoint {
        let [right, bottom] = photo_extent(self.size);
        [
            (column as f32 + 0.5) / self.size[0] as f32 * right,
            (row as f32 + 0.5) / self.size[1] as f32 * bottom,
        ]
    }

    // The texels a segment of a stroke of `radius` can reach.
    fn texels_within(&self, segment: [PhotoPoint; 2], radius: f32) -> Texels {
        let extent = photo_extent(self.size);
        let along = |axis: usize| {
            let texels_per_unit = self.size[axis] as f32 / extent[axis];
            let low = segment[0][axis].min(segment[1][axis]) - radius;
            let high = segment[0][axis].max(segment[1][axis]) + radius;
            let first = (low * texels_per_unit).floor().max(0.0) as u32;
            let end = ((high * texels_per_unit).ceil().max(0.0) as u32 + 1).min(self.size[axis]);
            first.min(end)..end
        };
        Texels {
            columns: along(0),
            rows: along(1),
        }
    }

    fn index(&self, column: u32, row: u32) -> usize {
        row as usize * self.size[0] as usize + column as usize
    }

    fn begin_stroke(&mut self) {
        self.before_last_stroke.copy_from_slice(&self.values);
        self.last_stroke_distances.fill(f32::MAX);
    }

    // Brings the last stroke's path closer to the texels around `segment`,
    // and repaints them.
    fn paint_segment(&mut self, stroke: &Stroke, segment: [PhotoPoint; 2]) -> Texels {
        let reached = self.texels_within(segment, stroke.radius);
        for row in reached.rows.clone() {
            for column in reached.columns.clone() {
                let index = self.index(column, row);
                let centre = self.texel_centre(column, row);
                let squared = distance_squared_to_segment(centre, segment[0], segment[1]);
                let nearest = self.last_stroke_distances[index].min(squared);
                self.last_stroke_distances[index] = nearest;
                let before = f32::from(self.before_last_stroke[index]) / 255.0;
                let after = stroke.over(before, nearest.sqrt());
                self.values[index] = (after * 255.0).round() as u8;
            }
        }
        reached
    }

    fn paint_segments(&mut self, stroke: &Stroke, already_painted: usize) -> Texels {
        let nothing = Texels {
            columns: 0..0,
            rows: 0..0,
        };
        stroke
            .segments()
            .skip(already_painted)
            .fold(nothing, |repainted, segment| {
                repainted.joined(&self.paint_segment(stroke, segment))
            })
    }

    fn repaint_all(&mut self, mask: &BrushMask) -> Texels {
        self.values.fill(0);
        for stroke in &mask.strokes {
            self.begin_stroke();
            self.paint_segments(stroke, 0);
        }
        self.everything()
    }

    // The number of segments of the shown last stroke, when `mask` is the
    // shown mask with that stroke gone further.
    fn segments_shown_of_last_stroke(&self, mask: &BrushMask) -> Option<usize> {
        let (shown_last, shown_earlier) = self.shown.strokes.split_last()?;
        let (last, earlier) = mask.strokes.split_last()?;
        let is_same_brush = Stroke {
            points: Vec::new(),
            ..shown_last.clone()
        } == Stroke {
            points: Vec::new(),
            ..last.clone()
        };
        let is_grown = earlier == shown_earlier
            && is_same_brush
            && last.points.starts_with(&shown_last.points);
        is_grown.then(|| shown_last.segments().count())
    }

    /// Makes the image the coverage of `mask`. Returns the texels that
    /// changed, `None` when the image already showed that mask.
    pub fn show(&mut self, mask: &BrushMask) -> Option<Texels> {
        if *mask == self.shown {
            return None;
        }
        let shown_count = self.shown.strokes.len();
        let is_one_stroke_more = mask.strokes.len() == shown_count + 1
            && mask.strokes[..shown_count] == self.shown.strokes;
        let repainted = match (
            self.segments_shown_of_last_stroke(mask),
            mask.strokes.last(),
        ) {
            (Some(shown_segments), Some(last)) => self.paint_segments(last, shown_segments),
            (None, Some(last)) if is_one_stroke_more => {
                self.begin_stroke();
                self.paint_segments(last, 0)
            }
            _ => self.repaint_all(mask),
        };
        self.shown = mask.clone();
        Some(repainted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIZE: [u32; 2] = [64, 32];

    fn stroke(points: &[PhotoPoint]) -> Stroke {
        Stroke {
            points: points.to_vec(),
            radius: 0.08,
            feather: 60.0,
            flow: 80.0,
            is_erasing: false,
        }
    }

    fn eraser(points: &[PhotoPoint]) -> Stroke {
        Stroke {
            is_erasing: true,
            ..stroke(points)
        }
    }

    fn assert_shows(coverage: &BrushCoverage, mask: &BrushMask) {
        for row in 0..SIZE[1] {
            for column in 0..SIZE[0] {
                let expected = mask.coverage(coverage.texel_centre(column, row)) * 255.0;
                let shown = f32::from(coverage.values()[coverage.index(column, row)]);
                assert!(
                    (shown - expected).abs() <= 1.5,
                    "texel {column},{row}: {shown} instead of {expected}"
                );
            }
        }
    }

    #[test]
    fn image_is_the_coverage_of_the_mask_at_each_texel() {
        let mask = BrushMask {
            strokes: vec![
                stroke(&[[0.1, 0.1], [0.6, 0.3], [0.9, 0.2]]),
                eraser(&[[0.5, 0.0], [0.5, 0.5]]),
                stroke(&[[0.3, 0.4]]),
            ],
        };
        let mut coverage = BrushCoverage::of_size(SIZE);

        assert_eq!(coverage.show(&mask), Some(coverage.everything()));
        assert_shows(&coverage, &mask);
    }

    #[test]
    fn growing_stroke_repaints_only_around_what_was_added() {
        let begun = BrushMask {
            strokes: vec![stroke(&[[0.1, 0.1], [0.2, 0.1]])],
        };
        let grown = BrushMask {
            strokes: vec![stroke(&[[0.1, 0.1], [0.2, 0.1], [0.3, 0.2]])],
        };
        let mut coverage = BrushCoverage::of_size(SIZE);
        coverage.show(&begun);

        let repainted = coverage.show(&grown).unwrap();

        assert!(repainted.columns.end < SIZE[0] && repainted.rows.end < SIZE[1]);
        assert_shows(&coverage, &grown);
    }

    #[test]
    fn stroke_after_stroke_and_undone_stroke_show_the_mask_they_leave() {
        let first = stroke(&[[0.1, 0.1], [0.9, 0.4]]);
        let second = eraser(&[[0.5, 0.0], [0.5, 0.5]]);
        let one = BrushMask {
            strokes: vec![first.clone()],
        };
        let two = BrushMask {
            strokes: vec![first, second],
        };
        let mut coverage = BrushCoverage::of_size(SIZE);

        coverage.show(&one);
        coverage.show(&two);
        assert_shows(&coverage, &two);

        coverage.show(&one);
        assert_shows(&coverage, &one);
    }

    #[test]
    fn showing_the_same_mask_again_repaints_nothing() {
        let mask = BrushMask {
            strokes: vec![stroke(&[[0.3, 0.3]])],
        };
        let mut coverage = BrushCoverage::of_size(SIZE);
        coverage.show(&mask);

        assert_eq!(coverage.show(&mask), None);
    }
}
