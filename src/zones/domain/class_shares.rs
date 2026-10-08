use super::matte_refinement::Plane;
use super::photo_view::PhotoRegion;
use crate::models::domain::model_runner::ModelOutput;

// A patch this many times smaller than the largest one of its class is not counted.
const SPECK_SHARE: usize = 4;

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

    /// The patches of touching texels where `class` is likelier than not,
    /// each as the texels it is made of.
    fn patches_of(&self, class: usize) -> Vec<Vec<usize>> {
        let map = self.map_of(class);
        let mut is_left: Vec<bool> = map.iter().map(|share| *share > 0.5).collect();
        let mut patches = Vec::new();
        for start in 0..is_left.len() {
            if !is_left[start] {
                continue;
            }
            is_left[start] = false;
            let mut patch = vec![start];
            let mut reached = 0;
            while let Some(&texel) = patch.get(reached) {
                reached += 1;
                let (column, row) = (texel % self.side, texel / self.side);
                let neighbours = [
                    (column > 0).then(|| texel - 1),
                    (column + 1 < self.side).then(|| texel + 1),
                    (row > 0).then(|| texel - self.side),
                    (row + 1 < self.side).then(|| texel + self.side),
                ];
                for neighbour in neighbours.into_iter().flatten() {
                    if is_left[neighbour] {
                        is_left[neighbour] = false;
                        patch.push(neighbour);
                    }
                }
            }
            patches.push(patch);
        }
        patches
    }

    fn extent_of_texels(&self, texels: &[usize]) -> Option<PhotoRegion> {
        let columns = texels.iter().map(|texel| texel % self.side);
        let rows = texels.iter().map(|texel| texel / self.side);
        let (left, right) = (columns.clone().min()?, columns.max()?);
        let (top, bottom) = (rows.clone().min()?, rows.max()?);
        let share = |texels: usize| texels as f32 / self.side as f32;
        Some(PhotoRegion {
            min: [share(left), share(top)],
            size: [share(right + 1 - left), share(bottom + 1 - top)],
        })
    }

    /// What holds the patch of `class` nearest the middle column of the map,
    /// in shares of the map; `None` when the class is nowhere. A map centred
    /// on a person may show a neighbour's face too: the person's own is the
    /// one in the middle. Patches much smaller than the largest are specks.
    pub fn centred_extent_of(&self, class: usize) -> Option<PhotoRegion> {
        let patches = self.patches_of(class);
        let largest = patches.iter().map(Vec::len).max()?;
        let extents = patches
            .iter()
            .filter(|patch| patch.len() * SPECK_SHARE >= largest)
            .filter_map(|patch| self.extent_of_texels(patch));
        let off_the_middle = |extent: &PhotoRegion| (extent.centre()[0] - 0.5).abs();
        extents.min_by(|one, other| off_the_middle(one).total_cmp(&off_the_middle(other)))
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
        assert_eq!(shares.centred_extent_of(1), Some(expected));
    }

    // Class 1 over a map of 8 by 8, where `is_of_class` says so.
    fn shares_where(is_of_class: impl Fn(usize, usize) -> bool) -> ClassShares {
        let values = (0..2).flat_map(|class| {
            let is_of_class = &is_of_class;
            (0..64).map(
                move |texel| match is_of_class(texel % 8, texel / 8) == (class == 1) {
                    true => 8.0,
                    false => -8.0,
                },
            )
        });
        let logits = ModelOutput {
            shape: vec![1, 2, 8, 8],
            values: values.collect(),
        };
        ClassShares::of_logits(&logits).unwrap()
    }

    #[test]
    fn of_two_patches_the_extent_is_of_the_one_nearest_the_middle_column() {
        let a_neighbour_at_the_edge = |column, row| column < 2 && row < 4;
        let in_the_middle = |column, row| (3..5).contains(&column) && (4..6).contains(&row);
        let shares = shares_where(|column, row| {
            a_neighbour_at_the_edge(column, row) || in_the_middle(column, row)
        });

        let expected = PhotoRegion {
            min: [0.375, 0.5],
            size: [0.25, 0.25],
        };
        assert_eq!(shares.centred_extent_of(1), Some(expected));
    }

    #[test]
    fn speck_in_the_middle_does_not_take_the_place_of_a_patch() {
        let a_patch = |column, row| column < 3 && row < 3;
        let a_speck = |column, row| column == 4 && row == 6;
        let shares = shares_where(|column, row| a_patch(column, row) || a_speck(column, row));

        let expected = PhotoRegion {
            min: [0.0, 0.0],
            size: [0.375, 0.375],
        };
        assert_eq!(shares.centred_extent_of(1), Some(expected));
    }

    #[test]
    fn class_found_nowhere_has_no_extent() {
        let nowhere = ModelOutput {
            shape: vec![1, 2, 2, 2],
            values: vec![8.0, 8.0, 8.0, 8.0, -8.0, -8.0, -8.0, -8.0],
        };

        assert_eq!(
            ClassShares::of_logits(&nowhere)
                .unwrap()
                .centred_extent_of(1),
            None
        );
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
