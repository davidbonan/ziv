use serde::{Deserialize, Serialize};

/// A tone before the curve `[0]` and the tone it gets `[1]`, both 0 (black) … 1 (white).
pub type CurvePoint = [f32; 2];

/// How close two points of a curve may get horizontally: one display level.
pub const CLOSEST_POINTS: f32 = 1.0 / 255.0;
pub const LOOKUP_SIZE: usize = 256;

const IDENTITY: [CurvePoint; 2] = [[0.0, 0.0], [1.0, 1.0]];

/// A mapping of display tones, drawn through its points from left to right.
/// The first and the last point are its end points; the default changes nothing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "Vec<CurvePoint>", into = "Vec<CurvePoint>")]
pub struct ToneCurve {
    points: Vec<CurvePoint>,
}

impl Default for ToneCurve {
    fn default() -> Self {
        Self {
            points: IDENTITY.to_vec(),
        }
    }
}

impl From<ToneCurve> for Vec<CurvePoint> {
    fn from(curve: ToneCurve) -> Self {
        curve.points
    }
}

impl TryFrom<Vec<CurvePoint>> for ToneCurve {
    type Error = &'static str;

    fn try_from(points: Vec<CurvePoint>) -> Result<Self, Self::Error> {
        let is_in_the_graph = |point: &CurvePoint| point.iter().all(|at| (0.0..=1.0).contains(at));
        let goes_rightwards = points.windows(2).all(|pair| pair[0][0] < pair[1][0]);
        if points.len() < 2 {
            return Err("a tone curve has two end points");
        }
        if !points.iter().all(is_in_the_graph) || !goes_rightwards {
            return Err("the points of a tone curve go from left to right inside the graph");
        }
        Ok(Self { points })
    }
}

/// The slope of the curve at each point: Fritsch–Carlson, so that the curve
/// never goes beyond the two points it runs between.
fn slopes(points: &[CurvePoint]) -> Vec<f32> {
    let chords: Vec<f32> = points
        .windows(2)
        .map(|pair| (pair[1][1] - pair[0][1]) / (pair[1][0] - pair[0][0]))
        .collect();
    let inner = chords.windows(2).map(|pair| match pair[0] * pair[1] > 0.0 {
        true => 2.0 * pair[0] * pair[1] / (pair[0] + pair[1]),
        false => 0.0,
    });
    let first = chords[0];
    let last = chords[chords.len() - 1];
    std::iter::once(first)
        .chain(inner)
        .chain(std::iter::once(last))
        .collect()
}

impl ToneCurve {
    pub fn points(&self) -> &[CurvePoint] {
        &self.points
    }

    fn last_index(&self) -> usize {
        self.points.len() - 1
    }

    pub fn is_end_point(&self, index: usize) -> bool {
        index == 0 || index == self.last_index()
    }

    pub fn is_identity(&self) -> bool {
        let [first, .., last] = self.points.as_slice() else {
            return false;
        };
        [*first, *last] == IDENTITY && self.points.iter().all(|[before, after]| before == after)
    }

    /// The tone `tone` gets. Flat left of the first point and right of the last.
    pub fn applied(&self, tone: f32) -> f32 {
        self.applied_with(&slopes(&self.points), tone)
    }

    fn applied_with(&self, slopes: &[f32], tone: f32) -> f32 {
        let points = &self.points;
        let [first_tone, first_result] = points[0];
        let [last_tone, last_result] = points[self.last_index()];
        if tone <= first_tone {
            return first_result;
        }
        if tone >= last_tone {
            return last_result;
        }
        let right = points.partition_point(|[before, _]| *before <= tone);
        let ([left_tone, left_result], [right_tone, right_result]) =
            (points[right - 1], points[right]);
        let width = right_tone - left_tone;
        let along = (tone - left_tone) / width;
        let (squared, cubed) = (along * along, along * along * along);
        (2.0 * cubed - 3.0 * squared + 1.0) * left_result
            + (cubed - 2.0 * squared + along) * width * slopes[right - 1]
            + (-2.0 * cubed + 3.0 * squared) * right_result
            + (cubed - squared) * width * slopes[right]
    }

    /// The curve at `LOOKUP_SIZE` tones evenly spread from black to white.
    pub fn lookup(&self) -> Vec<f32> {
        let slopes = slopes(&self.points);
        (0..LOOKUP_SIZE)
            .map(|level| self.applied_with(&slopes, level as f32 / (LOOKUP_SIZE - 1) as f32))
            .collect()
    }

    /// Where the point `index` may be: inside the graph, between its neighbours.
    fn kept_in_place(&self, index: usize, [tone, result]: CurvePoint) -> CurvePoint {
        let leftmost = match index {
            0 => 0.0,
            _ => self.points[index - 1][0] + CLOSEST_POINTS,
        };
        let rightmost = match self.points.get(index + 1) {
            Some([next, _]) => next - CLOSEST_POINTS,
            None => 1.0,
        };
        [tone.clamp(leftmost, rightmost), result.clamp(0.0, 1.0)]
    }

    pub fn with_point_moved(&self, index: usize, to: CurvePoint) -> Self {
        let mut points = self.points.clone();
        points[index] = self.kept_in_place(index, to);
        Self { points }
    }

    /// The curve with one more point at `point` and the index of that point.
    /// `None` where there is no room between two points, or outside the end points.
    pub fn with_point_added(&self, point: CurvePoint) -> Option<(Self, usize)> {
        let [tone, result] = point;
        let index = self.points.partition_point(|[before, _]| *before <= tone);
        let left = self.points.get(index.checked_sub(1)?)?[0] + CLOSEST_POINTS;
        let right = self.points.get(index)?[0] - CLOSEST_POINTS;
        if left > right {
            return None;
        }
        let mut points = self.points.clone();
        points.insert(index, [tone.clamp(left, right), result.clamp(0.0, 1.0)]);
        Some((Self { points }, index))
    }

    /// The end points stay.
    pub fn without_point(&self, index: usize) -> Self {
        if self.is_end_point(index) {
            return self.clone();
        }
        let mut points = self.points.clone();
        points.remove(index);
        Self { points }
    }
}

/// What a curve acts on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CurveChannel {
    /// The three channels alike.
    #[default]
    Rgb,
    Red,
    Green,
    Blue,
}

impl CurveChannel {
    pub const ALL: [Self; 4] = [Self::Rgb, Self::Red, Self::Green, Self::Blue];
}

/// The tone curves of a photo, one per curve channel.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ToneCurves {
    pub rgb: ToneCurve,
    pub red: ToneCurve,
    pub green: ToneCurve,
    pub blue: ToneCurve,
}

impl ToneCurves {
    pub fn of(&self, channel: CurveChannel) -> &ToneCurve {
        match channel {
            CurveChannel::Rgb => &self.rgb,
            CurveChannel::Red => &self.red,
            CurveChannel::Green => &self.green,
            CurveChannel::Blue => &self.blue,
        }
    }

    pub fn with(self, channel: CurveChannel, curve: ToneCurve) -> Self {
        match channel {
            CurveChannel::Rgb => Self { rgb: curve, ..self },
            CurveChannel::Red => Self { red: curve, ..self },
            CurveChannel::Green => Self {
                green: curve,
                ..self
            },
            CurveChannel::Blue => Self {
                blue: curve,
                ..self
            },
        }
    }

    pub fn is_identity(&self) -> bool {
        CurveChannel::ALL
            .iter()
            .all(|channel| self.of(*channel).is_identity())
    }

    /// `encoded`: a display pixel, each channel 0 … 1. The RGB curve first,
    /// then the curve of each channel.
    pub fn applied(&self, encoded: [f32; 3]) -> [f32; 3] {
        let [red, green, blue] = encoded.map(|channel| self.rgb.applied(channel));
        [
            self.red.applied(red),
            self.green.applied(green),
            self.blue.applied(blue),
        ]
    }

    /// What each channel gets at `LOOKUP_SIZE` tones from black to white: red, green, blue.
    pub fn lookup(&self) -> Vec<[f32; 3]> {
        let (red, green, blue) = (
            slopes(&self.red.points),
            slopes(&self.green.points),
            slopes(&self.blue.points),
        );
        self.rgb
            .lookup()
            .into_iter()
            .map(|tone| {
                [
                    self.red.applied_with(&red, tone),
                    self.green.applied_with(&green, tone),
                    self.blue.applied_with(&blue, tone),
                ]
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn curve(points: &[CurvePoint]) -> ToneCurve {
        ToneCurve::try_from(points.to_vec()).unwrap()
    }

    fn s_curve() -> ToneCurve {
        curve(&[[0.0, 0.0], [0.25, 0.15], [0.75, 0.85], [1.0, 1.0]])
    }

    fn tones() -> impl Iterator<Item = f32> {
        (0..=1000).map(|step| step as f32 / 1000.0)
    }

    #[test]
    fn default_curve_changes_no_tone() {
        let identity = ToneCurve::default();

        assert!(identity.is_identity());
        assert!(tones().all(|tone| (identity.applied(tone) - tone).abs() < 1e-6));
    }

    #[test]
    fn curve_goes_through_each_of_its_points() {
        let curve = s_curve();

        for [tone, result] in curve.points() {
            assert!((curve.applied(*tone) - result).abs() < 1e-6);
        }
        assert!(!curve.is_identity());
    }

    #[test]
    fn s_curve_darkens_the_dark_tones_and_brightens_the_bright_ones() {
        let curve = s_curve();

        assert!(curve.applied(0.2) < 0.2);
        assert!(curve.applied(0.8) > 0.8);
        assert!((curve.applied(0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn curve_never_leaves_the_graph_even_between_close_points() {
        let steep = curve(&[
            [0.0, 0.0],
            [0.4, 0.02],
            [0.42, 0.98],
            [0.5, 1.0],
            [1.0, 0.6],
        ]);

        assert!(tones().all(|tone| (0.0..=1.0).contains(&steep.applied(tone))));
    }

    #[test]
    fn rising_points_give_a_curve_that_never_falls() {
        let curve = s_curve();
        let results: Vec<f32> = tones().map(|tone| curve.applied(tone)).collect();

        assert!(results.windows(2).all(|pair| pair[0] <= pair[1]));
    }

    #[test]
    fn curve_is_flat_outside_its_end_points() {
        let lifted = curve(&[[0.2, 0.1], [0.8, 0.9]]);

        assert_eq!(lifted.applied(0.0), 0.1);
        assert_eq!(lifted.applied(0.2), 0.1);
        assert_eq!(lifted.applied(0.95), 0.9);
    }

    #[test]
    fn added_point_takes_its_place_between_its_neighbours() {
        let (curve, index) = s_curve().with_point_added([0.5, 0.6]).unwrap();

        assert_eq!(index, 2);
        assert_eq!(curve.points()[2], [0.5, 0.6]);
        assert_eq!(curve.points().len(), 5);
    }

    #[test]
    fn no_point_is_added_where_two_points_leave_no_room() {
        let crowded = curve(&[
            [0.0, 0.0],
            [0.5, 0.5],
            [0.5 + CLOSEST_POINTS, 0.6],
            [1.0, 1.0],
        ]);

        assert_eq!(
            crowded.with_point_added([0.5 + CLOSEST_POINTS / 2.0, 0.5]),
            None
        );
    }

    #[test]
    fn no_point_is_added_outside_the_end_points() {
        let lifted = curve(&[[0.2, 0.1], [0.8, 0.9]]);

        assert_eq!(lifted.with_point_added([0.1, 0.5]), None);
        assert_eq!(lifted.with_point_added([0.9, 0.5]), None);
    }

    #[test]
    fn moved_point_stops_at_its_neighbours_and_at_the_edges_of_the_graph() {
        let moved = s_curve().with_point_moved(1, [0.9, 1.4]);

        assert_eq!(moved.points()[1], [0.75 - CLOSEST_POINTS, 1.0]);
    }

    #[test]
    fn end_points_move_and_lift_black_or_dim_white() {
        let curve = ToneCurve::default()
            .with_point_moved(0, [-0.3, 0.2])
            .with_point_moved(1, [1.0, 0.8]);

        assert_eq!(curve.points(), [[0.0, 0.2], [1.0, 0.8]]);
        assert_eq!(curve.applied(0.0), 0.2);
        assert_eq!(curve.applied(1.0), 0.8);
    }

    #[test]
    fn inner_point_is_removed_and_end_points_stay() {
        let curve = s_curve();

        assert_eq!(curve.without_point(1).points().len(), 3);
        assert_eq!(curve.without_point(0), curve);
        assert_eq!(curve.without_point(3), curve);
    }

    #[test]
    fn point_on_the_diagonal_leaves_the_curve_the_identity() {
        let (curve, _) = ToneCurve::default().with_point_added([0.5, 0.5]).unwrap();

        assert!(curve.is_identity());
    }

    #[test]
    fn points_out_of_order_or_out_of_the_graph_are_not_a_curve() {
        for points in [
            vec![[0.0, 0.0]],
            vec![[0.5, 0.0], [0.5, 1.0]],
            vec![[0.6, 0.0], [0.4, 1.0]],
            vec![[0.0, 0.0], [1.0, 1.2]],
        ] {
            assert!(ToneCurve::try_from(points).is_err());
        }
    }

    #[test]
    fn rgb_curve_remaps_the_three_channels_alike() {
        let curves = ToneCurves::default().with(CurveChannel::Rgb, s_curve());
        let grey = curves.applied([0.3; 3]);

        assert!(grey[0] == grey[1] && grey[1] == grey[2]);
        assert_eq!(curves.lookup()[0], [0.0; 3]);
        assert_eq!(curves.lookup()[LOOKUP_SIZE - 1], [1.0; 3]);
    }

    fn raised_middle() -> ToneCurve {
        curve(&[[0.0, 0.0], [0.5, 0.7], [1.0, 1.0]])
    }

    #[test]
    fn channel_curve_remaps_its_channel_only() {
        let grey = [0.5; 3];
        let developed =
            [CurveChannel::Red, CurveChannel::Green, CurveChannel::Blue].map(|channel| {
                ToneCurves::default()
                    .with(channel, raised_middle())
                    .applied(grey)
            });

        assert_eq!(developed[0], [0.7, 0.5, 0.5]);
        assert_eq!(developed[1], [0.5, 0.7, 0.5]);
        assert_eq!(developed[2], [0.5, 0.5, 0.7]);
    }

    #[test]
    fn channel_curve_acts_on_what_the_rgb_curve_gives() {
        let dimmed_white = curve(&[[0.0, 0.0], [1.0, 0.5]]);
        let curves = ToneCurves::default()
            .with(CurveChannel::Rgb, dimmed_white)
            .with(CurveChannel::Blue, raised_middle());

        assert_eq!(curves.applied([1.0; 3]), [0.5, 0.5, 0.7]);
    }

    #[test]
    fn curves_are_the_identity_only_when_the_four_are() {
        assert!(ToneCurves::default().is_identity());
        for channel in CurveChannel::ALL {
            let curves = ToneCurves::default().with(channel, raised_middle());

            assert!(!curves.is_identity());
            assert_eq!(curves.of(channel), &raised_middle());
        }
    }

    #[test]
    fn lookup_holds_the_four_curves_combined() {
        let curves = ToneCurves::default()
            .with(CurveChannel::Rgb, s_curve())
            .with(CurveChannel::Red, raised_middle());
        let last_level = (LOOKUP_SIZE - 1) as f32;

        for (level, texel) in curves.lookup().into_iter().enumerate() {
            let expected = curves.applied([level as f32 / last_level; 3]);
            let is_close = texel
                .iter()
                .zip(expected)
                .all(|(one, other)| (one - other).abs() < 1e-6);
            assert!(is_close, "level {level}: {texel:?} is not {expected:?}");
        }
    }
}
