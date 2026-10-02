use std::sync::Arc;

use ziv::color::domain::illuminant::Illuminant;
use ziv::color::domain::working_space::DisplayTransform;
use ziv::develop::domain::adjustments::Adjustments;
use ziv::develop::domain::brush::{BrushMask, Stroke};
use ziv::develop::domain::coverage_image::CoverageImage;
use ziv::develop::domain::development::Development;
use ziv::develop::domain::edit::Edit;
use ziv::develop::domain::linear_gradient::LinearGradient;
use ziv::develop::domain::mask::{Mask, MaskShape, PhotoPoint, photo_extent};
use ziv::develop::domain::overlay::overlaid;
use ziv::develop::domain::polygon::Polygon;
use ziv::develop::domain::radial_gradient::RadialGradient;
use ziv::develop::domain::rectangle::Rectangle;
use ziv::develop::domain::white_balance::WhiteBalance;
use ziv::develop::domain::zone::{Zone, ZoneMask};
use ziv::engine::infrastructure::display_readback::DisplayPixels;
use ziv::engine::infrastructure::display_stage::DisplayRequest;
use ziv::photo::domain::photo_kind::PhotoKind;

use crate::golden::assert_matches_golden;
use crate::gpu::headless_engine;
use crate::synthetic::{HEIGHT, WIDTH, ramp_and_patches};

const RAW: PhotoKind = PhotoKind::Raw {
    as_shot: Illuminant {
        temperature: 5200.0,
        tint: 10.0,
    },
};

fn developed(development: Development) -> DisplayPixels {
    let engine = headless_engine();
    let source = engine.upload(&ramp_and_patches());
    let request = DisplayRequest {
        development,
        ..DisplayRequest::whole_source([WIDTH, HEIGHT])
    };
    let output = engine.render_display(&source, &request);
    engine.read_display_pixels(&output).unwrap()
}

fn pixel_centres() -> impl Iterator<Item = PhotoPoint> {
    let [right, bottom] = photo_extent([WIDTH, HEIGHT]);
    (0..HEIGHT).flat_map(move |y| {
        (0..WIDTH).map(move |x| {
            [
                (x as f32 + 0.5) / WIDTH as f32 * right,
                (y as f32 + 0.5) / HEIGHT as f32 * bottom,
            ]
        })
    })
}

fn assert_renders(development: &Development, expected: impl Fn([f32; 3], PhotoPoint) -> [f32; 3]) {
    let rendered = developed(development.clone());
    let image = ramp_and_patches();
    let pixels = image.pixels().iter().zip(pixel_centres());
    for (rendered, (working, centre)) in rendered.rgba.as_chunks::<4>().0.iter().zip(pixels) {
        let expected = expected(*working, centre).map(|channel| (channel * 255.0).round() as u8);
        let within_one = rendered[..3]
            .iter()
            .zip(expected)
            .all(|(rendered, expected)| rendered.abs_diff(expected) <= 1);
        assert!(
            within_one,
            "{development:?} on {working:?}: GPU {rendered:?}, expected {expected:?}"
        );
    }
}

fn assert_shader_agrees_with_the_domain(development: &Development) {
    let display = DisplayTransform::default();
    assert_renders(development, |working, centre| {
        development.to_display(working, centre, &display)
    });
}

fn edited(edit: Edit) -> Development {
    Development {
        edit,
        ..Development::default()
    }
}

#[test]
fn default_development_renders_the_undeveloped_photo() {
    assert_matches_golden(
        "display_ramp_and_patches",
        &developed(Development::default()),
    );
}

#[test]
fn default_development_is_the_display_transform_alone() {
    let display = DisplayTransform::default();

    assert_renders(&Development::default(), |working, _| {
        display.to_display(working)
    });
}

fn assert_adjustment_matches_its_golden_and_the_domain(golden: &str, edit: Edit) {
    let development = edited(edit);

    assert_matches_golden(golden, &developed(development.clone()));
    assert_shader_agrees_with_the_domain(&development);
}

#[test]
fn exposure_matches_its_golden_and_the_domain() {
    let darker = Edit::from(Adjustments {
        exposure: -1.5,
        ..Adjustments::default()
    });

    assert_adjustment_matches_its_golden_and_the_domain("develop_exposure_minus_1_5", darker);
}

#[test]
fn contrast_matches_its_golden_and_the_domain() {
    let punchier = Edit::from(Adjustments {
        contrast: 60.0,
        ..Adjustments::default()
    });

    assert_adjustment_matches_its_golden_and_the_domain("develop_contrast_plus_60", punchier);
}

#[test]
fn highlights_match_their_golden_and_the_domain() {
    let recovered = Edit::from(Adjustments {
        highlights: -80.0,
        ..Adjustments::default()
    });

    assert_adjustment_matches_its_golden_and_the_domain("develop_highlights_minus_80", recovered);
}

#[test]
fn shadows_match_their_golden_and_the_domain() {
    let opened = Edit::from(Adjustments {
        shadows: 70.0,
        ..Adjustments::default()
    });

    assert_adjustment_matches_its_golden_and_the_domain("develop_shadows_plus_70", opened);
}

#[test]
fn whites_match_their_golden_and_the_domain() {
    let dimmed = Edit::from(Adjustments {
        whites: -50.0,
        ..Adjustments::default()
    });

    assert_adjustment_matches_its_golden_and_the_domain("develop_whites_minus_50", dimmed);
}

#[test]
fn blacks_match_their_golden_and_the_domain() {
    let lifted = Edit::from(Adjustments {
        blacks: 60.0,
        ..Adjustments::default()
    });

    assert_adjustment_matches_its_golden_and_the_domain("develop_blacks_plus_60", lifted);
}

#[test]
fn vibrance_matches_its_golden_and_the_domain() {
    let livelier = Edit::from(Adjustments {
        vibrance: 70.0,
        ..Adjustments::default()
    });

    assert_adjustment_matches_its_golden_and_the_domain("develop_vibrance_plus_70", livelier);
}

#[test]
fn saturation_matches_its_golden_and_the_domain() {
    let duller = Edit::from(Adjustments {
        saturation: -60.0,
        ..Adjustments::default()
    });

    assert_adjustment_matches_its_golden_and_the_domain("develop_saturation_minus_60", duller);
}

#[test]
fn camera_like_base_rendering_matches_its_golden_and_the_domain() {
    let camera_like = Development {
        kind: RAW,
        ..Development::default()
    };

    assert_matches_golden(
        "develop_base_rendering_camera_like",
        &developed(camera_like.clone()),
    );
    assert_shader_agrees_with_the_domain(&camera_like);
}

#[test]
fn tone_on_a_camera_like_rendering_agrees_with_the_domain() {
    assert_shader_agrees_with_the_domain(&Development {
        kind: RAW,
        edit: Edit::from(Adjustments {
            contrast: 80.0,
            highlights: -60.0,
            shadows: 40.0,
            whites: 30.0,
            blacks: -50.0,
            ..Adjustments::default()
        }),
    });
}

#[test]
fn relative_white_balance_matches_its_golden_and_the_domain() {
    let warmer_and_greener = Edit::from(Adjustments {
        white_balance: Some(WhiteBalance {
            temperature: 40.0,
            tint: -30.0,
        }),
        ..Adjustments::default()
    });

    assert_adjustment_matches_its_golden_and_the_domain(
        "develop_white_balance_plus_40_minus_30",
        warmer_and_greener,
    );
}

#[test]
fn kelvin_white_balance_agrees_with_the_domain() {
    assert_shader_agrees_with_the_domain(&Development {
        kind: RAW,
        edit: Edit::from(Adjustments {
            white_balance: Some(WhiteBalance {
                temperature: 3400.0,
                tint: 45.0,
            }),
            ..Adjustments::default()
        }),
    });
}

fn slanted_gradient(adjustments: Adjustments) -> Mask {
    let [right, bottom] = photo_extent([WIDTH, HEIGHT]);
    let shape = MaskShape::LinearGradient(LinearGradient {
        full: [0.3 * right, 0.2 * bottom],
        none: [0.7 * right, 0.8 * bottom],
    });
    Mask {
        adjustments,
        ..Mask::of(shape)
    }
}

fn two_stops_darker() -> Adjustments {
    Adjustments {
        exposure: -2.0,
        ..Adjustments::default()
    }
}

fn masked(masks: Vec<Mask>) -> Edit {
    Edit {
        masks,
        ..Edit::default()
    }
}

#[test]
fn linear_gradient_mask_matches_its_golden_and_the_domain() {
    assert_adjustment_matches_its_golden_and_the_domain(
        "mask_linear_gradient_exposure_minus_2",
        masked(vec![slanted_gradient(two_stops_darker())]),
    );
}

#[test]
fn inverted_overlapping_and_hidden_masks_agree_with_the_domain() {
    let duller_elsewhere = Mask {
        is_inverted: true,
        ..slanted_gradient(Adjustments {
            saturation: -80.0,
            contrast: 40.0,
            ..Adjustments::default()
        })
    };
    let hidden = Mask {
        is_hidden: true,
        ..slanted_gradient(Adjustments {
            exposure: 3.0,
            ..Adjustments::default()
        })
    };
    let masks = vec![
        slanted_gradient(two_stops_darker()),
        hidden,
        duller_elsewhere,
    ];

    assert_shader_agrees_with_the_domain(&Development {
        kind: RAW,
        edit: Edit {
            adjustments: Adjustments {
                exposure: 0.5,
                ..Adjustments::default()
            },
            masks,
            ..Edit::default()
        },
    });
}

#[test]
fn overlay_veils_the_photo_by_the_coverage_of_the_mask() {
    let engine = headless_engine();
    let source = engine.upload(&ramp_and_patches());
    let mask = slanted_gradient(Adjustments::default());
    let request = DisplayRequest {
        overlaid_mask: Some(mask.clone()),
        ..DisplayRequest::whole_source([WIDTH, HEIGHT])
    };

    let output = engine.render_display(&source, &request);
    let rendered = engine.read_display_pixels(&output).unwrap();

    let display = DisplayTransform::default();
    let image = ramp_and_patches();
    let pixels = image.pixels().iter().zip(pixel_centres());
    for (rendered, (working, centre)) in rendered.rgba.as_chunks::<4>().0.iter().zip(pixels) {
        let expected = overlaid(display.to_display(*working), mask.coverage(centre))
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

fn tilted_ellipse(feather: f32) -> MaskShape {
    let [right, bottom] = photo_extent([WIDTH, HEIGHT]);
    MaskShape::RadialGradient(RadialGradient {
        centre: [0.45 * right, 0.5 * bottom],
        radii: [0.3 * right, 0.3 * bottom],
        rotation: 0.4,
        feather,
    })
}

#[test]
fn radial_gradient_mask_matches_its_golden_and_the_domain() {
    let brighter = Mask {
        adjustments: Adjustments {
            exposure: -2.0,
            ..Adjustments::default()
        },
        ..Mask::of(tilted_ellipse(60.0))
    };

    assert_adjustment_matches_its_golden_and_the_domain(
        "mask_radial_gradient_exposure_minus_2",
        masked(vec![brighter]),
    );
}

fn two_stops_darker_in(shape: MaskShape) -> Edit {
    masked(vec![Mask {
        adjustments: two_stops_darker(),
        ..Mask::of(shape)
    }])
}

#[test]
fn rectangle_mask_matches_its_golden_and_the_domain() {
    let [right, bottom] = photo_extent([WIDTH, HEIGHT]);
    let soft_box = MaskShape::Rectangle(Rectangle {
        centre: [0.4 * right, 0.45 * bottom],
        half_size: [0.25 * right, 0.3 * bottom],
        feather: 40.0,
    });

    assert_adjustment_matches_its_golden_and_the_domain(
        "mask_rectangle_exposure_minus_2",
        two_stops_darker_in(soft_box),
    );
}

#[test]
fn polygon_mask_matches_its_golden_and_the_domain() {
    let [right, bottom] = photo_extent([WIDTH, HEIGHT]);
    let corners = [[0.1, 0.1], [0.6, 0.2], [0.9, 0.9], [0.5, 0.6], [0.15, 0.8]];
    let soft_outline = MaskShape::Polygon(Polygon {
        corners: corners.map(|[x, y]| [x * right, y * bottom]).to_vec(),
        feather: 30.0,
    });

    assert_adjustment_matches_its_golden_and_the_domain(
        "mask_polygon_exposure_minus_2",
        two_stops_darker_in(soft_outline),
    );
}

#[test]
fn brush_mask_matches_its_golden_and_the_domain() {
    let [right, bottom] = photo_extent([WIDTH, HEIGHT]);
    let on_photo = |points: &[[f32; 2]]| -> Vec<PhotoPoint> {
        points
            .iter()
            .map(|[x, y]| [x * right, y * bottom])
            .collect()
    };
    let painted = Stroke {
        points: on_photo(&[[0.1, 0.2], [0.5, 0.7], [0.9, 0.3]]),
        radius: 0.15,
        feather: 70.0,
        flow: 90.0,
        is_erasing: false,
    };
    let erased = Stroke {
        points: on_photo(&[[0.5, 0.1], [0.5, 0.9]]),
        radius: 0.06,
        is_erasing: true,
        ..painted.clone()
    };
    let strokes = vec![painted, erased];

    assert_adjustment_matches_its_golden_and_the_domain(
        "mask_brush_exposure_minus_2",
        two_stops_darker_in(MaskShape::Brush(BrushMask { strokes })),
    );
}

#[test]
fn zone_mask_matches_its_golden_and_the_domain() {
    let soft_disc = (0..HEIGHT).flat_map(|row| {
        (0..WIDTH).map(move |column| {
            let across = column as f32 / WIDTH as f32 - 0.4;
            let down = row as f32 / HEIGHT as f32 - 0.5;
            let from_edge = (0.35 - across.hypot(down)) / 0.1;
            (from_edge.clamp(0.0, 1.0) * 255.0).round() as u8
        })
    });
    let detected = ZoneMask {
        zone: Zone::Subject,
        coverage: Arc::new(CoverageImage::new([WIDTH, HEIGHT], soft_disc.collect()).unwrap()),
    };

    assert_adjustment_matches_its_golden_and_the_domain(
        "mask_zone_exposure_minus_2",
        two_stops_darker_in(MaskShape::Zone(detected)),
    );
}
