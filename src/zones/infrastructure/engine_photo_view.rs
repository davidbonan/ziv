use std::sync::Arc;

use crate::develop::domain::development::Development;
use crate::develop::domain::edit::Edit;
use crate::engine::infrastructure::display_stage::DisplayRequest;
use crate::engine::infrastructure::engine::Engine;
use crate::engine::infrastructure::source_texture::SourceTexture;
use crate::photo::domain::photo_kind::PhotoKind;
use crate::photo::domain::picture_region::PictureRegion;
use crate::zones::domain::photo_view::{PhotoRegion, PhotoView};

/// An uploaded photo, rendered by the engine without its edit.
pub struct EnginePhotoView {
    pub engine: Arc<Engine>,
    pub source: Arc<SourceTexture>,
    pub kind: PhotoKind,
}

impl PhotoView for EnginePhotoView {
    fn photo_size(&self) -> [u32; 2] {
        self.source.size()
    }

    fn pixels(&self, region: &PhotoRegion, size: [u32; 2]) -> Result<Vec<u8>, String> {
        let request = DisplayRequest {
            region: PictureRegion::upright(region.min, region.size),
            development: Development {
                kind: self.kind,
                edit: Edit::default(),
            },
            ..DisplayRequest::whole_source(size)
        };
        let rendered = self.engine.render_display(&self.source, &request);
        self.engine
            .read_display_pixels(&rendered)
            .map(|pixels| pixels.rgba)
            .map_err(|error| error.to_string())
    }
}
