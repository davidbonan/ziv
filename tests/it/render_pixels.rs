use std::sync::Arc;

use ziv::color::domain::working_space::DisplayTransform;
use ziv::develop::domain::adjustments::Adjustments;
use ziv::develop::domain::brush::{BrushMask, Stroke};
use ziv::develop::domain::color_grading::ZoneGrade;
use ziv::develop::domain::color_mixer::ColorRange;
use ziv::develop::domain::coverage_image::{CoverageImage, coverage_image_size};
use ziv::develop::domain::development::Development;
use ziv::develop::domain::edit::Edit;
use ziv::develop::domain::linear_gradient::LinearGradient;
use ziv::develop::domain::mask::{Mask, MaskShape, photo_extent};
use ziv::develop::domain::tone_curve::ToneCurve;
use ziv::develop::domain::zone::{Zone, ZoneMask};
use ziv::engine::infrastructure::display_stage::DisplayRequest;
use ziv::photo::domain::working_image::WorkingImage;

use crate::gpu::headless_engine;
use crate::synthetic::{HEIGHT, WIDTH, ramp_and_patches};

const TALL_WIDTH: u32 = 4;
const TALL_HEIGHT: u32 = 2500;

/// Taller than one render strip, every row its own grey.
fn tall_gradient() -> WorkingImage {
    let pixels = (0..TALL_HEIGHT)
        .flat_map(|y| [[y as f32 / TALL_HEIGHT as f32; 3]; TALL_WIDTH as usize])
        .collect();
    WorkingImage::new(TALL_WIDTH, TALL_HEIGHT, pixels)
}

fn brighter() -> Development {
    Development {
        edit: Edit::from(Adjustments {
            exposure: 0.5,
            ..Adjustments::default()
        }),
        ..Development::default()
    }
}

#[test]
fn full_size_pixels_are_the_developed_photo_row_for_row_across_strips() {
    let engine = headless_engine();
    let image = tall_gradient();
    let display = DisplayTransform::default();

    let pixels = engine
        .render_pixels(
            &engine.upload(&image),
            &brighter(),
            [TALL_WIDTH, TALL_HEIGHT],
        )
        .unwrap();

    assert_eq!((pixels.width, pixels.height), (TALL_WIDTH, TALL_HEIGHT));
    for (rendered, working) in pixels.rgba.as_chunks::<4>().0.iter().zip(image.pixels()) {
        let expected = brighter()
            .to_display(*working, [0.0; 2], &display)
            .map(|channel| (channel * 255.0).round() as u8);
        let within_one = rendered[..3]
            .iter()
            .zip(expected)
            .all(|(rendered, expected)| rendered.abs_diff(expected) <= 1);
        assert!(
            within_one,
            "{working:?}: GPU {rendered:?}, expected {expected:?}"
        );
    }
}

#[test]
fn pixels_at_the_photo_size_equal_the_display_render() {
    let engine = headless_engine();
    let source = engine.upload(&ramp_and_patches());
    let request = DisplayRequest {
        development: brighter(),
        ..DisplayRequest::whole_source([WIDTH, HEIGHT])
    };

    let displayed = engine
        .read_display_pixels(&engine.render_display(&source, &request))
        .unwrap();
    let pixels = engine
        .render_pixels(&source, &brighter(), [WIDTH, HEIGHT])
        .unwrap();

    assert_eq!(pixels, displayed);
}

fn curved_mixed_and_graded() -> Development {
    let mut edit = Edit::default();
    edit.tone_curves.rgb =
        ToneCurve::try_from(vec![[0.0, 0.0], [0.25, 0.15], [0.75, 0.85], [1.0, 1.0]]).unwrap();
    edit.color_mixer.saturation[ColorRange::Green] = -60.0;
    edit.color_mixer.luminance[ColorRange::Blue] = -40.0;
    edit.color_grading.shadows = ZoneGrade {
        hue: 185.0,
        saturation: 60.0,
        luminance: 0.0,
    };
    Development {
        edit,
        ..Development::default()
    }
}

#[test]
fn pixels_of_a_photo_with_curve_color_mixer_and_color_grading_equal_the_display_render() {
    let engine = headless_engine();
    let source = engine.upload(&ramp_and_patches());

    for size in [[WIDTH, HEIGHT], [WIDTH / 2, HEIGHT / 2]] {
        let request = DisplayRequest {
            development: curved_mixed_and_graded(),
            ..DisplayRequest::whole_source(size)
        };
        let displayed = engine
            .read_display_pixels(&engine.render_display(&source, &request))
            .unwrap();
        let pixels = engine
            .render_pixels(&source, &curved_mixed_and_graded(), size)
            .unwrap();
        let untouched = engine
            .render_pixels(&source, &Development::default(), size)
            .unwrap();

        assert_eq!(pixels, displayed, "at {size:?}");
        assert_ne!(pixels, untouched, "at {size:?}");
    }
}

#[test]
fn pixels_can_be_rendered_smaller_than_the_photo() {
    let engine = headless_engine();
    let source = engine.upload(&tall_gradient());

    let pixels = engine
        .render_pixels(&source, &Development::default(), [2, TALL_HEIGHT / 2])
        .unwrap();

    assert_eq!((pixels.width, pixels.height), (2, TALL_HEIGHT / 2));
    let [top, bottom] = [0, pixels.rgba.len() - 4].map(|index| pixels.rgba[index]);
    assert!(top < 10 && bottom > 245, "top {top}, bottom {bottom}");
}

/// Fading in then out down the photo, across the border between two strips.
fn detected_band() -> MaskShape {
    let [width, height] = coverage_image_size([TALL_WIDTH, TALL_HEIGHT]);
    let rows = (0..height).map(|row| 255 - (row.abs_diff(height / 2) / 2).min(255) as u8);
    let values = rows.flat_map(|value| vec![value; width as usize]).collect();
    MaskShape::Zone(ZoneMask {
        zone: Zone::Sky,
        coverage: Arc::new(CoverageImage::new([width, height], values).unwrap()),
    })
}

/// A gradient, a brush stroke and a zone, all crossing the border between two strips.
fn masked_across_strips() -> Development {
    let [right, _] = photo_extent([TALL_WIDTH, TALL_HEIGHT]);
    let darker = Adjustments {
        exposure: -2.0,
        ..Adjustments::default()
    };
    let gradient = MaskShape::LinearGradient(LinearGradient {
        full: [0.0, 0.3],
        none: [0.0, 0.6],
    });
    let stroke = Stroke {
        points: vec![[right / 2.0, 0.7], [right / 2.0, 0.9]],
        radius: 0.05,
        feather: 50.0,
        flow: 100.0,
        is_erasing: false,
    };
    let painted = MaskShape::Brush(BrushMask {
        strokes: vec![stroke],
    });
    let masks = [gradient, painted, detected_band()].map(|shape| Mask {
        adjustments: darker,
        ..Mask::of(shape)
    });
    Development {
        edit: Edit {
            masks: masks.to_vec(),
            ..Edit::default()
        },
        ..Development::default()
    }
}

#[test]
fn masks_stay_in_place_across_strips_and_sizes() {
    let engine = headless_engine();
    let source = engine.upload(&tall_gradient());
    let development = masked_across_strips();

    for size in [[TALL_WIDTH, TALL_HEIGHT], [2, TALL_HEIGHT / 2]] {
        let request = DisplayRequest {
            development: development.clone(),
            ..DisplayRequest::whole_source(size)
        };
        let displayed = engine
            .read_display_pixels(&engine.render_display(&source, &request))
            .unwrap();

        let pixels = engine.render_pixels(&source, &development, size).unwrap();

        // A strip computes a place on the photo with other rounding than the whole render.
        let furthest_apart = pixels
            .rgba
            .iter()
            .zip(&displayed.rgba)
            .map(|(strip, whole)| strip.abs_diff(*whole))
            .max();
        assert_eq!(pixels.rgba.len(), displayed.rgba.len(), "at {size:?}");
        assert!(furthest_apart <= Some(1), "at {size:?}: {furthest_apart:?}");
    }
}

#[test]
fn masked_pixels_are_the_domain_s_at_full_size() {
    let engine = headless_engine();
    let image = tall_gradient();
    let display = DisplayTransform::default();
    let development = masked_across_strips();
    let size = [TALL_WIDTH, TALL_HEIGHT];
    let [right, bottom] = photo_extent(size);

    let pixels = engine
        .render_pixels(&engine.upload(&image), &development, size)
        .unwrap();

    let rows = pixels.rgba.as_chunks::<{ 4 * TALL_WIDTH as usize }>().0;
    for (y, (rendered, working)) in rows.iter().zip(image.pixels().chunks(4)).enumerate() {
        let centre = [
            0.5 / TALL_WIDTH as f32 * right,
            (y as f32 + 0.5) / TALL_HEIGHT as f32 * bottom,
        ];
        let expected = (development.to_display(working[0], centre, &display)[0] * 255.0).round();
        assert!(
            (f32::from(rendered[0]) - expected).abs() <= 1.0,
            "row {y}: GPU {}, expected {expected}",
            rendered[0]
        );
    }
}
