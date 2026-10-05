use std::sync::Arc;

use ziv::develop::domain::adjustments::Adjustments;
use ziv::develop::domain::coverage_image::{CoverageImage, coverage_image_size};
use ziv::develop::domain::development::Development;
use ziv::develop::domain::edit::Edit;
use ziv::develop::domain::framing::{CropFrame, Framing, Turn};
use ziv::develop::domain::linear_gradient::LinearGradient;
use ziv::develop::domain::mask::{Mask, MaskShape};
use ziv::develop::domain::zone::{Zone, ZoneMask};
use ziv::engine::infrastructure::display_readback::DisplayPixels;
use ziv::photo::domain::working_image::WorkingImage;
use ziv::viewport::domain::view::{View, Viewport};
use ziv::viewport::infrastructure::photo_presenter::ShownPhoto;

use crate::golden::assert_matches_golden;
use crate::gpu::headless_engine;

const PICTURE: [u32; 2] = [64, 32];

/// Red tells where a pixel is across the picture, green down it; a quarter of
/// the pixels are brighter in blue so that neighbours never look alike.
fn positions() -> WorkingImage {
    let [width, height] = PICTURE;
    let pixels = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| {
            let checker = if (x + y) % 4 == 0 { 0.6 } else { 0.1 };
            [x as f32 / width as f32, y as f32 / height as f32, checker]
        })
        .collect();
    WorkingImage::new(width, height, pixels)
}

fn framed(centre: [f32; 2], size: [f32; 2]) -> ShownPhoto {
    ShownPhoto {
        framing: Framing {
            frame: CropFrame { centre, size },
            ..Framing::default()
        },
        ..ShownPhoto::default()
    }
}

/// The photo as the viewport shows it at fit, one screen pixel per photo pixel.
fn rendered_at_fit(shown: &ShownPhoto) -> DisplayPixels {
    picture_rendered_at_fit(&positions(), shown)
}

fn picture_rendered_at_fit(picture: &WorkingImage, shown: &ShownPhoto) -> DisplayPixels {
    let engine = headless_engine();
    let source = engine.upload(picture);
    let viewport = Viewport {
        photo: shown.framing.framed_size(PICTURE),
        size: PICTURE.map(|side| side as f32),
    };
    let placement = View::fit().placement(&viewport);
    let output = engine.render_display(&source, &shown.request_at(PICTURE, &placement));
    engine.read_display_pixels(&output).unwrap()
}

fn rows_of(pixels: &DisplayPixels) -> Vec<&[u8]> {
    pixels.rgba.chunks(pixels.width as usize * 4).collect()
}

#[test]
fn a_framed_photo_is_the_pixels_of_the_picture_its_frame_holds() {
    let whole = rendered_at_fit(&ShownPhoto::default());

    let cropped = rendered_at_fit(&framed([0.5, 0.625], [0.5, 0.5]));

    assert_eq!((cropped.width, cropped.height), (32, 16));
    let held: Vec<&[u8]> = rows_of(&whole)[12..28]
        .iter()
        .map(|row| &row[16 * 4..48 * 4])
        .collect();
    assert_eq!(rows_of(&cropped), held);
    assert_matches_golden("framing_frame", &cropped);
}

#[test]
fn the_default_framing_shows_the_whole_picture() {
    let whole = rendered_at_fit(&ShownPhoto::default());

    assert_eq!((whole.width, whole.height), (64, 32));
    assert_eq!(whole, rendered_at_fit(&framed([0.5, 0.5], [1.0, 1.0])));
}

const LINE_TILT: f32 = 20.0;

/// A white line two pixels wide through the middle of a black picture,
/// going down to the right by `LINE_TILT` degrees.
fn tilted_line() -> WorkingImage {
    let [width, height] = PICTURE;
    let (sine, cosine) = LINE_TILT.to_radians().sin_cos();
    let pixels = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| {
            let from_middle = [
                x as f32 + 0.5 - width as f32 / 2.0,
                y as f32 + 0.5 - height as f32 / 2.0,
            ];
            let from_line = (from_middle[1] * cosine - from_middle[0] * sine).abs();
            [(1.5 - from_line).clamp(0.0, 1.0); 3]
        })
        .collect();
    WorkingImage::new(width, height, pixels)
}

#[test]
fn a_straightened_line_is_level_fine_and_without_steps() {
    let levelled = ShownPhoto {
        framing: framed([0.5, 0.5], [0.5, 0.5])
            .framing
            .straightened(PICTURE, -LINE_TILT),
        ..ShownPhoto::default()
    };

    let pixels = picture_rendered_at_fit(&tilted_line(), &levelled);

    assert_eq!((pixels.width, pixels.height), (32, 16));
    let rows = rows_of(&pixels);
    let column = |x: usize| -> Vec<u8> { rows.iter().map(|row| row[x * 4]).collect() };
    // The two rows the line runs along, and the rows its soft edges may reach.
    let (heart, edges) = (7..=8, 6..=9);
    for x in 0..pixels.width as usize {
        for (y, level) in column(x).into_iter().enumerate() {
            let is_as_expected = match (heart.contains(&y), edges.contains(&y)) {
                (true, _) => level > 200,
                (false, true) => true,
                (false, false) => level < 10,
            };
            assert!(is_as_expected, "column {x}, row {y}: {level}");
        }
    }
    assert_matches_golden("framing_straightened_line", &pixels);
}

/// The framed photo one screen pixel per pixel of the picture.
fn rendered_whole(shown: &ShownPhoto) -> DisplayPixels {
    let engine = headless_engine();
    let source = engine.upload(&positions());
    let request = shown.request_of(
        shown.framing.region(PICTURE),
        shown.framing.framed_size(PICTURE),
    );
    let output = engine.render_display(&source, &request);
    engine.read_display_pixels(&output).unwrap()
}

fn turned(turn: Turn) -> ShownPhoto {
    ShownPhoto {
        framing: Framing {
            turn,
            ..Framing::default()
        },
        ..ShownPhoto::default()
    }
}

fn pixel(pixels: &DisplayPixels, [x, y]: [usize; 2]) -> &[u8] {
    let start = (y * pixels.width as usize + x) * 4;
    &pixels.rgba[start..start + 4]
}

#[test]
fn a_photo_turned_right_has_the_left_of_its_picture_at_the_top() {
    let whole = rendered_whole(&ShownPhoto::default());

    let right = rendered_whole(&turned(Turn::default().turned_right()));

    assert_eq!((right.width, right.height), (32, 64));
    for (x, y) in (0..32).flat_map(|x| (0..64).map(move |y| (x, y))) {
        assert_eq!(
            pixel(&right, [x, y]),
            pixel(&whole, [y, 31 - x]),
            "{x}, {y}"
        );
    }
}

#[test]
fn a_photo_flipped_horizontally_has_the_left_of_its_picture_on_the_right() {
    let whole = rendered_whole(&ShownPhoto::default());

    let flipped = rendered_whole(&turned(Turn::default().flipped_horizontally()));

    assert_eq!((flipped.width, flipped.height), (64, 32));
    for (x, y) in (0..64).flat_map(|x| (0..32).map(move |y| (x, y))) {
        assert_eq!(
            pixel(&flipped, [x, y]),
            pixel(&whole, [63 - x, y]),
            "{x}, {y}"
        );
    }
}

#[test]
fn four_quarter_turns_and_two_mirrors_give_the_photo_back() {
    let whole = rendered_whole(&ShownPhoto::default());
    let right = Turn::default().turned_right();
    let around = right.turned_right().turned_right().turned_right();
    let flipped_twice = right.flipped_vertically().flipped_vertically();

    assert_eq!(rendered_whole(&turned(around)), whole);
    assert_eq!(
        rendered_whole(&turned(flipped_twice)),
        rendered_whole(&turned(right))
    );
}

#[test]
fn a_turned_and_mirrored_frame_keeps_its_part_of_the_picture() {
    let shown = ShownPhoto {
        framing: Framing {
            turn: Turn::default().turned_left().flipped_vertically(),
            ..framed([0.5, 0.625], [0.5, 0.5]).framing
        },
        ..ShownPhoto::default()
    };

    let pixels = rendered_whole(&shown);

    assert_eq!((pixels.width, pixels.height), (16, 32));
    assert_matches_golden("framing_turned_and_mirrored", &pixels);
}

/// A zone found on the left half of the picture, as a detection stored it.
fn zone_on_the_left() -> MaskShape {
    let [width, height] = coverage_image_size(PICTURE);
    let values = (0..height)
        .flat_map(|_| (0..width).map(|column| if column < width / 2 { 255 } else { 0 }))
        .collect();
    MaskShape::Zone(ZoneMask {
        zone: Zone::Sky,
        coverage: Arc::new(CoverageImage::new([width, height], values).unwrap()),
    })
}

/// A gradient down the picture and a zone, both darkening, over an enhancement.
fn retouched(framing: Framing) -> ShownPhoto {
    let gradient = MaskShape::LinearGradient(LinearGradient {
        full: [0.5, 0.1],
        none: [0.5, 0.4],
    });
    let masks = [gradient, zone_on_the_left()].map(|shape| Mask {
        adjustments: Adjustments {
            exposure: -1.0,
            ..Adjustments::default()
        },
        ..Mask::of(shape)
    });
    ShownPhoto {
        development: Development {
            edit: Edit {
                masks: masks.to_vec(),
                enhancement_intensity: 100.0,
                framing,
                ..Edit::default()
            },
            ..Development::default()
        },
        framing,
        ..ShownPhoto::default()
    }
}

/// The retouched photo one screen pixel per pixel of the picture, its
/// enhancement being the picture with red and green swapped.
fn rendered_retouched(framing: Framing) -> DisplayPixels {
    let engine = headless_engine();
    let picture = positions();
    let source = engine.upload(&picture);
    let swapped = picture
        .pixels()
        .iter()
        .map(|[red, green, blue]| [*green, *red, *blue]);
    engine.upload_enhancement(
        &source,
        &WorkingImage::new(PICTURE[0], PICTURE[1], swapped.collect()),
    );
    let shown = retouched(framing);
    let request = shown.request_of(framing.region(PICTURE), framing.framed_size(PICTURE));
    engine
        .read_display_pixels(&engine.render_display(&source, &request))
        .unwrap()
}

#[test]
fn masks_a_zone_and_an_enhancement_stay_on_the_picture_whatever_the_framing() {
    let whole = rendered_retouched(Framing::default());
    let framing = Framing {
        turn: Turn::default().turned_right(),
        ..framed([0.5, 0.625], [0.5, 0.5]).framing
    };

    let reframed = rendered_retouched(framing);

    assert_eq!((reframed.width, reframed.height), (16, 32));
    assert_ne!(whole, rendered_whole(&ShownPhoto::default()));
    for (x, y) in (0..16).flat_map(|x| (0..32).map(move |y| (x, y))) {
        let on_picture = pixel(&whole, [16 + y, 12 + 15 - x]);
        let is_same = pixel(&reframed, [x, y])
            .iter()
            .zip(on_picture)
            .all(|(reframed, whole)| reframed.abs_diff(*whole) <= 1);
        assert!(
            is_same,
            "{x}, {y}: {:?} is not {on_picture:?}",
            pixel(&reframed, [x, y])
        );
    }
}
