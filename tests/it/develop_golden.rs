use std::sync::Arc;

use ziv::color::domain::illuminant::Illuminant;
use ziv::color::domain::working_space::DisplayTransform;
use ziv::develop::domain::adjustments::Adjustments;
use ziv::develop::domain::brush::{BrushMask, Stroke};
use ziv::develop::domain::color_grading::{ColorGrading, TonalZone, ZoneGrade};
use ziv::develop::domain::color_mixer::{ColorMixer, ColorRange};
use ziv::develop::domain::coverage_image::CoverageImage;
use ziv::develop::domain::development::Development;
use ziv::develop::domain::edit::Edit;
use ziv::develop::domain::linear_gradient::LinearGradient;
use ziv::develop::domain::mask::{Mask, MaskShape, PhotoPoint, photo_extent};
use ziv::develop::domain::overlay::overlaid;
use ziv::develop::domain::polygon::Polygon;
use ziv::develop::domain::preset::{AppliedPreset, Preset};
use ziv::develop::domain::radial_gradient::RadialGradient;
use ziv::develop::domain::rectangle::Rectangle;
use ziv::develop::domain::tone_curve::{CurveChannel, ToneCurve, ToneCurves};
use ziv::develop::domain::white_balance::WhiteBalance;
use ziv::develop::domain::zone::{Zone, ZoneMask};
use ziv::engine::infrastructure::display_readback::DisplayPixels;
use ziv::engine::infrastructure::display_stage::DisplayRequest;
use ziv::photo::domain::photo_kind::PhotoKind;
use ziv::photo::domain::working_image::WorkingImage;

use crate::golden::assert_matches_golden;
use crate::gpu::headless_engine;
use crate::synthetic::{HEIGHT, WIDTH, hues_and_named_colors, ramp_and_patches};

const RAW: PhotoKind = PhotoKind::Raw {
    as_shot: Illuminant {
        temperature: 5200.0,
        tint: 10.0,
    },
};

fn developed(development: Development) -> DisplayPixels {
    developed_from(&ramp_and_patches(), development)
}

fn developed_from(image: &WorkingImage, development: Development) -> DisplayPixels {
    let engine = headless_engine();
    let source = engine.upload(image);
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
    assert_renders_from(&ramp_and_patches(), development, expected);
}

fn assert_renders_from(
    image: &WorkingImage,
    development: &Development,
    expected: impl Fn([f32; 3], PhotoPoint) -> [f32; 3],
) {
    let rendered = developed_from(image, development.clone());
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

fn curved(points: &[[f32; 2]]) -> Edit {
    Edit {
        tone_curves: ToneCurves {
            rgb: ToneCurve::try_from(points.to_vec()).unwrap(),
            ..ToneCurves::default()
        },
        ..Edit::default()
    }
}

fn curved_on(channel: CurveChannel, points: &[[f32; 2]]) -> Edit {
    let curve = ToneCurve::try_from(points.to_vec()).unwrap();
    Edit {
        tone_curves: ToneCurves::default().with(channel, curve),
        ..Edit::default()
    }
}

#[test]
fn each_channel_tone_curve_matches_its_golden_and_the_domain() {
    let raised_middle = [[0.0, 0.0], [0.5, 0.7], [1.0, 1.0]];
    let channels = [
        ("develop_tone_curve_red_raised", CurveChannel::Red),
        ("develop_tone_curve_green_raised", CurveChannel::Green),
        ("develop_tone_curve_blue_raised", CurveChannel::Blue),
    ];

    for (golden, channel) in channels {
        let edit = curved_on(channel, &raised_middle);
        assert_adjustment_matches_its_golden_and_the_domain(golden, edit);
    }
}

#[test]
fn the_four_tone_curves_combined_agree_with_the_domain() {
    let lifted_blue_blacks = ToneCurve::try_from(vec![[0.0, 0.15], [1.0, 1.0]]).unwrap();
    let lowered_red = ToneCurve::try_from(vec![[0.0, 0.0], [0.5, 0.35], [1.0, 1.0]]).unwrap();
    let s_curve = curved(&[[0.0, 0.0], [0.25, 0.12], [0.75, 0.88], [1.0, 1.0]]);
    let combined = Edit {
        tone_curves: s_curve
            .tone_curves
            .clone()
            .with(CurveChannel::Blue, lifted_blue_blacks)
            .with(CurveChannel::Red, lowered_red),
        ..s_curve
    };

    assert_shader_agrees_with_the_domain(&edited(combined));
}

#[test]
fn rgb_tone_curve_matches_its_golden_and_the_domain() {
    let s_curve = curved(&[[0.0, 0.0], [0.25, 0.12], [0.75, 0.88], [1.0, 1.0]]);

    assert_adjustment_matches_its_golden_and_the_domain("develop_tone_curve_rgb_s", s_curve);
}

#[test]
fn tone_curve_with_moved_end_points_agrees_with_the_domain() {
    let faded = curved(&[[0.1, 0.2], [0.5, 0.6], [0.9, 0.8]]);

    assert_shader_agrees_with_the_domain(&edited(faded));
}

#[test]
fn tone_curve_on_a_camera_like_rendering_agrees_with_the_domain() {
    let development = Development {
        kind: RAW,
        edit: curved(&[[0.0, 0.0], [0.3, 0.45], [1.0, 1.0]]),
    };

    assert_shader_agrees_with_the_domain(&development);
}

fn mixed(adjust: impl Fn(&mut ColorMixer)) -> Edit {
    let mut color_mixer = ColorMixer::default();
    adjust(&mut color_mixer);
    Edit {
        color_mixer,
        ..Edit::default()
    }
}

fn assert_colors_match_their_golden_and_the_domain(golden: &str, development: &Development) {
    let colors = hues_and_named_colors();
    let display = DisplayTransform::default();

    assert_matches_golden(golden, &developed_from(&colors, development.clone()));
    assert_renders_from(&colors, development, |working, centre| {
        development.to_display(working, centre, &display)
    });
}

#[test]
fn undeveloped_hues_and_named_colors_match_their_golden() {
    let undeveloped = Development::default();

    assert_colors_match_their_golden_and_the_domain("display_hues_and_named_colors", &undeveloped);
}

#[test]
fn color_mixer_hue_matches_its_golden_and_the_domain() {
    let turned = mixed(|mixer| {
        mixer.hue[ColorRange::Red] = 100.0;
        mixer.hue[ColorRange::Green] = -100.0;
        mixer.hue[ColorRange::Blue] = 100.0;
    });

    assert_colors_match_their_golden_and_the_domain("develop_color_mixer_hue", &edited(turned));
}

#[test]
fn color_mixer_saturation_matches_its_golden_and_the_domain() {
    let muted_and_vivid = mixed(|mixer| {
        mixer.saturation[ColorRange::Green] = -100.0;
        mixer.saturation[ColorRange::Orange] = -100.0;
        mixer.saturation[ColorRange::Purple] = 60.0;
    });

    assert_colors_match_their_golden_and_the_domain(
        "develop_color_mixer_saturation",
        &edited(muted_and_vivid),
    );
}

#[test]
fn color_mixer_luminance_matches_its_golden_and_the_domain() {
    let darker_and_brighter = mixed(|mixer| {
        mixer.luminance[ColorRange::Blue] = -100.0;
        mixer.luminance[ColorRange::Aqua] = -100.0;
        mixer.luminance[ColorRange::Yellow] = 100.0;
    });

    assert_colors_match_their_golden_and_the_domain(
        "develop_color_mixer_luminance",
        &edited(darker_and_brighter),
    );
}

#[test]
fn color_mixer_leaves_the_grey_ramp_as_it_was() {
    let everything = mixed(|mixer| {
        for range in ColorRange::ALL {
            mixer.hue[range] = 100.0;
            mixer.saturation[range] = -100.0;
            mixer.luminance[range] = 100.0;
        }
    });
    let ramp_rows = (HEIGHT / 2 * WIDTH * 4) as usize;

    let mixed = developed(edited(everything));
    let untouched = developed(Development::default());

    assert_eq!(mixed.rgba[..ramp_rows], untouched.rgba[..ramp_rows]);
}

#[test]
fn color_mixer_on_a_masked_camera_like_rendering_agrees_with_the_domain() {
    let warmer_left = Mask {
        adjustments: Adjustments {
            exposure: 0.8,
            saturation: 40.0,
            ..Adjustments::default()
        },
        ..Mask::of(MaskShape::LinearGradient(LinearGradient {
            full: [0.2, 0.25],
            none: [0.8, 0.25],
        }))
    };
    let edit = Edit {
        masks: vec![warmer_left],
        ..mixed(|mixer| {
            mixer.hue[ColorRange::Red] = -60.0;
            mixer.saturation[ColorRange::Blue] = 50.0;
            mixer.luminance[ColorRange::Green] = -70.0;
        })
    };
    let development = Development { kind: RAW, edit };
    let display = DisplayTransform::default();

    assert_renders_from(&hues_and_named_colors(), &development, |working, centre| {
        development.to_display(working, centre, &display)
    });
}

fn graded(zone: TonalZone, grade: ZoneGrade) -> Edit {
    Edit {
        color_grading: ColorGrading::default().with(zone, grade),
        ..Edit::default()
    }
}

#[test]
fn each_tonal_zone_matches_its_golden_and_the_domain() {
    let teal_and_brighter = ZoneGrade {
        hue: 180.0,
        saturation: 80.0,
        luminance: 30.0,
    };
    let zones = [
        ("develop_color_grading_shadows", TonalZone::Shadows),
        ("develop_color_grading_midtones", TonalZone::Midtones),
        ("develop_color_grading_highlights", TonalZone::Highlights),
        ("develop_color_grading_global", TonalZone::Global),
    ];

    for (golden, zone) in zones {
        let edit = graded(zone, teal_and_brighter);
        assert_adjustment_matches_its_golden_and_the_domain(golden, edit);
    }
}

#[test]
fn color_grading_with_balance_and_blending_agrees_with_the_domain() {
    let orange = ZoneGrade {
        hue: 35.0,
        saturation: 70.0,
        luminance: -20.0,
    };
    let teal = ZoneGrade {
        hue: 185.0,
        saturation: 60.0,
        luminance: 15.0,
    };
    let color_grading = ColorGrading {
        blending: 15.0,
        balance: 60.0,
        ..ColorGrading::default()
            .with(TonalZone::Highlights, orange)
            .with(TonalZone::Shadows, teal)
    };

    assert_shader_agrees_with_the_domain(&edited(Edit {
        color_grading,
        ..Edit::default()
    }));
}

#[test]
fn color_grading_under_a_tone_curve_on_a_camera_like_rendering_agrees_with_the_domain() {
    let magenta = ZoneGrade {
        hue: 300.0,
        saturation: 50.0,
        luminance: 10.0,
    };
    let edit = Edit {
        color_grading: ColorGrading::default().with(TonalZone::Midtones, magenta),
        ..curved(&[[0.0, 0.05], [0.4, 0.5], [1.0, 0.95]])
    };

    assert_shader_agrees_with_the_domain(&Development { kind: RAW, edit });
}

#[test]
fn a_masked_area_gets_the_curve_the_color_mixer_and_the_color_grading_like_the_rest() {
    let brighter_left = Mask {
        adjustments: Adjustments {
            exposure: 1.0,
            ..Adjustments::default()
        },
        ..Mask::of(MaskShape::LinearGradient(LinearGradient {
            full: [0.2, 0.25],
            none: [0.8, 0.25],
        }))
    };
    let mut edit = Edit {
        masks: vec![brighter_left],
        ..curved(&[[0.0, 0.0], [0.25, 0.15], [0.75, 0.85], [1.0, 1.0]])
    };
    edit.color_mixer.saturation[ColorRange::Red] = -80.0;
    edit.color_grading.global = ZoneGrade {
        hue: 200.0,
        saturation: 40.0,
        luminance: 0.0,
    };

    assert_shader_agrees_with_the_domain(&edited(edit));
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
fn mask_of_an_applied_preset_at_a_partial_intensity_agrees_with_the_domain() {
    let dosed = AppliedPreset {
        intensity: 35.0,
        ..AppliedPreset::of(Preset::EnhancedSky, 0)
    };
    let of_the_applied_preset = Mask {
        applied_preset: Some(dosed),
        ..slanted_gradient(two_stops_darker())
    };

    assert_shader_agrees_with_the_domain(&Development {
        kind: RAW,
        edit: masked(vec![of_the_applied_preset]),
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
