use std::array;
use std::f32::consts::TAU;
use std::ops::{Index, IndexMut};

use serde::{Deserialize, Serialize};

use crate::color::domain::oklab::oklab;

pub const RANGE_COUNT: usize = 8;

const FULL_ADJUSTMENT: f32 = 100.0;
/// How far a full Hue turns a color, as a share of the way to the
/// neighbouring range. Under a third, hues keep their order even when two
/// neighbours are turned toward each other.
const HUE_REACH: f32 = 0.3;
/// How much a full Luminance scales the lightness of a color.
const LIGHTNESS_REACH: f32 = 0.25;
/// The chroma under which a color counts less and less as one of a range.
pub const CHROMA_FADE: f32 = 0.04;
const SAMPLE_LIGHTNESS: f32 = 0.7;
const SAMPLE_CHROMA: f32 = 0.12;

/// The pixels whose hue is near a named color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorRange {
    Red,
    Orange,
    Yellow,
    Green,
    Aqua,
    Blue,
    Purple,
    Magenta,
}

impl ColorRange {
    /// In the order of the hues.
    pub const ALL: [Self; RANGE_COUNT] = [
        Self::Red,
        Self::Orange,
        Self::Yellow,
        Self::Green,
        Self::Aqua,
        Self::Blue,
        Self::Purple,
        Self::Magenta,
    ];

    /// A plain color of the range, as a working-space pixel.
    pub fn sample(self) -> [f32; 3] {
        let (sine, cosine) = self.centre().to_radians().sin_cos();
        oklab().to_working([
            SAMPLE_LIGHTNESS,
            SAMPLE_CHROMA * cosine,
            SAMPLE_CHROMA * sine,
        ])
    }

    /// The Oklch hue of the color the range is named after, in degrees.
    pub fn centre(self) -> f32 {
        match self {
            Self::Red => 29.0,
            Self::Orange => 53.0,
            Self::Yellow => 110.0,
            Self::Green => 142.0,
            Self::Aqua => 195.0,
            Self::Blue => 264.0,
            Self::Purple => 294.0,
            Self::Magenta => 328.0,
        }
    }
}

/// What the color mixer adjusts of a color range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MixerAdjustment {
    #[default]
    Hue,
    Saturation,
    Luminance,
}

impl MixerAdjustment {
    pub const ALL: [Self; 3] = [Self::Hue, Self::Saturation, Self::Luminance];
}

/// One value of an adjustment for each color range, −100 … +100.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PerColorRange([f32; RANGE_COUNT]);

impl PerColorRange {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

impl Index<ColorRange> for PerColorRange {
    type Output = f32;

    fn index(&self, range: ColorRange) -> &f32 {
        &self.0[range as usize]
    }
}

impl IndexMut<ColorRange> for PerColorRange {
    fn index_mut(&mut self, range: ColorRange) -> &mut f32 {
        &mut self.0[range as usize]
    }
}

/// Hue, Saturation and Luminance of each color range. The default changes nothing.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorMixer {
    pub hue: PerColorRange,
    pub saturation: PerColorRange,
    pub luminance: PerColorRange,
}

/// A color range as the quantities the math and the shader work with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RangeFactors {
    /// The hue of the range, in radians.
    pub centre: f32,
    /// What the hue of its colors is turned by, in radians.
    pub hue_turn: f32,
    pub chroma_scale: f32,
    pub lightness_scale: f32,
}

impl RangeFactors {
    fn toward(&self, next: &Self, share: f32) -> Self {
        let between = |from: f32, to: f32| from + (to - from) * share;
        Self {
            centre: between(self.centre, next.centre),
            hue_turn: between(self.hue_turn, next.hue_turn),
            chroma_scale: between(self.chroma_scale, next.chroma_scale),
            lightness_scale: between(self.lightness_scale, next.lightness_scale),
        }
    }
}

fn smooth_step(position: f32) -> f32 {
    let clamped = position.clamp(0.0, 1.0);
    clamped * clamped * (3.0 - 2.0 * clamped)
}

impl ColorMixer {
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }

    pub fn of(&self, adjustment: MixerAdjustment) -> &PerColorRange {
        match adjustment {
            MixerAdjustment::Hue => &self.hue,
            MixerAdjustment::Saturation => &self.saturation,
            MixerAdjustment::Luminance => &self.luminance,
        }
    }

    pub fn with(self, adjustment: MixerAdjustment, values: PerColorRange) -> Self {
        match adjustment {
            MixerAdjustment::Hue => Self {
                hue: values,
                ..self
            },
            MixerAdjustment::Saturation => Self {
                saturation: values,
                ..self
            },
            MixerAdjustment::Luminance => Self {
                luminance: values,
                ..self
            },
        }
    }

    /// One per range, in the order of the hues.
    pub fn range_factors(&self) -> [RangeFactors; RANGE_COUNT] {
        let centres = ColorRange::ALL.map(|range| range.centre().to_radians());
        let gap = |from: usize, to: usize| (centres[to] - centres[from]).rem_euclid(TAU);
        array::from_fn(|index| {
            let range = ColorRange::ALL[index];
            let hue = self.hue[range] / FULL_ADJUSTMENT;
            let to_neighbour = match hue >= 0.0 {
                true => gap(index, (index + 1) % RANGE_COUNT),
                false => gap((index + RANGE_COUNT - 1) % RANGE_COUNT, index),
            };
            RangeFactors {
                centre: centres[index],
                hue_turn: hue * HUE_REACH * to_neighbour,
                chroma_scale: 1.0 + self.saturation[range] / FULL_ADJUSTMENT,
                lightness_scale: 1.0 + self.luminance[range] / FULL_ADJUSTMENT * LIGHTNESS_REACH,
            }
        })
    }

    /// What a color of this hue gets: a share of the two ranges it sits between.
    fn factors_at(&self, hue: f32) -> RangeFactors {
        let ranges = self.range_factors();
        let hue = match hue < ranges[0].centre {
            true => hue + TAU,
            false => hue,
        };
        let after = ranges
            .iter()
            .position(|range| hue < range.centre)
            .unwrap_or(0);
        let before = ranges[(after + RANGE_COUNT - 1) % RANGE_COUNT];
        let next = match after {
            0 => RangeFactors {
                centre: ranges[0].centre + TAU,
                ..ranges[0]
            },
            _ => ranges[after],
        };
        let share = smooth_step((hue - before.centre) / (next.centre - before.centre));
        before.toward(&next, share)
    }

    /// A working-space pixel with the colors of each range moved. A grey stays as it is.
    pub fn applied(&self, working: [f32; 3]) -> [f32; 3] {
        if self.is_default() {
            return working;
        }
        let [lightness, green_red, blue_yellow] = oklab().of_working(working);
        let chroma = green_red.hypot(blue_yellow);
        if chroma <= 0.0 {
            return working;
        }
        let hue = blue_yellow.atan2(green_red);
        let factors = self.factors_at(hue);
        let fade = smooth_step(chroma / CHROMA_FADE);
        let turned = hue + factors.hue_turn * fade;
        let scaled = chroma * factors.chroma_scale;
        let lightened = 1.0 + (factors.lightness_scale - 1.0) * fade;
        let mixed = [lightness, scaled * turned.cos(), scaled * turned.sin()];
        oklab().to_working(mixed.map(|axis| axis * lightened))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::domain::working_space::{SrgbInput, luminance};

    const NAMED_COLORS: [[f32; 3]; RANGE_COUNT] = [
        [1.0, 0.0, 0.0],
        [1.0, 0.5, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.5, 0.0, 1.0],
        [1.0, 0.0, 1.0],
    ];

    fn named(range: ColorRange) -> [f32; 3] {
        SrgbInput::default().to_working(NAMED_COLORS[range as usize])
    }

    fn hue_of(working: [f32; 3]) -> f32 {
        let [_, green_red, blue_yellow] = oklab().of_working(working);
        blue_yellow.atan2(green_red).to_degrees().rem_euclid(360.0)
    }

    fn chroma_of(working: [f32; 3]) -> f32 {
        let [_, green_red, blue_yellow] = oklab().of_working(working);
        green_red.hypot(blue_yellow)
    }

    /// A color of this Oklch hue, of middling lightness, as a working pixel.
    fn of_hue(degrees: f32) -> [f32; 3] {
        let (sine, cosine) = degrees.to_radians().sin_cos();
        oklab().to_working([0.7, 0.1 * cosine, 0.1 * sine])
    }

    fn with(adjust: impl FnOnce(&mut ColorMixer)) -> ColorMixer {
        let mut mixer = ColorMixer::default();
        adjust(&mut mixer);
        mixer
    }

    fn opposite(range: ColorRange) -> ColorRange {
        ColorRange::ALL[(range as usize + RANGE_COUNT / 2) % RANGE_COUNT]
    }

    fn assert_unchanged(mixer: &ColorMixer, working: [f32; 3]) {
        let mixed = mixer.applied(working);
        let is_unchanged = mixed
            .iter()
            .zip(working)
            .all(|(mixed, working)| (mixed - working).abs() < 1e-4);
        assert!(is_unchanged, "{working:?} became {mixed:?}");
    }

    /// The signed way from one hue to another, in degrees, −180 … 180.
    fn turn(from: f32, to: f32) -> f32 {
        (to - from + 180.0).rem_euclid(360.0) - 180.0
    }

    #[test]
    fn each_range_is_centred_on_the_color_it_is_named_after() {
        for range in ColorRange::ALL {
            let off_centre = turn(range.centre(), hue_of(named(range)));

            assert!(off_centre.abs() < 1.0, "{range:?}: {off_centre}° off");
        }
    }

    #[test]
    fn default_mixer_changes_no_pixel() {
        let pixel = [0.1, 0.4, 0.9];

        assert_eq!(ColorMixer::default().applied(pixel), pixel);
    }

    #[test]
    fn a_grey_is_never_changed() {
        let mut everything = ColorMixer::default();
        for range in ColorRange::ALL {
            everything.hue[range] = 100.0;
            everything.saturation[range] = -100.0;
            everything.luminance[range] = 100.0;
        }

        for light in [0.0, 0.02, 0.18, 1.0, 3.0] {
            assert_unchanged(&everything, [light; 3]);
        }
    }

    #[test]
    fn an_adjustment_of_a_range_leaves_the_opposite_hue_alone() {
        for range in ColorRange::ALL {
            let far = named(opposite(range));

            assert_unchanged(&with(|mixer| mixer.hue[range] = 100.0), far);
            assert_unchanged(&with(|mixer| mixer.saturation[range] = -100.0), far);
            assert_unchanged(&with(|mixer| mixer.luminance[range] = 100.0), far);
        }
    }

    #[test]
    fn raised_hue_turns_a_color_toward_the_next_range_and_lowered_toward_the_one_before() {
        for (index, range) in ColorRange::ALL.into_iter().enumerate() {
            let centre = range.centre();
            let next = ColorRange::ALL[(index + 1) % RANGE_COUNT].centre();
            let previous = ColorRange::ALL[(index + RANGE_COUNT - 1) % RANGE_COUNT].centre();
            let raised = with(|mixer| mixer.hue[range] = 100.0).applied(of_hue(centre));
            let lowered = with(|mixer| mixer.hue[range] = -100.0).applied(of_hue(centre));

            let up = turn(centre, next) * HUE_REACH;
            let down = turn(centre, previous) * HUE_REACH;
            assert!((turn(centre, hue_of(raised)) - up).abs() < 0.5, "{range:?}");
            assert!(
                (turn(centre, hue_of(lowered)) - down).abs() < 0.5,
                "{range:?}"
            );
        }
    }

    #[test]
    fn hue_keeps_the_light_and_the_intensity_of_a_color() {
        let red = of_hue(ColorRange::Red.centre());
        let turned = with(|mixer| mixer.hue[ColorRange::Red] = 100.0).applied(red);

        assert!((chroma_of(turned) - chroma_of(red)).abs() < 1e-4);
        assert!((oklab().of_working(turned)[0] - 0.7).abs() < 1e-4);
    }

    #[test]
    fn saturation_at_its_lowest_makes_the_colors_of_its_range_grey() {
        for range in ColorRange::ALL {
            let grey = with(|mixer| mixer.saturation[range] = -100.0).applied(named(range));

            assert!(chroma_of(grey) < 1e-3, "{range:?}: {grey:?}");
        }
    }

    #[test]
    fn raised_saturation_makes_the_colors_of_its_range_more_intense() {
        let green = of_hue(ColorRange::Green.centre());
        let vivid = with(|mixer| mixer.saturation[ColorRange::Green] = 50.0).applied(green);

        assert!((chroma_of(vivid) / chroma_of(green) - 1.5).abs() < 1e-3);
    }

    #[test]
    fn luminance_brightens_or_darkens_the_colors_of_its_range_and_keeps_their_hue() {
        let blue = of_hue(ColorRange::Blue.centre());
        let brighter = with(|mixer| mixer.luminance[ColorRange::Blue] = 100.0).applied(blue);
        let darker = with(|mixer| mixer.luminance[ColorRange::Blue] = -100.0).applied(blue);

        assert!(luminance(brighter) > 1.5 * luminance(blue));
        assert!(luminance(darker) < 0.6 * luminance(blue));
        assert!(turn(hue_of(blue), hue_of(brighter)).abs() < 0.1);
        assert!(turn(hue_of(blue), hue_of(darker)).abs() < 0.1);
    }

    #[test]
    fn a_color_between_two_ranges_gets_a_share_of_each() {
        let between = (ColorRange::Green.centre() + ColorRange::Aqua.centre()) / 2.0;
        let muted = with(|mixer| mixer.saturation[ColorRange::Green] = -100.0);

        let share = chroma_of(muted.applied(of_hue(between))) / chroma_of(of_hue(between));

        assert!((share - 0.5).abs() < 0.01, "kept {share} of its chroma");
    }

    /// Every range pushed to an end of its sliders, neighbours to opposite ends.
    fn pushed_apart(adjust: impl Fn(&mut ColorMixer, ColorRange, f32)) -> ColorMixer {
        let mut mixer = ColorMixer::default();
        for (index, range) in ColorRange::ALL.into_iter().enumerate() {
            let end = if index % 2 == 0 { 100.0 } else { -100.0 };
            adjust(&mut mixer, range, end);
        }
        mixer
    }

    fn gradient_of_hues(mixer: &ColorMixer) -> Vec<[f32; 3]> {
        const STEPS_A_DEGREE: usize = 10;
        (0..=360 * STEPS_A_DEGREE)
            .map(|step| mixer.applied(of_hue(step as f32 / STEPS_A_DEGREE as f32)))
            .collect()
    }

    #[test]
    fn a_gradient_of_hues_stays_smooth_whatever_the_sliders() {
        const LARGEST_STEP: f32 = 0.004;
        let wild = pushed_apart(|mixer, range, end| {
            mixer.hue[range] = end;
            mixer.saturation[range] = end;
            mixer.luminance[range] = -end;
        });

        for pair in gradient_of_hues(&wild).windows(2) {
            let [one, other] = [pair[0], pair[1]].map(|working| oklab().of_working(working));
            let step = (0..3)
                .map(|axis| (one[axis] - other[axis]).abs())
                .fold(0.0, f32::max);
            assert!(step < LARGEST_STEP, "a step of {step} between {pair:?}");
        }
    }

    #[test]
    fn hues_keep_their_order_when_neighbours_are_turned_toward_each_other() {
        let turned = pushed_apart(|mixer, range, end| mixer.hue[range] = end);

        for pair in gradient_of_hues(&turned).windows(2) {
            assert!(turn(hue_of(pair[0]), hue_of(pair[1])) > 0.0, "{pair:?}");
        }
    }

    #[test]
    fn a_faint_color_is_moved_less_than_a_vivid_one() {
        let brighter = with(|mixer| mixer.luminance[ColorRange::Red] = 100.0);
        let (sine, cosine) = ColorRange::Red.centre().to_radians().sin_cos();
        let faint = oklab().to_working([0.7, 0.004 * cosine, 0.004 * sine]);

        let gain = luminance(brighter.applied(faint)) / luminance(faint);

        assert!(gain > 1.0 && gain < 1.1, "gained ×{gain}");
    }
}
