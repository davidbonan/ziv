use std::mem::discriminant;

use crate::photo::domain::photo_kind::PhotoKind;

use super::edit::Edit;

/// An edit kept by Copy, with the kind of photo it came from.
#[derive(Debug, Clone, PartialEq)]
pub struct CopiedEdit {
    edit: Edit,
    source: PhotoKind,
}

impl CopiedEdit {
    pub fn of(edit: Edit, source: &PhotoKind) -> Self {
        Self {
            edit,
            source: *source,
        }
    }

    /// The edit a photo gets from Paste. White balance does not cross between
    /// a RAW and a standard image: their scales are not the same.
    pub fn pasted_onto(&self, current: &Edit, target: &PhotoKind) -> Edit {
        let mut pasted = self.edit.clone();
        if discriminant(&self.source) != discriminant(target) {
            pasted.adjustments.white_balance = current.adjustments.white_balance;
        }
        pasted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::domain::illuminant::Illuminant;
    use crate::develop::domain::adjustments::Adjustments;
    use crate::develop::domain::color_grading::{ColorGrading, TonalZone, ZoneGrade};
    use crate::develop::domain::color_mixer::{ColorMixer, ColorRange};
    use crate::develop::domain::linear_gradient::LinearGradient;
    use crate::develop::domain::mask::{Mask, MaskShape};
    use crate::develop::domain::tone_curve::{CurveChannel, ToneCurve, ToneCurves};
    use crate::develop::domain::white_balance::WhiteBalance;

    const RAW: PhotoKind = PhotoKind::Raw {
        as_shot: Illuminant {
            temperature: 5200.0,
            tint: 10.0,
        },
    };
    const OTHER_RAW: PhotoKind = PhotoKind::Raw {
        as_shot: Illuminant {
            temperature: 3100.0,
            tint: -4.0,
        },
    };

    fn kelvin_edit() -> Edit {
        Edit::from(Adjustments {
            white_balance: Some(WhiteBalance {
                temperature: 6400.0,
                tint: 20.0,
            }),
            exposure: 0.7,
            ..Adjustments::default()
        })
    }

    fn relative_edit() -> Edit {
        Edit::from(Adjustments {
            white_balance: Some(WhiteBalance {
                temperature: -15.0,
                tint: 0.0,
            }),
            contrast: 30.0,
            ..Adjustments::default()
        })
    }

    #[test]
    fn paste_between_photos_of_the_same_kind_replaces_the_whole_edit() {
        let copied = CopiedEdit::of(kelvin_edit(), &RAW);

        assert_eq!(
            copied.pasted_onto(&Edit::default(), &OTHER_RAW),
            kelvin_edit()
        );
    }

    #[test]
    fn paste_across_kinds_carries_the_curves_the_color_mixer_and_the_color_grading() {
        let mut color_mixer = ColorMixer::default();
        color_mixer.hue[ColorRange::Orange] = 20.0;
        let warm = ZoneGrade {
            hue: 40.0,
            saturation: 25.0,
            luminance: 0.0,
        };
        let lifted = ToneCurve::default().with_point_moved(0, [0.0, 0.1]);
        let toned = Edit {
            tone_curves: ToneCurves::default().with(CurveChannel::Blue, lifted),
            color_mixer,
            color_grading: ColorGrading::default().with(TonalZone::Highlights, warm),
            ..Edit::default()
        };
        let copied = CopiedEdit::of(toned.clone(), &RAW);

        let pasted = copied.pasted_onto(&relative_edit(), &PhotoKind::StandardImage);

        assert_eq!(pasted.tone_curves, toned.tone_curves);
        assert_eq!(pasted.color_mixer, toned.color_mixer);
        assert_eq!(pasted.color_grading, toned.color_grading);
    }

    #[test]
    fn paste_across_kinds_leaves_white_balance_as_it_was() {
        let copied = CopiedEdit::of(kelvin_edit(), &RAW);

        let pasted = copied.pasted_onto(&relative_edit(), &PhotoKind::StandardImage);

        let mut expected = kelvin_edit();
        expected.adjustments.white_balance = relative_edit().adjustments.white_balance;
        assert_eq!(pasted, expected);
    }

    #[test]
    fn paste_brings_the_masks_along() {
        let gradient = LinearGradient {
            full: [0.2, 0.2],
            none: [0.6, 0.4],
        };
        let masked = Edit {
            masks: vec![Mask::of(MaskShape::LinearGradient(gradient))],
            ..kelvin_edit()
        };
        let copied = CopiedEdit::of(masked.clone(), &RAW);

        let pasted = copied.pasted_onto(&relative_edit(), &PhotoKind::StandardImage);

        assert_eq!(pasted.masks, masked.masks);
    }
}
