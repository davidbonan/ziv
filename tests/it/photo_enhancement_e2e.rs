use std::fs;
use std::io::Read;
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};

use ziv::enhance::application::enhancer::{EnhancementError, Enhancer};
use ziv::enhance::domain::enhancement_model::ENHANCEMENT_MODEL;
use ziv::enhance::domain::enhancement_storage::EnhancementStorage;
use ziv::enhance::infrastructure::enhancement_files::{EnhancementFiles, enhancement_path};
use ziv::enhance::infrastructure::photo_enhancement::enhance_photo;
use ziv::models::application::model_store::ModelStore;
use ziv::models::domain::model::RemoteFile;
use ziv::models::domain::model_runner::{ModelInput, ModelOutput, ModelRunner};
use ziv::models::domain::model_source::ModelSource;
use ziv::photo::infrastructure::file_decoder::FileDecoder;

struct NoNetwork;

impl ModelSource for NoNetwork {
    fn download(&self, _: &RemoteFile) -> Result<Box<dyn Read>, String> {
        Err("no network".to_owned())
    }
}

/// Answers every tile a little darker than it was shown.
struct Darkening;

impl ModelRunner for Darkening {
    fn outputs(&self, _: &Path, input: ModelInput) -> Result<Vec<ModelOutput>, String> {
        let [_, _, height, width] = input.shape;
        let photo_planes = &input.values[..3 * height * width];
        Ok(vec![ModelOutput {
            shape: vec![1, 3, height, width],
            values: photo_planes.iter().map(|value| value - 0.1).collect(),
        }])
    }
}

struct Shoot {
    folder: tempfile::TempDir,
}

impl Shoot {
    fn with_patches() -> Self {
        let folder = tempfile::tempdir().unwrap();
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/patches.png");
        fs::copy(fixture, folder.path().join("patches.png")).unwrap();
        let models = folder.path().join("models");
        fs::create_dir(&models).unwrap();
        fs::write(models.join(ENHANCEMENT_MODEL.file_name()), b"weights").unwrap();
        Self { folder }
    }

    fn photo(&self) -> PathBuf {
        self.folder.path().join("patches.png")
    }

    fn enhancer(&self) -> Enhancer<NoNetwork, Darkening> {
        let store = ModelStore::new(self.folder.path().join("models"), NoNetwork);
        Enhancer::new(store, Darkening)
    }

    fn file_names(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(self.folder.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }
}

#[test]
fn enhanced_photo_is_kept_beside_the_photo_and_found_again() {
    let shoot = Shoot::with_patches();
    let photo = shoot.photo();

    let enhanced = enhance_photo(
        &shoot.enhancer(),
        &photo,
        &mut |_| ControlFlow::Continue(()),
    );

    let original = FileDecoder.decode(&photo).unwrap().image;
    let found_again = EnhancementFiles.stored_enhanced_image(&photo, &original);
    assert!(enhancement_path(&photo).is_file());
    assert_eq!(found_again, Ok(Some(enhanced.unwrap())));
}

#[test]
fn cancelled_enhancement_writes_nothing() {
    let shoot = Shoot::with_patches();

    let enhanced = enhance_photo(&shoot.enhancer(), &shoot.photo(), &mut |_| {
        ControlFlow::Break(())
    });

    assert_eq!(enhanced, Err(EnhancementError::Cancelled));
    assert_eq!(shoot.file_names(), ["models", "patches.png"]);
}

#[test]
fn photo_that_cannot_be_read_is_reported() {
    let shoot = Shoot::with_patches();
    let missing = shoot.folder.path().join("gone.png");

    let enhanced = enhance_photo(&shoot.enhancer(), &missing, &mut |_| {
        ControlFlow::Continue(())
    });

    assert!(matches!(enhanced, Err(EnhancementError::Photo(_))));
}
