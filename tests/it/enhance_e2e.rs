use std::ops::ControlFlow;

use ziv::enhance::application::enhancer::Enhancer;
use ziv::enhance::domain::enhancement_model::{ENHANCEMENT_MODEL, TILE_SHAPE};
use ziv::models::application::model_store::ModelStore;
use ziv::models::infrastructure::model_downloads::ModelDownloads;
use ziv::models::infrastructure::models_folder::models_folder;
use ziv::models::infrastructure::onnx_runner::OnnxRunner;
use ziv::photo::domain::working_image::WorkingImage;

const WIDTH: u32 = 600;
const HEIGHT: u32 = 300;
const GREY: f32 = 0.2;

/// Tests never download: an enhancer only when the model is already here.
fn local_enhancer() -> Option<Enhancer<ModelDownloads, OnnxRunner>> {
    let folder = models_folder()?;
    if !folder.join(ENHANCEMENT_MODEL.file_name()).is_file() {
        eprintln!("skipped: the enhancement model is not on this machine");
        return None;
    }
    let store = ModelStore::new(folder, ModelDownloads);
    Some(Enhancer::new(store, OnnxRunner::on_gpu(&TILE_SHAPE)))
}

/// A flat grey under noise that differs at every pixel and channel, the same at each run.
fn noisy_grey() -> WorkingImage {
    let mut state = 0x2545_f491_u32;
    let mut noise = || {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (state >> 8) as f32 / (1 << 24) as f32 - 0.5
    };
    let pixels = (0..WIDTH * HEIGHT).map(|_| [(); 3].map(|()| GREY + 0.12 * noise()));
    WorkingImage::new(WIDTH, HEIGHT, pixels.collect())
}

fn spread_around_grey(image: &WorkingImage) -> f32 {
    let channels = image.pixels().iter().flatten();
    let squares: f32 = channels.map(|value| (value - GREY).powi(2)).sum();
    (squares / (3 * WIDTH * HEIGHT) as f32).sqrt()
}

#[test]
fn noise_of_a_flat_grey_is_removed_and_the_grey_kept() {
    let Some(enhancer) = local_enhancer() else {
        return;
    };
    let noisy = noisy_grey();

    let enhancement = enhancer
        .enhancement(&noisy, &mut |_| ControlFlow::Continue(()))
        .unwrap();

    let enhanced = enhancement.enhanced(&noisy).unwrap();
    let [before, after] = [&noisy, &enhanced].map(spread_around_grey);
    assert!(after < before / 4.0, "noise {before} became {after}");
}
