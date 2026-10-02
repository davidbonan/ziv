use std::path::Path;

use crate::enhance::application::enhancer::{EnhancementError, Enhancer, OnStep};
use crate::enhance::domain::enhancement_storage::EnhancementStorage;
use crate::models::domain::model_runner::ModelRunner;
use crate::models::domain::model_source::ModelSource;
use crate::photo::domain::working_image::WorkingImage;
use crate::photo::infrastructure::file_decoder::FileDecoder;

use super::enhancement_files::EnhancementFiles;

/// Decodes a photo, enhances it and writes its enhancement file. Returns
/// the enhanced photo; nothing is written when it fails or is cancelled.
pub fn enhance_photo<Source: ModelSource, Runner: ModelRunner>(
    enhancer: &Enhancer<Source, Runner>,
    photo: &Path,
    on_step: OnStep<'_>,
) -> Result<WorkingImage, EnhancementError> {
    let decoded = FileDecoder
        .decode(photo)
        .map_err(|error| EnhancementError::Photo(error.to_string()))?;
    let enhancement = enhancer.enhancement(&decoded.image, on_step)?;
    EnhancementFiles
        .store_enhancement(photo, &enhancement)
        .map_err(EnhancementError::Storage)?;
    let enhanced = enhancement.enhanced(&decoded.image);
    Ok(enhanced.expect("the enhancement is the one of this photo"))
}
