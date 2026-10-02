use std::path::Path;

use crate::engine::infrastructure::engine::Engine;
use crate::engine::infrastructure::source_texture::SourceTexture;
use crate::enhance::domain::enhancement_storage::EnhancementStorage;
use crate::photo::domain::working_image::WorkingImage;

use super::enhancement_files::EnhancementFiles;

/// `image` on the GPU, with the enhancement kept beside `photo` when there
/// is one. An enhancement file that cannot be used is left aside.
pub fn upload_with_enhancement(
    engine: &Engine,
    photo: &Path,
    image: &WorkingImage,
) -> SourceTexture {
    let source = engine.upload(image);
    if let Ok(Some(enhanced)) = EnhancementFiles.stored_enhanced_image(photo, image) {
        engine.upload_enhancement(&source, &enhanced);
    }
    source
}
