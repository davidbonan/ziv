use super::matte_refinement::Plane;
use super::photo_view::PhotoRegion;
use crate::models::domain::model_runner::ModelOutput;

/// For each texel of a square map, how likely each class is: the answer of a
/// model that tells classes apart, its logits turned into shares that add up to 1.
pub struct ClassShares {
    classes: usize,
    side: usize,
    // Class after class, each a whole map.
    shares: Vec<f32>,
}

impl ClassShares {
    /// `None` when `logits` is not a square map a class.
    pub fn of_logits(logits: &ModelOutput) -> Option<Self> {
        let [_, classes, height, width] = logits.shape[..] else {
            return None;
        };
        let texels = width * height;
        if width != height || texels == 0 || logits.values.len() != classes * texels {
            return None;
        }
        let mut shares = logits.values.clone();
        for texel in 0..texels {
            let of_class = |class: usize| class * texels + texel;
            let highest = (0..classes)
                .map(|class| logits.values[of_class(class)])
                .fold(f32::MIN, f32::max);
            let mut total = 0.0;
            for class in 0..classes {
                shares[of_class(class)] = (logits.values[of_class(class)] - highest).exp();
                total += shares[of_class(class)];
            }
            for class in 0..classes {
                shares[of_class(class)] /= total;
            }
        }
        Some(Self {
            classes,
            side: width,
            shares,
        })
    }

    fn map_of(&self, class: usize) -> &[f32] {
        let texels = self.side * self.side;
        &self.shares[class * texels..(class + 1) * texels]
    }

    /// How likely each texel is one of `classes`.
    pub fn share_of(&self, classes: &[usize]) -> Plane {
        let mut values = vec![0.0; self.side * self.side];
        for class in classes.iter().filter(|class| **class < self.classes) {
            for (value, share) in values.iter_mut().zip(self.map_of(*class)) {
                *value += share;
            }
        }
        Plane {
            size: [self.side; 2],
            values,
        }
    }

    /// What holds the texels where `class` is likelier than not, in shares of
    /// the map; `None` when there is none.
    pub fn extent_of(&self, class: usize) -> Option<PhotoRegion> {
        let map = self.map_of(class);
        let is_of_class = |column: usize, row: usize| map[row * self.side + column] > 0.5;
        let columns =
            (0..self.side).filter(|column| (0..self.side).any(|row| is_of_class(*column, row)));
        let rows =
            (0..self.side).filter(|row| (0..self.side).any(|column| is_of_class(column, *row)));
        let (left, right) = (columns.clone().min()?, columns.max()?);
        let (top, bottom) = (rows.clone().min()?, rows.max()?);
        let share = |texels: usize| texels as f32 / self.side as f32;
        Some(PhotoRegion {
            min: [share(left), share(top)],
            size: [share(right + 1 - left), share(bottom + 1 - top)],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Three classes over a map of 4 by 4: class 1 in the two middle rows of
    // the left half, class 2 in the last row, class 0 elsewhere.
    fn logits() -> ModelOutput {
        let class_at = |texel: usize| match (texel % 4, texel / 4) {
            (0..2, 1..3) => 1,
            (_, 3) => 2,
            _ => 0,
        };
        let values = (0..3).flat_map(|class| {
            (0..16).map(move |texel| if class_at(texel) == class { 8.0 } else { -8.0 })
        });
        ModelOutput {
            shape: vec![1, 3, 4, 4],
            values: values.collect(),
        }
    }

    #[test]
    fn share_of_classes_is_how_likely_a_texel_is_one_of_them() {
        let shares = ClassShares::of_logits(&logits()).unwrap();

        let middle_or_last = shares.share_of(&[1, 2]);

        assert!(middle_or_last.values[4] > 0.99);
        assert!(middle_or_last.values[15] > 0.99);
        assert!(middle_or_last.values[0] < 0.01);
    }

    #[test]
    fn extent_holds_the_texels_of_the_class() {
        let shares = ClassShares::of_logits(&logits()).unwrap();

        let expected = PhotoRegion {
            min: [0.0, 0.25],
            size: [0.5, 0.5],
        };
        assert_eq!(shares.extent_of(1), Some(expected));
    }

    #[test]
    fn class_found_nowhere_has_no_extent() {
        let nowhere = ModelOutput {
            shape: vec![1, 2, 2, 2],
            values: vec![8.0, 8.0, 8.0, 8.0, -8.0, -8.0, -8.0, -8.0],
        };

        assert_eq!(ClassShares::of_logits(&nowhere).unwrap().extent_of(1), None);
    }

    #[test]
    fn logits_that_are_not_square_maps_are_refused() {
        let ragged = ModelOutput {
            shape: vec![1, 2, 2, 3],
            values: vec![0.0; 12],
        };

        assert!(ClassShares::of_logits(&ragged).is_none());
    }
}
