// The statistics of ImageNet, which the models were trained with.
const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const DEVIATION: [f32; 3] = [0.229, 0.224, 0.225];

/// RGBA pixels as the models read them: the red values, then the green, then
/// the blue, each centred and scaled.
pub fn normalized_planes(rgba: &[u8]) -> Vec<f32> {
    let (pixels, _) = rgba.as_chunks::<4>();
    let plane = |channel: usize| {
        pixels.iter().map(move |pixel| {
            (f32::from(pixel[channel]) / 255.0 - MEAN[channel]) / DEVIATION[channel]
        })
    };
    plane(0).chain(plane(1)).chain(plane(2)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planes_are_red_then_green_then_blue_centred_on_the_mean() {
        let black_then_white = [0, 0, 0, 255, 255, 255, 255, 255];

        let planes = normalized_planes(&black_then_white);

        let expected = [
            -0.485 / 0.229,
            0.515 / 0.229,
            -0.456 / 0.224,
            0.544 / 0.224,
            -0.406 / 0.225,
            0.594 / 0.225,
        ];
        assert_eq!(planes.len(), expected.len());
        for (plane, expected) in planes.iter().zip(expected) {
            assert!(
                (plane - expected).abs() < 1e-5,
                "{plane} instead of {expected}"
            );
        }
    }
}
