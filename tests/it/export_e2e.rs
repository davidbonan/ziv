use std::fs;
use std::path::{Path, PathBuf};
use ziv::develop::domain::adjustments::Adjustments;

use ziv::develop::domain::edit::Edit;
use ziv::develop::domain::edit_storage::EditStorage;
use ziv::develop::domain::framing::{CropFrame, Framing, Turn};
use ziv::develop::infrastructure::sidecar_files::SidecarFiles;
use ziv::enhance::domain::enhancement::Enhancement;
use ziv::enhance::domain::enhancement_storage::EnhancementStorage;
use ziv::enhance::domain::model_encoding::ModelEncoding;
use ziv::enhance::infrastructure::enhancement_files::EnhancementFiles;
use ziv::export::domain::export_settings::{ExportFormat, ExportSettings, ExportSize};
use ziv::export::infrastructure::photo_export::export_photo;

use crate::gpu::headless_engine;

struct Shoot {
    folder: tempfile::TempDir,
}

impl Shoot {
    fn with(fixtures: &[&str]) -> Self {
        let folder = tempfile::tempdir().unwrap();
        fs::create_dir(folder.path().join("exports")).unwrap();
        for fixture in fixtures {
            let source = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(fixture);
            fs::copy(source, folder.path().join(fixture)).unwrap();
        }
        Self { folder }
    }

    fn photo(&self, name: &str) -> PathBuf {
        self.folder.path().join(name)
    }

    fn settings(&self, format: ExportFormat) -> ExportSettings {
        ExportSettings {
            format,
            destination: Some(self.folder.path().join("exports")),
            ..ExportSettings::default()
        }
    }

    fn exported_names(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(self.folder.path().join("exports"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }
}

const WHITE_PATCH: (u32, u32) = (8, 24);

fn white_patch_of(exported: &Path) -> u8 {
    image::open(exported)
        .unwrap()
        .into_rgb8()
        .get_pixel(WHITE_PATCH.0, WHITE_PATCH.1)[0]
}

#[test]
fn png_export_is_the_photo_pixel_for_pixel() {
    let shoot = Shoot::with(&["patches.png"]);

    let exported = export_photo(
        &headless_engine(),
        &shoot.photo("patches.png"),
        &shoot.settings(ExportFormat::Png),
    )
    .unwrap();

    assert_eq!(shoot.exported_names(), ["patches.png"]);
    let source = image::open(shoot.photo("patches.png")).unwrap().into_rgb8();
    let written = image::open(exported).unwrap().into_rgb8();
    assert_eq!(written.dimensions(), source.dimensions());
    let is_same = written
        .iter()
        .zip(source.iter())
        .all(|(written, source)| written.abs_diff(*source) <= 1);
    assert!(is_same);
}

#[test]
fn jpeg_export_is_a_jpeg_of_the_photo_size() {
    let shoot = Shoot::with(&["patches.png"]);

    let exported = export_photo(
        &headless_engine(),
        &shoot.photo("patches.png"),
        &shoot.settings(ExportFormat::Jpeg),
    )
    .unwrap();

    assert_eq!(shoot.exported_names(), ["patches.jpg"]);
    let source = image::image_dimensions(shoot.photo("patches.png")).unwrap();
    assert_eq!(image::image_dimensions(&exported).unwrap(), source);
    assert_eq!(
        image::ImageFormat::from_path(&exported).unwrap(),
        image::ImageFormat::Jpeg
    );
}

#[test]
fn exporting_again_never_overwrites() {
    let shoot = Shoot::with(&["patches.png"]);
    let engine = headless_engine();
    let settings = shoot.settings(ExportFormat::Png);

    for _ in 0..3 {
        export_photo(&engine, &shoot.photo("patches.png"), &settings).unwrap();
    }

    assert_eq!(
        shoot.exported_names(),
        ["patches-1.png", "patches-2.png", "patches.png"]
    );
}

#[test]
fn export_develops_the_photo_with_its_stored_edit() {
    let shoot = Shoot::with(&["patches.png"]);
    let photo = shoot.photo("patches.png");
    let engine = headless_engine();
    let settings = shoot.settings(ExportFormat::Png);
    let untouched = export_photo(&engine, &photo, &settings).unwrap();
    let darker = Edit::from(Adjustments {
        exposure: -2.0,
        ..Adjustments::default()
    });
    SidecarFiles.store_edit(&photo, &darker).unwrap();

    let developed = export_photo(&engine, &photo, &settings).unwrap();

    assert_eq!(white_patch_of(&untouched), 255);
    assert!(white_patch_of(&developed) < 160);
}

/// An enhancement that darkens every channel of every pixel by 40 levels.
fn store_darkening_enhancement(photo: &Path) {
    let (width, height) = image::image_dimensions(photo).unwrap();
    let levels = vec![88; (width * height * 3) as usize];
    let encoding = ModelEncoding::with_ceiling(1.0);
    let darkening = Enhancement::new([width, height], encoding, levels).unwrap();
    EnhancementFiles
        .store_enhancement(photo, &darkening)
        .unwrap();
}

#[test]
fn export_applies_the_enhancement_at_the_intensity_of_the_edit() {
    let shoot = Shoot::with(&["patches.png"]);
    let photo = shoot.photo("patches.png");
    let engine = headless_engine();
    let settings = shoot.settings(ExportFormat::Png);
    store_darkening_enhancement(&photo);
    let exported_at = |enhancement_intensity| {
        let edit = Edit {
            enhancement_intensity,
            ..Edit::default()
        };
        SidecarFiles.store_edit(&photo, &edit).unwrap();
        white_patch_of(&export_photo(&engine, &photo, &settings).unwrap())
    };

    let [without, halfway, full] = [0.0, 50.0, 100.0].map(exported_at);

    assert_eq!(without, 255);
    assert!(
        full < halfway && halfway < without,
        "{full} {halfway} {without}"
    );
}

#[test]
fn export_of_a_photo_whose_enhancement_file_is_gone_is_the_photo() {
    let shoot = Shoot::with(&["patches.png"]);
    let photo = shoot.photo("patches.png");
    let enhanced_once = Edit {
        enhancement_intensity: 100.0,
        ..Edit::default()
    };
    SidecarFiles.store_edit(&photo, &enhanced_once).unwrap();

    let settings = shoot.settings(ExportFormat::Png);
    let exported = export_photo(&headless_engine(), &photo, &settings).unwrap();

    assert_eq!(white_patch_of(&exported), 255);
}

#[test]
fn long_edge_export_is_smaller_and_keeps_the_aspect_ratio() {
    let shoot = Shoot::with(&["patches.tiff"]);
    let (width, height) = image::image_dimensions(shoot.photo("patches.tiff")).unwrap();
    let settings = ExportSettings {
        size: ExportSize::LongEdge,
        long_edge: width.max(height) / 2,
        ..shoot.settings(ExportFormat::Png)
    };

    let exported =
        export_photo(&headless_engine(), &shoot.photo("patches.tiff"), &settings).unwrap();

    assert_eq!(
        image::image_dimensions(exported).unwrap(),
        (width / 2, height / 2)
    );
}

#[test]
fn photo_that_cannot_be_decoded_exports_nothing() {
    let shoot = Shoot::with(&[]);
    fs::write(shoot.photo("broken.jpg"), "not a jpeg").unwrap();

    let result = export_photo(
        &headless_engine(),
        &shoot.photo("broken.jpg"),
        &shoot.settings(ExportFormat::Jpeg),
    );

    assert!(result.is_err());
    assert!(shoot.exported_names().is_empty());
}

/// The right half of the picture, turned right.
fn right_half_turned_right() -> Edit {
    Edit {
        framing: Framing {
            frame: CropFrame {
                centre: [0.75, 0.5],
                size: [0.5, 1.0],
            },
            turn: Turn::default().turned_right(),
            ..Framing::default()
        },
        ..Edit::default()
    }
}

#[test]
fn export_of_a_framed_photo_is_its_frame_pixel_for_pixel() {
    let shoot = Shoot::with(&["patches.png"]);
    let photo = shoot.photo("patches.png");
    SidecarFiles
        .store_edit(&photo, &right_half_turned_right())
        .unwrap();

    let exported = export_photo(
        &headless_engine(),
        &photo,
        &shoot.settings(ExportFormat::Png),
    )
    .unwrap();

    let source = image::open(&photo).unwrap().into_rgb8();
    let (width, height) = source.dimensions();
    let written = image::open(exported).unwrap().into_rgb8();
    assert_eq!(written.dimensions(), (height, width / 2));
    for (x, y, pixel) in written.enumerate_pixels() {
        let on_picture = source.get_pixel(width / 2 + y, height - 1 - x);
        let is_same = (0..3).all(|channel| pixel[channel].abs_diff(on_picture[channel]) <= 1);
        assert!(is_same, "{x}, {y}: {pixel:?} is not {on_picture:?}");
    }
}

#[test]
fn long_edge_export_of_a_framed_photo_scales_its_frame() {
    let shoot = Shoot::with(&["patches.png"]);
    let photo = shoot.photo("patches.png");
    let (width, height) = image::image_dimensions(&photo).unwrap();
    SidecarFiles
        .store_edit(&photo, &right_half_turned_right())
        .unwrap();
    let framed_long_edge = height.max(width / 2);
    let settings = ExportSettings {
        size: ExportSize::LongEdge,
        long_edge: framed_long_edge / 2,
        ..shoot.settings(ExportFormat::Png)
    };

    let exported = export_photo(&headless_engine(), &photo, &settings).unwrap();

    assert_eq!(
        image::image_dimensions(exported).unwrap(),
        (height / 2, width / 4)
    );
}
