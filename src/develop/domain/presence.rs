use crate::color::domain::working_space::luminance;

use super::adjustments::Adjustments;

const FULL_ADJUSTMENT: f32 = 100.0;

/// Vibrance and saturation of an edit, as the quantities the math works with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Presence {
    /// −1 … 1: how much the color of a grey-ish pixel is scaled down or up.
    pub vibrance: f32,
    /// What every color's distance to grey is multiplied by.
    pub saturation: f32,
}

// 0 for a grey, 1 for a color with one channel at zero.
fn colorfulness(working: [f32; 3]) -> f32 {
    let strongest = working.into_iter().fold(f32::MIN, f32::max);
    let weakest = working.into_iter().fold(f32::MAX, f32::min);
    if strongest <= 0.0 {
        return 0.0;
    }
    ((strongest - weakest) / strongest).clamp(0.0, 1.0)
}

impl Presence {
    pub fn of(edit: &Adjustments) -> Self {
        Self {
            vibrance: edit.vibrance / FULL_ADJUSTMENT,
            saturation: 1.0 + edit.saturation / FULL_ADJUSTMENT,
        }
    }

    /// Moves the pixel away from or toward the grey of the same luminance.
    pub fn applied(&self, working: [f32; 3]) -> [f32; 3] {
        let grey = luminance(working);
        let vibrance = 1.0 + self.vibrance * (1.0 - colorfulness(working));
        let scale = vibrance * self.saturation;
        working.map(|channel| grey + (channel - grey) * scale)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MUTED: [f32; 3] = [0.42, 0.40, 0.38];
    const VIVID: [f32; 3] = [0.80, 0.20, 0.02];

    fn applied(edit: Adjustments, working: [f32; 3]) -> [f32; 3] {
        Presence::of(&edit).applied(working)
    }

    fn with_saturation(saturation: f32) -> Adjustments {
        Adjustments {
            saturation,
            ..Adjustments::default()
        }
    }

    fn with_vibrance(vibrance: f32) -> Adjustments {
        Adjustments {
            vibrance,
            ..Adjustments::default()
        }
    }

    fn spread(working: [f32; 3]) -> f32 {
        working.into_iter().fold(f32::MIN, f32::max) - working.into_iter().fold(f32::MAX, f32::min)
    }

    #[test]
    fn default_edit_leaves_colors_unchanged() {
        let unchanged = applied(Adjustments::default(), VIVID);

        assert!(
            unchanged
                .iter()
                .zip(VIVID)
                .all(|(left, right)| (left - right).abs() < 1e-6)
        );
    }

    #[test]
    fn saturation_at_its_lowest_gives_black_and_white() {
        let [red, green, blue] = applied(with_saturation(-100.0), VIVID);

        assert!((red - green).abs() < 1e-6 && (green - blue).abs() < 1e-6);
        assert!((green - luminance(VIVID)).abs() < 1e-6);
    }

    #[test]
    fn saturation_scales_every_color_alike_and_keeps_luminance() {
        for color in [MUTED, VIVID] {
            let saturated = applied(with_saturation(50.0), color);

            assert!((spread(saturated) / spread(color) - 1.5).abs() < 1e-4);
            assert!((luminance(saturated) - luminance(color)).abs() < 1e-5);
        }
    }

    #[test]
    fn vibrance_acts_on_muted_colors_and_holds_back_on_vivid_ones() {
        for vibrance in [-80.0, 80.0] {
            let muted_change = spread(applied(with_vibrance(vibrance), MUTED)) / spread(MUTED);
            let vivid_change = spread(applied(with_vibrance(vibrance), VIVID)) / spread(VIVID);

            assert!(
                (muted_change - 1.0).abs() > 5.0 * (vivid_change - 1.0).abs(),
                "vibrance {vibrance}: muted ×{muted_change}, vivid ×{vivid_change}"
            );
        }
    }

    #[test]
    fn grey_stays_grey() {
        let edit = Adjustments {
            vibrance: 100.0,
            saturation: 100.0,
            ..Adjustments::default()
        };

        let [red, green, blue] = applied(edit, [0.3; 3]);

        assert!(
            (red - 0.3).abs() < 1e-6 && (green - 0.3).abs() < 1e-6 && (blue - 0.3).abs() < 1e-6
        );
    }
}
