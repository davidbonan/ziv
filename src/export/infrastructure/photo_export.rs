use std::path::{Path, PathBuf};

use crate::develop::domain::development::Development;
use crate::develop::domain::edit_storage::EditStorage;
use crate::develop::infrastructure::sidecar_files::SidecarFiles;
use crate::engine::infrastructure::display_stage::DisplayRequest;
use crate::engine::infrastructure::engine::Engine;
use crate::enhance::infrastructure::enhanced_upload::upload_with_enhancement;
use crate::export::domain::export_settings::ExportSettings;
use crate::photo::infrastructure::file_decoder::FileDecoder;

use super::exported_file::write_exported_file;

/// Decodes a photo, develops and frames it with its stored edit and its
/// enhancement, and writes the exported file. A stored edit that cannot be used exports the
/// photo without edit.
pub fn export_photo(
    engine: &Engine,
    photo: &Path,
    settings: &ExportSettings,
) -> Result<PathBuf, String> {
    let decoded = FileDecoder
        .decode(photo)
        .map_err(|error| error.to_string())?;
    let development = Development {
        kind: decoded.kind,
        edit: SidecarFiles
            .stored_edit(photo)
            .ok()
            .flatten()
            .unwrap_or_default(),
    };
    let image = &decoded.image;
    let picture = [image.width(), image.height()];
    let framing = development.edit.framing;
    let source = match development.edit.enhancement_share() > 0.0 {
        true => upload_with_enhancement(engine, photo, image),
        false => engine.upload(image),
    };
    let request = DisplayRequest {
        region: framing.region(picture),
        development,
        ..DisplayRequest::whole_source(settings.output_size(framing.framed_size(picture)))
    };
    let pixels = engine
        .render_pixels(&source, &request)
        .map_err(|error| error.to_string())?;
    write_exported_file(&pixels, photo, settings)
}
