use ziv::color::domain::working_space::DisplayTransform;
use ziv::develop::domain::adjustments::Adjustments;
use ziv::develop::domain::development::Development;
use ziv::develop::domain::edit::Edit;
use ziv::develop::domain::linear_gradient::LinearGradient;
use ziv::develop::domain::mask::{Mask, MaskShape, photo_extent};
use ziv::engine::infrastructure::display_readback::DisplayPixels;
use ziv::engine::infrastructure::display_stage::DisplayRequest;
use ziv::engine::infrastructure::engine::Engine;
use ziv::engine::infrastructure::source_texture::SourceTexture;
use ziv::photo::domain::working_image::WorkingImage;

use crate::golden::assert_matches_golden;
use crate::gpu::headless_engine;
use crate::synthetic::{HEIGHT, WIDTH, ramp_and_patches};

/// What enhancing would never do, and shows: the photo upside down.
fn upside_down() -> WorkingImage {
    let photo = ramp_and_patches();
    let rows = photo.pixels().chunks(WIDTH as usize).rev();
    WorkingImage::new(WIDTH, HEIGHT, rows.flatten().copied().collect())
}

fn enhanced_source(engine: &Engine) -> SourceTexture {
    let source = engine.upload(&ramp_and_patches());
    engine.upload_enhancement(&source, &upside_down());
    source
}

/// Darker, and brighter again on the left: adjustments and a mask over the enhancement.
fn developed_at(enhancement_intensity: f32) -> Development {
    let brighter_left = Mask {
        adjustments: Adjustments {
            exposure: 1.0,
            ..Adjustments::default()
        },
        ..Mask::of(MaskShape::LinearGradient(LinearGradient {
            full: [0.2, 0.0],
            none: [0.8, 0.0],
        }))
    };
    let edit = Edit {
        adjustments: Adjustments {
            exposure: -1.0,
            ..Adjustments::default()
        },
        masks: vec![brighter_left],
        enhancement_intensity,
        ..Edit::default()
    };
    Development {
        edit,
        ..Development::default()
    }
}

fn rendered(engine: &Engine, source: &SourceTexture, development: &Development) -> DisplayPixels {
    let request = DisplayRequest {
        development: development.clone(),
        ..DisplayRequest::whole_source([WIDTH, HEIGHT])
    };
    engine
        .read_display_pixels(&engine.render_display(source, &request))
        .unwrap()
}

fn assert_is_the_domain_s(rendered: &DisplayPixels, development: &Development) {
    let display = DisplayTransform::default();
    let [right, bottom] = photo_extent([WIDTH, HEIGHT]);
    let [original, enhancement] = [ramp_and_patches(), upside_down()];
    for (index, rendered) in rendered.rgba.as_chunks::<4>().0.iter().enumerate() {
        let [x, y] = [index as u32 % WIDTH, index as u32 / WIDTH];
        let centre = [
            (x as f32 + 0.5) / WIDTH as f32 * right,
            (y as f32 + 0.5) / HEIGHT as f32 * bottom,
        ];
        let working = development.enhanced(original.pixel(x, y), enhancement.pixel(x, y));
        let expected = development
            .to_display(working, centre, &display)
            .map(|channel| (channel * 255.0).round() as u8);
        let within_one = rendered[..3]
            .iter()
            .zip(expected)
            .all(|(rendered, expected)| rendered.abs_diff(expected) <= 1);
        assert!(
            within_one,
            "at {x}, {y}: GPU {rendered:?}, expected {expected:?}"
        );
    }
}

#[test]
fn intensity_mixes_the_enhancement_in_before_adjustments_and_masks() {
    let engine = headless_engine();
    let source = enhanced_source(&engine);

    for intensity in [0.0, 50.0, 100.0] {
        let development = developed_at(intensity);

        let rendered = rendered(&engine, &source, &development);

        assert_is_the_domain_s(&rendered, &development);
    }
    let halfway = rendered(&engine, &source, &developed_at(50.0));
    assert_matches_golden("enhancement_intensity_50", &halfway);
}

#[test]
fn intensity_changes_nothing_on_a_photo_without_enhancement() {
    let engine = headless_engine();
    let source = engine.upload(&ramp_and_patches());

    let at_full = rendered(&engine, &source, &developed_at(100.0));

    assert_eq!(at_full, rendered(&engine, &source, &developed_at(0.0)));
}

#[test]
fn full_size_pixels_of_an_enhanced_photo_equal_the_display_render() {
    let engine = headless_engine();
    let source = enhanced_source(&engine);
    let development = developed_at(70.0);

    let pixels = engine
        .render_pixels(&source, &development, [WIDTH, HEIGHT])
        .unwrap();

    assert_eq!(pixels, rendered(&engine, &source, &development));
}
