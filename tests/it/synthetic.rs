use ziv::color::domain::working_space::SrgbInput;
use ziv::photo::domain::working_image::WorkingImage;

pub const WIDTH: u32 = 64;
pub const HEIGHT: u32 = 32;
const PATCH_WIDTH: u32 = 8;
const RAMP_PEAK: f32 = 2.0;

/// Top half: grey ramp reaching twice display white. Bottom half: color patches,
/// some outside the display gamut.
pub fn ramp_and_patches() -> WorkingImage {
    let srgb = SrgbInput::default();
    let patches = [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.18, 0.18, 0.18],
        srgb.to_working([1.0, 0.0, 0.0]),
        srgb.to_working([0.0, 1.0, 0.0]),
        srgb.to_working([0.0, 0.0, 1.0]),
    ];
    let pixels = (0..HEIGHT)
        .flat_map(|y| (0..WIDTH).map(move |x| (x, y)))
        .map(|(x, y)| {
            if y < HEIGHT / 2 {
                [RAMP_PEAK * x as f32 / (WIDTH - 1) as f32; 3]
            } else {
                patches[(x / PATCH_WIDTH) as usize]
            }
        })
        .collect();
    WorkingImage::new(WIDTH, HEIGHT, pixels)
}
