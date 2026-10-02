use crate::color::domain::working_space::luminance;

use super::adjustments::Adjustments;

const FULL_ADJUSTMENT: f32 = 100.0;
const CONTRAST_STOPS: f32 = 0.5;
const HIGHLIGHTS_STOPS: f32 = 1.5;
pub const HIGHLIGHTS_WIDTH_STOPS: f32 = 3.5;
const SHADOWS_STOPS: f32 = 2.0;
pub const SHADOWS_WIDTH_STOPS: f32 = 5.0;
const WHITES_STRETCH: f32 = 0.3;
const BLACKS_OFFSET: f32 = 0.03;
/// Luminance from which Blacks leaves the photo alone.
pub const BLACKS_REACH: f32 = 0.36;
pub const DARKEST_LUMINANCE: f32 = 1e-6;

fn smoothstep(position: f32) -> f32 {
    let position = position.clamp(0.0, 1.0);
    position * position * (3.0 - 2.0 * position)
}

/// The tone adjustments of an edit, as the quantities the math works with.
/// Luminance is handled in stops above the photo's middle grey.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tone {
    pub middle_grey: f32,
    pub contrast_slope: f32,
    pub highlights_shift: f32,
    pub shadows_shift: f32,
    pub whites_stretch: f32,
    pub blacks_offset: f32,
}

impl Tone {
    pub fn of(edit: &Adjustments, middle_grey: f32) -> Self {
        Self {
            middle_grey,
            contrast_slope: (edit.contrast / FULL_ADJUSTMENT * CONTRAST_STOPS).exp2(),
            highlights_shift: edit.highlights / FULL_ADJUSTMENT * HIGHLIGHTS_STOPS,
            shadows_shift: edit.shadows / FULL_ADJUSTMENT * SHADOWS_STOPS,
            whites_stretch: edit.whites / FULL_ADJUSTMENT * WHITES_STRETCH,
            blacks_offset: -edit.blacks / FULL_ADJUSTMENT * BLACKS_OFFSET,
        }
    }

    /// How far the white of the untouched photo sits above its middle grey.
    pub fn white_stops(&self) -> f32 {
        (1.0 / self.middle_grey).log2()
    }

    // Steeper through middle grey, eased back so that display white stays put.
    fn contrasted(&self, stops: f32) -> f32 {
        let slope = self.contrast_slope;
        let white = self.white_stops();
        if stops <= 0.0 {
            return slope * stops;
        }
        if stops < white {
            return slope * stops + (1.0 - slope) / white * stops * stops;
        }
        white + (2.0 - slope) * (stops - white)
    }

    fn with_highlights_and_shadows(&self, stops: f32) -> f32 {
        stops
            + self.highlights_shift * smoothstep(stops / HIGHLIGHTS_WIDTH_STOPS)
            + self.shadows_shift * smoothstep(-stops / SHADOWS_WIDTH_STOPS)
    }

    fn with_whites(&self, stops: f32) -> f32 {
        let above_grey = stops.max(0.0);
        stops + self.whites_stretch * above_grey * above_grey / (above_grey + 1.0)
    }

    fn black_lift(&self, luminance: f32) -> f32 {
        let nearness_to_black = (1.0 - luminance / BLACKS_REACH).max(0.0);
        -self.blacks_offset * nearness_to_black * nearness_to_black
    }

    /// Luminance after every tone adjustment but Blacks.
    pub fn scaled_luminance(&self, luminance: f32) -> f32 {
        let stops = (luminance / self.middle_grey).log2();
        let toned = self.with_whites(self.with_highlights_and_shadows(self.contrasted(stops)));
        self.middle_grey * toned.exp2()
    }

    /// Scales the pixel to its new luminance, hue kept, then moves the black point.
    pub fn applied(&self, working: [f32; 3]) -> [f32; 3] {
        let luminance = luminance(working).max(DARKEST_LUMINANCE);
        let scaled = self.scaled_luminance(luminance);
        let lift = self.black_lift(scaled);
        working.map(|channel| channel * (scaled / luminance) + lift)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::domain::working_space::MIDDLE_GREY;

    const DISPLAY_WHITE: f32 = 1.0;
    const DEEP_SHADOW: f32 = 0.01;

    fn toned(edit: Adjustments, luminance: f32) -> f32 {
        Tone::of(&edit, MIDDLE_GREY).applied([luminance; 3])[0]
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= expected.abs() * 1e-4 + 1e-6,
            "{actual} is not {expected}"
        );
    }

    fn extreme_edits() -> Vec<Adjustments> {
        let ends = [-100.0, 100.0];
        let mut edits = Vec::new();
        for contrast in ends {
            for highlights in ends {
                for shadows in ends {
                    for whites in ends {
                        for blacks in ends {
                            edits.push(Adjustments {
                                contrast,
                                highlights,
                                shadows,
                                whites,
                                blacks,
                                ..Adjustments::default()
                            });
                        }
                    }
                }
            }
        }
        edits
    }

    #[test]
    fn default_edit_leaves_every_luminance_unchanged() {
        for luminance in [0.0, DEEP_SHADOW, MIDDLE_GREY, DISPLAY_WHITE, 4.0] {
            assert_close(toned(Adjustments::default(), luminance), luminance);
        }
    }

    #[test]
    fn contrast_keeps_middle_grey_and_display_white() {
        for contrast in [-100.0, 100.0] {
            let edit = Adjustments {
                contrast,
                ..Adjustments::default()
            };

            assert_close(toned(edit, MIDDLE_GREY), MIDDLE_GREY);
            assert_close(toned(edit, DISPLAY_WHITE), DISPLAY_WHITE);
        }
    }

    #[test]
    fn raised_contrast_spreads_tones_away_from_middle_grey() {
        let edit = Adjustments {
            contrast: 100.0,
            ..Adjustments::default()
        };

        assert!(toned(edit, 0.05) < 0.05);
        assert!(toned(edit, 0.5) > 0.5);
    }

    #[test]
    fn highlights_move_the_bright_part_and_leave_the_dark_part_alone() {
        let lowered = Adjustments {
            highlights: -100.0,
            ..Adjustments::default()
        };

        assert!(toned(lowered, DISPLAY_WHITE) < 0.5);
        assert_close(toned(lowered, MIDDLE_GREY), MIDDLE_GREY);
        assert_close(toned(lowered, DEEP_SHADOW), DEEP_SHADOW);
    }

    #[test]
    fn lowered_highlights_bring_back_light_above_display_white() {
        let lowered = Adjustments {
            highlights: -100.0,
            ..Adjustments::default()
        };

        assert!(toned(lowered, 2.0) < DISPLAY_WHITE);
    }

    #[test]
    fn shadows_move_the_dark_part_and_leave_the_bright_part_alone() {
        let raised = Adjustments {
            shadows: 100.0,
            ..Adjustments::default()
        };

        assert!(toned(raised, DEEP_SHADOW) > 2.0 * DEEP_SHADOW);
        assert_close(toned(raised, MIDDLE_GREY), MIDDLE_GREY);
        assert_close(toned(raised, DISPLAY_WHITE), DISPLAY_WHITE);
    }

    #[test]
    fn whites_move_what_reaches_display_white_and_leave_the_dark_part_alone() {
        let raised = Adjustments {
            whites: 100.0,
            ..Adjustments::default()
        };
        let lowered = Adjustments {
            whites: -100.0,
            ..Adjustments::default()
        };

        assert!(toned(raised, 0.8) > DISPLAY_WHITE);
        assert!(toned(lowered, DISPLAY_WHITE) < DISPLAY_WHITE);
        assert_close(toned(raised, DEEP_SHADOW), DEEP_SHADOW);
    }

    #[test]
    fn blacks_move_what_reaches_pure_black_and_leave_the_bright_part_alone() {
        let lowered = Adjustments {
            blacks: -100.0,
            ..Adjustments::default()
        };
        let raised = Adjustments {
            blacks: 100.0,
            ..Adjustments::default()
        };

        assert!(toned(lowered, DEEP_SHADOW) < 0.0);
        assert!(toned(raised, 0.0) > 0.0);
        assert_close(toned(lowered, DISPLAY_WHITE), DISPLAY_WHITE);
    }

    #[test]
    fn brighter_light_never_renders_darker_whatever_the_edit() {
        for edit in extreme_edits() {
            let steps: Vec<f32> = (-80..=40)
                .map(|tenth_of_stop| {
                    toned(edit, MIDDLE_GREY * (tenth_of_stop as f32 / 10.0).exp2())
                })
                .collect();

            assert!(
                steps.windows(2).all(|pair| pair[0] < pair[1]),
                "{edit:?} folds tones"
            );
        }
    }

    #[test]
    fn tone_keeps_the_hue_of_a_color() {
        let edit = Adjustments {
            contrast: 60.0,
            highlights: -40.0,
            ..Adjustments::default()
        };
        let [red, green, blue] = Tone::of(&edit, MIDDLE_GREY).applied([0.6, 0.3, 0.1]);

        assert_close(red / green, 2.0);
        assert_close(green / blue, 3.0);
    }

    #[test]
    fn contrast_turns_around_the_middle_grey_of_the_photo() {
        let edit = Adjustments {
            contrast: 100.0,
            ..Adjustments::default()
        };
        let darker_grey = 0.08;

        let toned = Tone::of(&edit, darker_grey).applied([darker_grey; 3])[0];

        assert!((toned - darker_grey).abs() < 1e-6);
    }
}
