/// Largest size at which the whole photo is visible in `available`, never
/// enlarging it. Same unit as `available`, aspect ratio kept.
pub fn fit_size(photo: [u32; 2], available: [f32; 2]) -> [f32; 2] {
    let photo = photo.map(|side| side as f32);
    let scale = (available[0] / photo[0])
        .min(available[1] / photo[1])
        .min(1.0);
    photo.map(|side| side * scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn landscape_photo_fills_the_width() {
        assert_eq!(fit_size([4000, 2000], [400.0, 300.0]), [400.0, 200.0]);
    }

    #[test]
    fn portrait_photo_fills_the_height() {
        assert_eq!(fit_size([2000, 4000], [400.0, 300.0]), [150.0, 300.0]);
    }

    #[test]
    fn photo_smaller_than_the_viewport_is_not_enlarged() {
        assert_eq!(fit_size([100, 50], [400.0, 300.0]), [100.0, 50.0]);
    }
}
