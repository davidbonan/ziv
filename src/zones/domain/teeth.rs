use std::ops::RangeInclusive;

/// The class of the model of face parts that is the inside of the mouth:
/// teeth, but also the tongue and the dark behind them.
pub const MOUTH_INSIDE_CLASS: usize = 10;

// A pixel is as bright as its brightest channel, and compared to the bright of its mouth.
const BRIGHT_OF_THE_MOUTH_PERCENTILE: f32 = 0.9;
// Teeth are among the brightest of a mouth: darker than this share of its bright is the dark behind them.
const DARK_TO_TEETH: RangeInclusive<f32> = 0.35..=0.6;
// Teeth are white to yellow, a tongue, lips and gums are red: how much of
// its red a pixel has no green for, from teeth to them.
const TEETH_TO_RED: RangeInclusive<f32> = 0.36..=0.5;
const COVERED: u8 = 127;

fn share_along(range: &RangeInclusive<f32>, value: f32) -> f32 {
    ((value - range.start()) / (range.end() - range.start())).clamp(0.0, 1.0)
}

fn brightness([red, green, blue, _]: [u8; 4]) -> f32 {
    f32::from(red.max(green).max(blue)) / 255.0
}

fn redness([red, green, ..]: [u8; 4]) -> f32 {
    match red {
        0 => 0.0,
        _ => f32::from(red.saturating_sub(green)) / f32::from(red),
    }
}

// How bright the bright of the mouth is; `None` when the mouth covers nothing.
fn bright_of(mouth: &[u8], pixels: &[[u8; 4]]) -> Option<f32> {
    let of_the_mouth = mouth
        .iter()
        .zip(pixels)
        .filter(|(covered, _)| **covered > COVERED);
    let mut brightnesses: Vec<f32> = of_the_mouth.map(|(_, pixel)| brightness(*pixel)).collect();
    brightnesses.sort_by(f32::total_cmp);
    let rank = (brightnesses.len() as f32 * BRIGHT_OF_THE_MOUTH_PERCENTILE) as usize;
    brightnesses
        .get(rank.min(brightnesses.len().checked_sub(1)?))
        .copied()
}

/// What is teeth of the inside of a mouth: `mouth` is its coverage, one value
/// a pixel of `rgba`, display-encoded. Teeth are what is bright and not red
/// of it.
pub fn teeth_coverage(mouth: &[u8], rgba: &[u8]) -> Vec<u8> {
    let (pixels, _) = rgba.as_chunks::<4>();
    let Some(bright) = bright_of(mouth, pixels).filter(|bright| *bright > 0.0) else {
        return vec![0; mouth.len()];
    };
    let teeth = mouth.iter().zip(pixels).map(|(covered, pixel)| {
        let is_bright = share_along(&DARK_TO_TEETH, brightness(*pixel) / bright);
        let is_not_red = 1.0 - share_along(&TEETH_TO_RED, redness(*pixel));
        (f32::from(*covered) * is_bright * is_not_red).round() as u8
    });
    teeth.collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOOTH: [u8; 4] = [225, 215, 195, 255];
    const YELLOWED_TOOTH_IN_THE_SHADE: [u8; 4] = [170, 145, 95, 255];
    const DARK_BEHIND: [u8; 4] = [45, 20, 20, 255];
    const TONGUE: [u8; 4] = [200, 90, 95, 255];

    fn teeth_of(mouth: &[u8], pixels: &[[u8; 4]]) -> Vec<u8> {
        teeth_coverage(mouth, pixels.as_flattened())
    }

    #[test]
    fn teeth_are_kept_and_the_dark_and_the_tongue_are_left() {
        let teeth = teeth_of(&[255; 4], &[TOOTH, DARK_BEHIND, TONGUE, TOOTH]);

        assert_eq!(teeth, [255, 0, 0, 255]);
    }

    #[test]
    fn yellowed_tooth_in_the_shade_of_the_mouth_is_still_a_tooth() {
        let teeth = teeth_of(&[255; 3], &[TOOTH, YELLOWED_TOOTH_IN_THE_SHADE, TONGUE]);

        assert_eq!(teeth, [255, 255, 0]);
    }

    #[test]
    fn what_is_outside_the_mouth_is_not_teeth_however_white() {
        let teeth = teeth_of(&[255, 0, 60], &[TOOTH, TOOTH, TOOTH]);

        assert_eq!(teeth, [255, 0, 60]);
    }

    #[test]
    fn mouth_showing_only_its_tongue_has_no_teeth() {
        let teeth = teeth_of(&[255; 3], &[TONGUE, TONGUE, DARK_BEHIND]);

        assert_eq!(teeth, [0; 3]);
    }

    #[test]
    fn mouth_that_is_nowhere_has_no_teeth() {
        assert_eq!(teeth_of(&[0; 2], &[TOOTH, TOOTH]), [0; 2]);
    }
}
