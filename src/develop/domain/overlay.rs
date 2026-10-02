/// The red veil showing what a mask covers, in encoded display values.
pub const OVERLAY_COLOUR: [f32; 3] = [1.0, 0.2, 0.15];
/// How much of the veil shows at full coverage.
pub const OVERLAY_OPACITY: f32 = 0.5;

/// An encoded display pixel under the veil of a mask covering it by `coverage`.
pub fn overlaid(display: [f32; 3], coverage: f32) -> [f32; 3] {
    let veil = coverage * OVERLAY_OPACITY;
    [0, 1, 2].map(|channel| display[channel] + (OVERLAY_COLOUR[channel] - display[channel]) * veil)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uncovered_pixel_is_left_alone() {
        assert_eq!(overlaid([0.2, 0.4, 0.6], 0.0), [0.2, 0.4, 0.6]);
    }

    #[test]
    fn fully_covered_pixel_is_half_veil() {
        assert_eq!(overlaid([0.0, 0.0, 0.0], 1.0), [0.5, 0.1, 0.075]);
    }
}
