// A normal noise of deviation 1 has this median absolute value.
const NORMAL_MEDIAN: f32 = 0.6745;

/// The standard deviation of the noise of a plane of `width` values a row.
/// Read in the finest diagonal detail of each block of four values, where a
/// picture holds little else than its noise; the median leaves edges out.
pub fn noise_level(plane: &[f32], width: usize) -> f32 {
    let rows = plane.chunks_exact(width);
    let mut details: Vec<f32> = rows
        .clone()
        .step_by(2)
        .zip(rows.skip(1).step_by(2))
        .flat_map(|(above, below)| {
            let blocks = above
                .as_chunks::<2>()
                .0
                .iter()
                .zip(below.as_chunks::<2>().0);
            blocks.map(|([a, b], [c, d])| ((a - b - c + d) / 2.0).abs())
        })
        .collect();
    if details.is_empty() {
        return 0.0;
    }
    let middle = details.len() / 2;
    let (_, median, _) = details.select_nth_unstable_by(middle, f32::total_cmp);
    *median / NORMAL_MEDIAN
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIDE: usize = 128;

    /// About normal, of deviation 1, the same at each run.
    fn noise() -> impl FnMut() -> f32 {
        let mut state = 0x2545_f491_u32;
        move || {
            let uniforms = [(); 12].map(|()| {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (state >> 8) as f32 / (1 << 24) as f32
            });
            uniforms.iter().sum::<f32>() - 6.0
        }
    }

    #[test]
    fn noise_is_measured_over_a_picture_with_an_edge() {
        let mut noise = noise();
        let noisy_edge: Vec<f32> = (0..SIDE * SIDE)
            .map(|index| {
                let picture = if index % SIDE < SIDE / 3 { 0.2 } else { 0.7 };
                picture + 0.05 * noise()
            })
            .collect();

        let level = noise_level(&noisy_edge, SIDE);

        assert!((level - 0.05).abs() < 0.005, "{level}");
    }

    #[test]
    fn clean_picture_has_no_noise() {
        let ramp: Vec<f32> = (0..SIDE * SIDE)
            .map(|index| (index % SIDE + index / SIDE) as f32 / (2 * SIDE) as f32)
            .collect();

        assert!(noise_level(&ramp, SIDE) < 1e-6);
    }
}
