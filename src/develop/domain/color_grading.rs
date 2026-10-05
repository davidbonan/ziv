use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use crate::color::domain::working_space::display_luma;

pub const HUE_RANGE: RangeInclusive<f32> = 0.0..=360.0;
pub const GRADE_SATURATION_RANGE: RangeInclusive<f32> = 0.0..=100.0;
pub const BLENDING_RANGE: RangeInclusive<f32> = 0.0..=100.0;
pub const DEFAULT_BLENDING: f32 = 50.0;

const FULL_ADJUSTMENT: f32 = 100.0;
/// How far a full Saturation takes a tone toward the color of its wheel.
const TINT_REACH: f32 = 0.25;
/// How much a full Luminance brightens or darkens a tone, display white being 1.
const LUMINANCE_REACH: f32 = 0.25;
/// A full Balance moves the tone where shadows and highlights meet by this many
/// powers of two: from a quarter of the way to white to seven tenths.
const BALANCE_REACH: f32 = 1.0;
/// Where shadows end and where highlights begin, once Balance is applied.
const SHADOWS_EDGE: f32 = 1.0 / 3.0;
const HIGHLIGHTS_EDGE: f32 = 2.0 / 3.0;
/// Half the width of the passage between two zones, at Blending 0 and 100.
const NARROWEST_PASSAGE: f32 = 0.05;
const WIDEST_PASSAGE: f32 = 0.6;

/// The tones a grade acts on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TonalZone {
    Shadows,
    Midtones,
    Highlights,
    /// Every tone alike.
    Global,
}

impl TonalZone {
    pub const ALL: [Self; 4] = [
        Self::Shadows,
        Self::Midtones,
        Self::Highlights,
        Self::Global,
    ];
}

/// The tint and the luminance given to a tonal zone. The default changes nothing.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ZoneGrade {
    /// The angle on the color wheel, in degrees: red at 0, green at 120, blue at 240.
    pub hue: f32,
    /// The distance from the centre of the wheel, 0 … 100.
    pub saturation: f32,
    /// −100 … +100.
    pub luminance: f32,
}

impl ZoneGrade {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    fn changes_nothing(&self) -> bool {
        self.saturation == 0.0 && self.luminance == 0.0
    }

    /// What a tone fully in the zone is moved by, per display channel.
    fn offset(&self) -> [f32; 3] {
        let wheel = wheel_color(self.hue);
        let tint = self.saturation / FULL_ADJUSTMENT * TINT_REACH;
        let lift = self.luminance / FULL_ADJUSTMENT * LUMINANCE_REACH;
        let grey = display_luma(wheel);
        wheel.map(|channel| (channel - grey) * tint + lift)
    }
}

/// The color at an angle of the wheel, at its rim: display values, each 0 … 1.
pub fn wheel_color(hue: f32) -> [f32; 3] {
    let sixth = hue.rem_euclid(360.0) / 60.0;
    [0.0, 4.0, 2.0].map(|shift: f32| {
        let along = (sixth + shift).rem_euclid(6.0);
        ((along - 3.0).abs() - 1.0).clamp(0.0, 1.0)
    })
}

/// Shadows, Midtones, Highlights and Global, and how the first three share the tones.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorGrading {
    pub shadows: ZoneGrade,
    pub midtones: ZoneGrade,
    pub highlights: ZoneGrade,
    pub global: ZoneGrade,
    /// How much the zones overlap, 0 … 100.
    pub blending: f32,
    /// Where the tones pass from shadows to highlights, −100 … +100.
    pub balance: f32,
}

impl Default for ColorGrading {
    fn default() -> Self {
        Self {
            shadows: ZoneGrade::default(),
            midtones: ZoneGrade::default(),
            highlights: ZoneGrade::default(),
            global: ZoneGrade::default(),
            blending: DEFAULT_BLENDING,
            balance: 0.0,
        }
    }
}

/// Color grading as the quantities the math and the shader work with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradingFactors {
    /// What a tone fully in each zone is moved by: shadows, midtones, highlights, global.
    pub offsets: [[f32; 3]; 4],
    /// The power the luma of a tone is raised to before the zones share it.
    pub balance_exponent: f32,
    /// Half the width of the passage between two zones.
    pub passage: f32,
    pub shadows_edge: f32,
    pub highlights_edge: f32,
}

fn smooth_step(position: f32) -> f32 {
    let clamped = position.clamp(0.0, 1.0);
    clamped * clamped * (3.0 - 2.0 * clamped)
}

impl GradingFactors {
    // How much of a tone is past the passage centred on `edge`.
    fn past(&self, edge: f32, tone: f32) -> f32 {
        smooth_step((tone - (edge - self.passage)) / (2.0 * self.passage))
    }

    /// The share of shadows, midtones and highlights in a tone of this luma: they sum to 1.
    pub fn zone_shares(&self, luma: f32) -> [f32; 3] {
        let tone = luma.clamp(0.0, 1.0).powf(self.balance_exponent);
        let past_shadows = self.past(self.shadows_edge, tone);
        let in_highlights = self.past(self.highlights_edge, tone);
        [
            1.0 - past_shadows,
            past_shadows - in_highlights,
            in_highlights,
        ]
    }

    /// `encoded`: a display pixel, each channel 0 … 1.
    pub fn applied(&self, encoded: [f32; 3]) -> [f32; 3] {
        let [shadows, midtones, highlights] = self.zone_shares(display_luma(encoded));
        let shares = [shadows, midtones, highlights, 1.0];
        [0, 1, 2].map(|channel| {
            let moved: f32 = (0..shares.len())
                .map(|zone| shares[zone] * self.offsets[zone][channel])
                .sum();
            (encoded[channel] + moved).clamp(0.0, 1.0)
        })
    }
}

impl ColorGrading {
    pub fn of(&self, zone: TonalZone) -> &ZoneGrade {
        match zone {
            TonalZone::Shadows => &self.shadows,
            TonalZone::Midtones => &self.midtones,
            TonalZone::Highlights => &self.highlights,
            TonalZone::Global => &self.global,
        }
    }

    pub fn with(self, zone: TonalZone, grade: ZoneGrade) -> Self {
        match zone {
            TonalZone::Shadows => Self {
                shadows: grade,
                ..self
            },
            TonalZone::Midtones => Self {
                midtones: grade,
                ..self
            },
            TonalZone::Highlights => Self {
                highlights: grade,
                ..self
            },
            TonalZone::Global => Self {
                global: grade,
                ..self
            },
        }
    }

    /// With no tint and no luminance in any zone, Blending and Balance have nothing to act on.
    pub fn changes_nothing(&self) -> bool {
        TonalZone::ALL
            .iter()
            .all(|zone| self.of(*zone).changes_nothing())
    }

    pub fn factors(&self) -> GradingFactors {
        let blending = (self.blending / FULL_ADJUSTMENT).clamp(0.0, 1.0);
        GradingFactors {
            offsets: TonalZone::ALL.map(|zone| self.of(zone).offset()),
            balance_exponent: (-self.balance / FULL_ADJUSTMENT * BALANCE_REACH).exp2(),
            passage: NARROWEST_PASSAGE + (WIDEST_PASSAGE - NARROWEST_PASSAGE) * blending,
            shadows_edge: SHADOWS_EDGE,
            highlights_edge: HIGHLIGHTS_EDGE,
        }
    }

    /// `encoded`: a display pixel, each channel 0 … 1.
    pub fn applied(&self, encoded: [f32; 3]) -> [f32; 3] {
        if self.changes_nothing() {
            return encoded;
        }
        self.factors().applied(encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DARK: [f32; 3] = [0.08; 3];
    const MIDDLE: [f32; 3] = [0.5; 3];
    const BRIGHT: [f32; 3] = [0.92; 3];
    const TEAL: f32 = 180.0;

    fn tinted(zone: TonalZone, hue: f32) -> ColorGrading {
        let grade = ZoneGrade {
            hue,
            saturation: 100.0,
            luminance: 0.0,
        };
        ColorGrading::default().with(zone, grade)
    }

    fn lit(zone: TonalZone, luminance: f32) -> ColorGrading {
        let grade = ZoneGrade {
            luminance,
            ..ZoneGrade::default()
        };
        ColorGrading::default().with(zone, grade)
    }

    /// How far a pixel is from the grey of its luma: 0 for a grey.
    fn tint_of(encoded: [f32; 3]) -> f32 {
        let grey = display_luma(encoded);
        encoded
            .into_iter()
            .map(|channel| (channel - grey).abs())
            .fold(0.0, f32::max)
    }

    fn assert_close(actual: [f32; 3], expected: [f32; 3]) {
        let is_close = actual
            .iter()
            .zip(expected)
            .all(|(actual, expected)| (actual - expected).abs() < 1e-5);
        assert!(is_close, "{actual:?} is not {expected:?}");
    }

    #[test]
    fn the_wheel_goes_from_red_through_green_and_blue() {
        assert_eq!(wheel_color(0.0), [1.0, 0.0, 0.0]);
        assert_eq!(wheel_color(60.0), [1.0, 1.0, 0.0]);
        assert_eq!(wheel_color(120.0), [0.0, 1.0, 0.0]);
        assert_eq!(wheel_color(240.0), [0.0, 0.0, 1.0]);
        assert_eq!(wheel_color(360.0), wheel_color(0.0));
    }

    #[test]
    fn default_grading_changes_no_pixel() {
        let pixel = [0.1, 0.4, 0.9];

        assert_eq!(ColorGrading::default().applied(pixel), pixel);
    }

    #[test]
    fn hue_without_saturation_changes_nothing() {
        let grade = ZoneGrade {
            hue: 200.0,
            ..ZoneGrade::default()
        };
        let pixel = [0.1, 0.4, 0.9];

        for zone in TonalZone::ALL {
            let hue_alone = ColorGrading::default().with(zone, grade);
            assert_eq!(hue_alone.applied(pixel), pixel);
        }
    }

    #[test]
    fn blending_and_balance_alone_change_nothing() {
        let moved = ColorGrading {
            blending: 100.0,
            balance: -80.0,
            ..ColorGrading::default()
        };
        let pixel = [0.1, 0.4, 0.9];

        assert_eq!(moved.applied(pixel), pixel);
    }

    #[test]
    fn a_tint_moves_a_grey_toward_its_color_and_keeps_its_luma() {
        let teal = tinted(TonalZone::Global, TEAL).applied(MIDDLE);

        assert!(teal[0] < 0.5 && teal[1] > 0.5 && teal[2] > 0.5, "{teal:?}");
        assert!((display_luma(teal) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn each_zone_tints_its_tones_and_leaves_the_far_end_alone() {
        let shadows = tinted(TonalZone::Shadows, TEAL);
        let midtones = tinted(TonalZone::Midtones, TEAL);
        let highlights = tinted(TonalZone::Highlights, TEAL);

        assert!(tint_of(shadows.applied(DARK)) > 0.05);
        assert_close(shadows.applied(BRIGHT), BRIGHT);
        assert!(tint_of(highlights.applied(BRIGHT)) > 0.05);
        assert_close(highlights.applied(DARK), DARK);
        assert!(tint_of(midtones.applied(MIDDLE)) > 0.05);
        assert!(tint_of(midtones.applied(MIDDLE)) > 4.0 * tint_of(midtones.applied(DARK)));
        assert!(tint_of(midtones.applied(MIDDLE)) > 4.0 * tint_of(midtones.applied(BRIGHT)));
    }

    #[test]
    fn global_tints_every_tone_alike() {
        let global = tinted(TonalZone::Global, TEAL);
        let tints = [DARK, MIDDLE, [0.8; 3]].map(|tone| {
            let graded = global.applied(tone);
            graded[1] - tone[1]
        });

        assert!((tints[0] - tints[1]).abs() < 1e-5 && (tints[1] - tints[2]).abs() < 1e-5);
    }

    #[test]
    fn luminance_of_a_zone_brightens_or_darkens_its_tones_only() {
        let brighter_shadows = lit(TonalZone::Shadows, 100.0);
        let darker_highlights = lit(TonalZone::Highlights, -100.0);

        assert!(brighter_shadows.applied(DARK)[1] > DARK[1] + 0.15);
        assert_close(brighter_shadows.applied(BRIGHT), BRIGHT);
        assert!(darker_highlights.applied(BRIGHT)[1] < BRIGHT[1] - 0.15);
        assert_close(darker_highlights.applied(DARK), DARK);
    }

    #[test]
    fn the_three_zones_share_every_tone_entirely() {
        for blending in [0.0, 50.0, 100.0] {
            for balance in [-100.0, 0.0, 100.0] {
                let factors = ColorGrading {
                    blending,
                    balance,
                    ..ColorGrading::default()
                }
                .factors();
                for step in 0..=100 {
                    let shares = factors.zone_shares(step as f32 / 100.0);
                    let whole: f32 = shares.iter().sum();
                    assert!((whole - 1.0).abs() < 1e-5);
                    assert!(shares.iter().all(|share| *share >= -1e-6), "{shares:?}");
                }
            }
        }
    }

    #[test]
    fn raised_balance_counts_more_tones_as_highlights_and_lowered_as_shadows() {
        let highlights_of = |balance: f32| {
            let grading = ColorGrading {
                balance,
                ..tinted(TonalZone::Highlights, TEAL)
            };
            tint_of(grading.applied(MIDDLE))
        };
        let shadows_of = |balance: f32| {
            let grading = ColorGrading {
                balance,
                ..tinted(TonalZone::Shadows, TEAL)
            };
            tint_of(grading.applied(MIDDLE))
        };

        assert!(highlights_of(100.0) > highlights_of(0.0) + 0.02);
        assert!(highlights_of(0.0) > highlights_of(-100.0) + 0.02);
        assert!(shadows_of(-100.0) > shadows_of(0.0) + 0.02);
        assert!(shadows_of(0.0) > shadows_of(100.0) + 0.02);
    }

    #[test]
    fn blending_widens_how_far_a_zone_spreads_into_the_others() {
        let shadows_at_the_middle = |blending: f32| {
            let grading = ColorGrading {
                blending,
                ..tinted(TonalZone::Shadows, TEAL)
            };
            tint_of(grading.applied(MIDDLE))
        };

        assert!(shadows_at_the_middle(0.0) < 1e-5);
        assert!(shadows_at_the_middle(50.0) > 0.01);
        assert!(shadows_at_the_middle(100.0) > 1.5 * shadows_at_the_middle(50.0));
    }

    #[test]
    fn a_graded_pixel_stays_in_the_display_range() {
        let pushed = lit(TonalZone::Global, 100.0);

        assert_close(pushed.applied([0.9, 0.95, 1.0]), [1.0; 3]);
        assert_close(lit(TonalZone::Global, -100.0).applied(DARK), [0.0; 3]);
    }
}
