const AMOUNT: f32 = 0.4;
// One pixel of standard deviation.
const BLUR_WEIGHTS: [f32; 5] = [1.0 / 16.0, 4.0 / 16.0, 6.0 / 16.0, 4.0 / 16.0, 1.0 / 16.0];
const BLUR_REACH: usize = BLUR_WEIGHTS.len() / 2;

fn blurred_along(
    values: &[f32],
    [length, step]: [usize; 2],
    start: usize,
) -> impl Iterator<Item = f32> {
    (0..length).map(move |position| {
        let weighted = BLUR_WEIGHTS.iter().enumerate().map(|(tap, weight)| {
            let read = (position + tap).saturating_sub(BLUR_REACH).min(length - 1);
            weight * values[start + read * step]
        });
        weighted.sum()
    })
}

fn blurred(plane: &[f32], side: usize) -> Vec<f32> {
    let along_rows: Vec<f32> = (0..side)
        .flat_map(|row| blurred_along(plane, [side, 1], row * side))
        .collect();
    let mut blurred = vec![0.0; plane.len()];
    for column in 0..side {
        for (row, value) in blurred_along(&along_rows, [side, side], column).enumerate() {
            blurred[row * side + column] = value;
        }
    }
    blurred
}

/// A square plane with its detail strengthened.
pub fn sharpened(plane: &[f32], side: usize) -> Vec<f32> {
    let blurred = blurred(plane, side);
    let both = plane.iter().zip(blurred);
    both.map(|(value, blurred)| value + AMOUNT * (value - blurred))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIDE: usize = 8;

    #[test]
    fn flat_plane_is_left_as_it_is() {
        let flat = vec![0.4; SIDE * SIDE];

        let sharpened = sharpened(&flat, SIDE);

        let furthest = sharpened
            .iter()
            .map(|value| (value - 0.4).abs())
            .fold(0.0, f32::max);
        assert!(furthest < 1e-6, "{furthest}");
    }

    #[test]
    fn edge_is_made_steeper_on_both_sides_and_both_directions() {
        let dark_then_light = |along: usize| if along < SIDE / 2 { 0.2 } else { 0.6 };
        let vertical_edge: Vec<f32> = (0..SIDE * SIDE)
            .map(|index| dark_then_light(index % SIDE))
            .collect();
        let horizontal_edge: Vec<f32> = (0..SIDE * SIDE)
            .map(|index| dark_then_light(index / SIDE))
            .collect();

        let across_vertical = sharpened(&vertical_edge, SIDE);
        let across_horizontal = sharpened(&horizontal_edge, SIDE);

        let middle = SIDE / 2;
        let [dark, light] =
            [middle - 1, middle].map(|column| across_vertical[middle * SIDE + column]);
        assert!(dark < 0.2 && light > 0.6, "{dark} | {light}");
        let [above, below] = [middle - 1, middle].map(|row| across_horizontal[row * SIDE + middle]);
        assert_eq!([above, below], [dark, light]);
    }
}
