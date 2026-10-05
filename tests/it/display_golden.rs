use ziv::color::domain::working_space::DisplayTransform;
use ziv::engine::infrastructure::display_readback::DisplayPixels;
use ziv::engine::infrastructure::display_stage::{DisplayRequest, Sampling};
use ziv::photo::domain::picture_region::PictureRegion;
use ziv::photo::domain::working_image::WorkingImage;

use crate::golden::assert_matches_golden;
use crate::gpu::headless_engine;
use crate::synthetic::{HEIGHT, WIDTH, ramp_and_patches};

#[test]
fn display_render_matches_its_golden() {
    let engine = headless_engine();
    let source = engine.upload(&ramp_and_patches());

    let output = engine.render_display(&source, &DisplayRequest::whole_source([WIDTH, HEIGHT]));

    let pixels = engine.read_display_pixels(&output).unwrap();
    assert_matches_golden("display_ramp_and_patches", &pixels);
}

#[test]
fn shader_agrees_with_the_domain_display_transform() {
    let image = ramp_and_patches();
    let engine = headless_engine();
    let source = engine.upload(&image);

    let output = engine.render_display(&source, &DisplayRequest::whole_source([WIDTH, HEIGHT]));

    let pixels = engine.read_display_pixels(&output).unwrap();
    let display = DisplayTransform::default();
    for (rendered, working) in pixels.rgba.as_chunks::<4>().0.iter().zip(image.pixels()) {
        let expected = display
            .to_display(*working)
            .map(|channel| (channel * 255.0).round() as u8);
        let within_one = rendered[..3]
            .iter()
            .zip(expected)
            .all(|(rendered, expected)| rendered.abs_diff(expected) <= 1);
        assert!(
            within_one,
            "{working:?}: GPU {rendered:?}, domain {expected:?}"
        );
    }
}

#[test]
fn downscaled_render_averages_in_linear_light() {
    const SIZE: u32 = 8;
    let one_white_column_in_four = (0..SIZE * SIZE)
        .map(|index| if index % 4 == 0 { [1.0; 3] } else { [0.0; 3] })
        .collect();
    let engine = headless_engine();
    let source = engine.upload(&WorkingImage::new(SIZE, SIZE, one_white_column_in_four));

    let output = engine.render_display(&source, &DisplayRequest::whole_source([2, 2]));

    let quarter_white =
        (DisplayTransform::default().to_display([0.25; 3])[0] * 255.0).round() as u8;
    let pixels = engine.read_display_pixels(&output).unwrap();
    for rendered in pixels.rgba.as_chunks::<4>().0 {
        assert!(
            rendered[0].abs_diff(quarter_white) <= 2,
            "{rendered:?} is not {quarter_white}"
        );
    }
}

fn black_then_white() -> WorkingImage {
    WorkingImage::new(2, 1, vec![[0.0; 3], [1.0; 3]])
}

fn red_channels(pixels: &DisplayPixels) -> Vec<u8> {
    pixels
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|pixel| pixel[0])
        .collect()
}

#[test]
fn region_render_shows_only_that_part_of_the_source() {
    let engine = headless_engine();
    let source = engine.upload(&black_then_white());
    let white_half = DisplayRequest {
        region: PictureRegion::upright([0.5, 0.0], [0.5, 1.0]),
        sampling: Sampling::Pixelated,
        ..DisplayRequest::whole_source([4, 1])
    };

    let output = engine.render_display(&source, &white_half);

    let pixels = engine.read_display_pixels(&output).unwrap();
    assert_eq!(red_channels(&pixels), [255; 4]);
}

#[test]
fn pixelated_magnification_keeps_photo_pixels_as_sharp_squares() {
    let engine = headless_engine();
    let source = engine.upload(&black_then_white());
    let magnified = DisplayRequest {
        sampling: Sampling::Pixelated,
        ..DisplayRequest::whole_source([8, 1])
    };

    let output = engine.render_display(&source, &magnified);

    let pixels = engine.read_display_pixels(&output).unwrap();
    assert_eq!(red_channels(&pixels), [0, 0, 0, 0, 255, 255, 255, 255]);
}
