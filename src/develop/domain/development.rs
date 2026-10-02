use crate::color::domain::working_space::DisplayTransform;

use crate::photo::domain::photo_kind::PhotoKind;

use super::adjustments::Adjustments;
use super::base_rendering::BaseRendering;
use super::edit::Edit;
use super::mask::{Mask, PhotoPoint};
use super::presence::Presence;
use super::tone::Tone;

const UNCHANGED: [[f32; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

/// Adjustments as the quantities the math and the shader work with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdjustmentFactors {
    /// A working-space transform.
    pub white_balance: [[f32; 3]; 3],
    pub exposure_gain: f32,
    pub tone: Tone,
    pub presence: Presence,
}

impl AdjustmentFactors {
    pub fn applied(&self, working: [f32; 3]) -> [f32; 3] {
        let exposed = self.white_balance.map(|row| {
            (row[0] * working[0] + row[1] * working[1] + row[2] * working[2]) * self.exposure_gain
        });
        self.presence.applied(self.tone.applied(exposed))
    }
}

/// Everything that turns a photo's working pixels into what is shown.
#[derive(Debug, Clone, PartialEq)]
pub struct Development {
    pub kind: PhotoKind,
    pub edit: Edit,
}

impl Default for Development {
    fn default() -> Self {
        Self {
            kind: PhotoKind::StandardImage,
            edit: Edit::default(),
        }
    }
}

impl Development {
    pub fn base_rendering(&self) -> BaseRendering {
        BaseRendering::of(&self.kind)
    }

    // `white_balance_scale`: the kind of photo that says what Temp and Tint mean.
    fn factors(
        &self,
        adjustments: &Adjustments,
        white_balance_scale: &PhotoKind,
    ) -> AdjustmentFactors {
        AdjustmentFactors {
            white_balance: adjustments
                .white_balance
                .map_or(UNCHANGED, |balance| balance.adaptation(white_balance_scale)),
            exposure_gain: adjustments.exposure_gain(),
            tone: Tone::of(adjustments, self.base_rendering().middle_grey()),
            presence: Presence::of(adjustments),
        }
    }

    pub fn photo_factors(&self) -> AdjustmentFactors {
        self.factors(&self.edit.adjustments, &self.kind)
    }

    /// Temp and Tint of a mask are relative on every photo.
    pub fn mask_factors(&self, mask: &Mask) -> AdjustmentFactors {
        self.factors(&mask.adjustments, &PhotoKind::StandardImage)
    }

    /// The pixel the adjustments start from: the original, with as much of
    /// its enhancement as the edit asks for.
    pub fn enhanced(&self, original: [f32; 3], enhancement: [f32; 3]) -> [f32; 3] {
        let share = self.edit.enhancement_share();
        [0, 1, 2]
            .map(|channel| original[channel] + (enhancement[channel] - original[channel]) * share)
    }

    /// The edited value of the working-space pixel at `point`, base rendering
    /// not applied yet. Each visible mask blends its adjustments by its coverage.
    pub fn edited(&self, working: [f32; 3], point: PhotoPoint) -> [f32; 3] {
        let globally = self.photo_factors().applied(working);
        self.edit.visible_masks().fold(globally, |colour, mask| {
            let coverage = mask.coverage(point);
            let locally = self.mask_factors(mask).applied(colour);
            [0, 1, 2]
                .map(|channel| colour[channel] + (locally[channel] - colour[channel]) * coverage)
        })
    }

    pub fn to_display(
        &self,
        working: [f32; 3],
        point: PhotoPoint,
        transform: &DisplayTransform,
    ) -> [f32; 3] {
        let display_linear = transform.to_display_linear(self.edited(working, point));
        transform.encoded(self.base_rendering().rendered(display_linear))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::domain::illuminant::Illuminant;
    use crate::develop::domain::linear_gradient::LinearGradient;
    use crate::develop::domain::mask::MaskShape;
    use crate::develop::domain::white_balance::WhiteBalance;

    const PIXEL: [f32; 3] = [0.1, 0.4, 0.9];

    const SOMEWHERE: PhotoPoint = [0.5, 0.5];

    fn edited(edit: impl Into<Edit>) -> [f32; 3] {
        let development = Development {
            edit: edit.into(),
            ..Development::default()
        };
        development.edited(PIXEL, SOMEWHERE)
    }

    fn one_stop_brighter(shape: LinearGradient) -> Mask {
        Mask {
            adjustments: Adjustments {
                exposure: 1.0,
                ..Adjustments::default()
            },
            ..Mask::of(MaskShape::LinearGradient(shape))
        }
    }

    const COVERING: LinearGradient = LinearGradient {
        full: [0.8, 0.5],
        none: [0.9, 0.5],
    };
    const LEAVING: LinearGradient = LinearGradient {
        full: [0.1, 0.5],
        none: [0.2, 0.5],
    };
    const HALF_COVERING: LinearGradient = LinearGradient {
        full: [0.0, 0.5],
        none: [1.0, 0.5],
    };

    fn masked(masks: Vec<Mask>) -> Edit {
        Edit {
            masks,
            ..Edit::default()
        }
    }

    fn assert_close(actual: [f32; 3], expected: [f32; 3]) {
        let is_close = actual
            .iter()
            .zip(expected)
            .all(|(actual, expected)| (actual - expected).abs() < 1e-5);
        assert!(is_close, "{actual:?} is not {expected:?}");
    }

    #[test]
    fn intensity_goes_from_the_original_to_its_enhancement() {
        let cleaned = [0.2, 0.2, 0.5];
        let enhanced_at = |enhancement_intensity| {
            let edit = Edit {
                enhancement_intensity,
                ..Edit::default()
            };
            Development {
                edit,
                ..Development::default()
            }
            .enhanced(PIXEL, cleaned)
        };

        assert_close(enhanced_at(0.0), PIXEL);
        assert_close(enhanced_at(50.0), [0.15, 0.3, 0.7]);
        assert_close(enhanced_at(100.0), cleaned);
    }

    #[test]
    fn default_edit_leaves_the_pixel_unchanged() {
        assert_close(edited(Edit::default()), PIXEL);
    }

    #[test]
    fn one_stop_of_exposure_doubles_or_halves_the_light() {
        let brighter = Adjustments {
            exposure: 1.0,
            ..Adjustments::default()
        };
        let darker = Adjustments {
            exposure: -1.0,
            ..Adjustments::default()
        };

        assert_close(edited(brighter), [0.2, 0.8, 1.8]);
        assert_close(edited(darker), [0.05, 0.2, 0.45]);
    }

    #[test]
    fn mask_applies_its_adjustments_where_it_covers_and_nowhere_else() {
        assert_close(
            edited(masked(vec![one_stop_brighter(COVERING)])),
            [0.2, 0.8, 1.8],
        );
        assert_close(edited(masked(vec![one_stop_brighter(LEAVING)])), PIXEL);
    }

    #[test]
    fn half_covered_pixel_gets_half_of_the_change() {
        assert_close(
            edited(masked(vec![one_stop_brighter(HALF_COVERING)])),
            [0.15, 0.6, 1.35],
        );
    }

    #[test]
    fn mask_without_adjustment_changes_nothing() {
        let untouched = Mask::of(MaskShape::LinearGradient(COVERING));

        assert_close(edited(masked(vec![untouched])), PIXEL);
    }

    #[test]
    fn overlapping_masks_add_up() {
        let twice = vec![one_stop_brighter(COVERING), one_stop_brighter(COVERING)];

        assert_close(edited(masked(twice)), [0.4, 1.6, 3.6]);
    }

    #[test]
    fn hidden_mask_changes_nothing() {
        let hidden = Mask {
            is_hidden: true,
            ..one_stop_brighter(COVERING)
        };

        assert_close(edited(masked(vec![hidden])), PIXEL);
    }

    #[test]
    fn mask_white_balance_is_relative_on_a_raw_too() {
        let warmer = Adjustments {
            white_balance: Some(WhiteBalance {
                temperature: 40.0,
                tint: 0.0,
            }),
            ..Adjustments::default()
        };
        let mask = Mask {
            adjustments: warmer,
            ..Mask::of(MaskShape::LinearGradient(COVERING))
        };
        let on = |kind: PhotoKind| {
            let development = Development {
                kind,
                edit: masked(vec![mask.clone()]),
            };
            development.edited(PIXEL, SOMEWHERE)
        };
        let raw = PhotoKind::Raw {
            as_shot: Illuminant {
                temperature: 5200.0,
                tint: 10.0,
            },
        };

        assert_close(on(raw), on(PhotoKind::StandardImage));
        assert!(on(raw)[0] > PIXEL[0] && on(raw)[2] < PIXEL[2]);
    }
}
