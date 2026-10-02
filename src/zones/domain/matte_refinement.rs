/// One value a texel, row after row.
#[derive(Debug, Clone, PartialEq)]
pub struct Plane {
    pub size: [usize; 2],
    pub values: Vec<f32>,
}

// How far around a texel the matte is fitted to the photo, in texels of the coarse matte.
const FIT_RADIUS: usize = 3;
// Colours closer than this (as a variance) are one flat area, not two sides of an outline.
const FLAT_VARIANCE: f32 = 1e-4;
const CHANNELS: usize = 3;

impl Plane {
    /// Coverage values, 0 … 255, as shares 0 … 1.
    pub fn of_coverage(size: [u32; 2], coverage: &[u8]) -> Self {
        let shares = coverage.iter().map(|value| f32::from(*value) / 255.0);
        Self {
            size: size.map(|side| side as usize),
            values: shares.collect(),
        }
    }

    fn combined(&self, other: &Self, combine: impl Fn(f32, f32) -> f32) -> Self {
        let values = self.values.iter().zip(&other.values);
        Self {
            size: self.size,
            values: values.map(|(own, others)| combine(*own, *others)).collect(),
        }
    }

    fn times(&self, other: &Self) -> Self {
        self.combined(other, |own, others| own * others)
    }

    fn minus(&self, other: &Self) -> Self {
        self.combined(other, |own, others| own - others)
    }

    // Sums of all the values above and left of each texel, one row and column wider.
    fn summed_area(&self) -> Vec<f64> {
        let [width, height] = self.size;
        let mut sums = vec![0.0; (width + 1) * (height + 1)];
        for row in 0..height {
            let mut along_row = 0.0;
            for column in 0..width {
                along_row += f64::from(self.values[row * width + column]);
                sums[(row + 1) * (width + 1) + column + 1] =
                    sums[row * (width + 1) + column + 1] + along_row;
            }
        }
        sums
    }

    // The mean of the texels within the fit radius of each texel, the photo's edge cutting the window.
    fn mean_around(&self) -> Self {
        let [width, height] = self.size;
        let sums = self.summed_area();
        let sum_at = |column: usize, row: usize| sums[row * (width + 1) + column];
        let window = |texel: usize, side: usize| {
            (
                texel.saturating_sub(FIT_RADIUS),
                (texel + FIT_RADIUS + 1).min(side),
            )
        };
        let mut values = Vec::with_capacity(self.values.len());
        for row in 0..height {
            let (top, bottom) = window(row, height);
            for column in 0..width {
                let (left, right) = window(column, width);
                let sum = sum_at(right, bottom) - sum_at(left, bottom) - sum_at(right, top)
                    + sum_at(left, top);
                let count = (right - left) * (bottom - top);
                values.push((sum / count as f64) as f32);
            }
        }
        Self {
            size: self.size,
            values,
        }
    }

    fn texel(&self, column: f32, row: f32) -> f32 {
        let [width, height] = self.size;
        let column = column.clamp(0.0, (width - 1) as f32) as usize;
        let row = row.clamp(0.0, (height - 1) as f32) as usize;
        self.values[row * width + column]
    }

    // The value between texels at `[across, down]`, each 0 … 1 over the plane.
    fn at_share(&self, [across, down]: [f32; 2]) -> f32 {
        let column = across * self.size[0] as f32 - 0.5;
        let row = down * self.size[1] as f32 - 0.5;
        let [left, top] = [column.floor(), row.floor()];
        let [rightwards, downwards] = [column - left, row - top];
        let upper =
            self.texel(left, top) * (1.0 - rightwards) + self.texel(left + 1.0, top) * rightwards;
        let lower = self.texel(left, top + 1.0) * (1.0 - rightwards)
            + self.texel(left + 1.0, top + 1.0) * rightwards;
        upper * (1.0 - downwards) + lower * downwards
    }
}

/// The red, green and blue of display-encoded pixels, 0 … 1, a plane each.
pub struct Colours {
    channels: [Plane; CHANNELS],
}

impl Colours {
    pub fn of_rgba(size: [u32; 2], rgba: &[u8]) -> Self {
        let (pixels, _) = rgba.as_chunks::<4>();
        let channel = |channel: usize| Plane {
            size: size.map(|side| side as usize),
            values: pixels
                .iter()
                .map(|pixel| f32::from(pixel[channel]) / 255.0)
                .collect(),
        };
        Self {
            channels: [channel(0), channel(1), channel(2)],
        }
    }
}

/// A coarse matte and the colours of what the model saw, texel for texel.
pub struct CoarseMatte {
    pub matte: Plane,
    pub colours: Colours,
}

// What `matrix`, symmetric, turns into `product`. A flat area, whose matrix
// is close to singular, was kept away from it by `FLAT_VARIANCE`.
fn solved(matrix: [[f32; CHANNELS]; CHANNELS], product: [f32; CHANNELS]) -> [f32; CHANNELS] {
    let [[a, b, c], [_, d, e], [_, _, f]] = matrix;
    let cofactors = [
        [d * f - e * e, c * e - b * f, b * e - c * d],
        [c * e - b * f, a * f - c * c, b * c - a * e],
        [b * e - c * d, b * c - a * e, a * d - b * b],
    ];
    let determinant = a * cofactors[0][0] + b * cofactors[0][1] + c * cofactors[0][2];
    cofactors
        .map(|row| (row[0] * product[0] + row[1] * product[1] + row[2] * product[2]) / determinant)
}

// Around each texel, the matte as a weight a channel of the photo's colour plus an offset.
struct Fit {
    weights: [Plane; CHANNELS],
    offset: Plane,
}

fn fit_around_each_texel(coarse: &CoarseMatte) -> Fit {
    let CoarseMatte { matte, colours } = coarse;
    let channels = &colours.channels;
    let mean_matte = matte.mean_around();
    let means = channels.each_ref().map(Plane::mean_around);
    let covariance_with = |channel: usize, other: &Plane, mean_other: &Plane| {
        channels[channel]
            .times(other)
            .mean_around()
            .minus(&means[channel].times(mean_other))
    };
    let with_matte = [0, 1, 2].map(|channel| covariance_with(channel, matte, &mean_matte));
    let between = |one: usize, other: usize| covariance_with(one, &channels[other], &means[other]);
    let (red_red, red_green, red_blue) = (between(0, 0), between(0, 1), between(0, 2));
    let (green_green, green_blue, blue_blue) = (between(1, 1), between(1, 2), between(2, 2));

    let texels = matte.values.len();
    let mut weights = [0, 1, 2].map(|_| Vec::with_capacity(texels));
    let mut offsets = Vec::with_capacity(texels);
    for texel in 0..texels {
        let at = |plane: &Plane| plane.values[texel];
        let colour_spread = [
            [at(&red_red) + FLAT_VARIANCE, at(&red_green), at(&red_blue)],
            [
                at(&red_green),
                at(&green_green) + FLAT_VARIANCE,
                at(&green_blue),
            ],
            [
                at(&red_blue),
                at(&green_blue),
                at(&blue_blue) + FLAT_VARIANCE,
            ],
        ];
        let weight = solved(colour_spread, with_matte.each_ref().map(at));
        let explained: f32 = (0..CHANNELS)
            .map(|channel| weight[channel] * at(&means[channel]))
            .sum();
        offsets.push(at(&mean_matte) - explained);
        for channel in 0..CHANNELS {
            weights[channel].push(weight[channel]);
        }
    }
    let plane = |values: Vec<f32>| Plane {
        size: matte.size,
        values,
    };
    Fit {
        weights: weights.map(|values| plane(values).mean_around()),
        offset: plane(offsets).mean_around(),
    }
}

/// The coarse matte at the size of `fine`, 0 … 255, its edges moved onto the
/// outlines of the photo (a guided filter): around each texel the matte is
/// fitted as a line of the photo's colour, and that line is read again with
/// the finer colours.
pub fn refined_coverage(coarse: &CoarseMatte, fine: &Colours) -> Vec<u8> {
    let fit = fit_around_each_texel(coarse);
    let [width, height] = fine.channels[0].size;
    let mut coverage = Vec::with_capacity(width * height);
    for row in 0..height {
        for column in 0..width {
            let share = [
                (column as f32 + 0.5) / width as f32,
                (row as f32 + 0.5) / height as f32,
            ];
            let weighted = (0..CHANNELS).map(|channel| {
                fit.weights[channel].at_share(share)
                    * fine.channels[channel].values[row * width + column]
            });
            let fitted = weighted.sum::<f32>() + fit.offset.at_share(share);
            coverage.push((fitted.clamp(0.0, 1.0) * 255.0).round() as u8);
        }
    }
    coverage
}

#[cfg(test)]
mod tests {
    use super::*;

    const COARSE: [usize; 2] = [16, 4];
    const FINE: [usize; 2] = [64, 16];
    const ORANGE: [f32; 3] = [0.9, 0.5, 0.2];
    const BLUE: [f32; 3] = [0.3, 0.5, 0.8];
    const GREY: [f32; 3] = [0.5; 3];

    fn plane(size: [usize; 2], value_at_column: impl Fn(usize) -> f32) -> Plane {
        let row: Vec<f32> = (0..size[0]).map(value_at_column).collect();
        Plane {
            size,
            values: row.repeat(size[1]),
        }
    }

    // `left` up to the middle column, `right` from there.
    fn two_colours(size: [usize; 2], [left, right]: [[f32; 3]; 2]) -> Colours {
        let channel = |channel: usize| {
            plane(size, |column| match column < size[0] / 2 {
                true => left[channel],
                false => right[channel],
            })
        };
        Colours {
            channels: [channel(0), channel(1), channel(2)],
        }
    }

    // Fully covered at the left, not at all at the right, half way in the middle.
    fn soft_matte() -> Plane {
        plane(COARSE, |column| {
            (1.0 - (column as f32 - 4.0) / 8.0).clamp(0.0, 1.0)
        })
    }

    fn middle_row(coverage: &[u8]) -> &[u8] {
        &coverage[FINE[0] * 8..FINE[0] * 9]
    }

    #[test]
    fn soft_edge_of_the_matte_lands_on_the_outline_of_the_photo() {
        let coarse = CoarseMatte {
            matte: soft_matte(),
            colours: two_colours(COARSE, [BLUE, ORANGE]),
        };

        let refined = refined_coverage(&coarse, &two_colours(FINE, [BLUE, ORANGE]));

        let row = middle_row(&refined);
        let drop_at_the_outline = row[31] - row[32];
        assert!(drop_at_the_outline > 80, "{row:?}");
    }

    #[test]
    fn matte_over_a_flat_photo_keeps_its_soft_edge() {
        let coarse = CoarseMatte {
            matte: soft_matte(),
            colours: two_colours(COARSE, [GREY, GREY]),
        };

        let refined = refined_coverage(&coarse, &two_colours(FINE, [GREY, GREY]));

        let row = middle_row(&refined);
        assert!(row.is_sorted_by(|left, right| left >= right), "{row:?}");
        assert!(row[0] > 195 && row[63] < 60, "{row:?}");
    }

    #[test]
    fn full_coverage_stays_full_whatever_the_photo_shows() {
        let coarse = CoarseMatte {
            matte: plane(COARSE, |_| 1.0),
            colours: two_colours(COARSE, [BLUE, ORANGE]),
        };

        let refined = refined_coverage(&coarse, &two_colours(FINE, [BLUE, ORANGE]));

        assert!(refined.iter().all(|value| *value == 255));
    }
}
