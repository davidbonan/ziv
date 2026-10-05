use std::sync::Arc;

use eframe::egui_wgpu::{RenderState, Renderer};
use egui::mutex::RwLock;

use crate::develop::domain::development::Development;
use crate::develop::domain::framing::Framing;
use crate::develop::domain::mask::Mask;
use crate::engine::infrastructure::display_stage::{DisplayRequest, Sampling};
use crate::engine::infrastructure::engine::Engine;
use crate::engine::infrastructure::source_texture::SourceTexture;
use crate::photo::domain::picture_region::PictureRegion;
use crate::viewport::domain::view::{ACTUAL_SIZE, Placement};

/// How a photo is shown: developed, framed, and under the veil of a mask when
/// one is being looked at.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ShownPhoto {
    pub development: Development,
    pub framing: Framing,
    pub overlaid_mask: Option<Mask>,
}

impl ShownPhoto {
    /// `region` of the picture at `size`, whatever the framing.
    pub fn request_of(&self, region: PictureRegion, size: [u32; 2]) -> DisplayRequest {
        DisplayRequest {
            region,
            development: self.development.clone(),
            overlaid_mask: self.overlaid_mask.clone(),
            ..DisplayRequest::whole_source(size)
        }
    }

    /// What `placement` shows of the framed photo, `picture` being the size of its picture.
    pub fn request_at(&self, picture: [u32; 2], placement: &Placement) -> DisplayRequest {
        let framed = self.framing.framed_size(picture).map(|side| side as f32);
        let in_shares = |position: [f32; 2]| [0, 1].map(|axis| position[axis] / framed[axis]);
        let shown = in_shares(placement.photo_size);
        let region = self.framing.region(picture);
        // A photo with an angle is resampled: its pixels are not the picture's.
        let is_magnified_picture = placement.scale > ACTUAL_SIZE && self.framing.angle == 0.0;
        DisplayRequest {
            sampling: match is_magnified_picture {
                true => Sampling::Pixelated,
                false => Sampling::Smooth,
            },
            ..self.request_of(
                region.part(in_shares(placement.photo_min), shown),
                placement.screen_size.map(|side| side.ceil() as u32),
            )
        }
    }
}

struct DisplayOutput {
    request: DisplayRequest,
    // An enhancement arrives after the photo: what was rendered without it is outdated.
    is_enhanced: bool,
    texture_id: egui::TextureId,
}

/// A photo on the GPU together with its latest render for the screen.
pub struct PresentedPhoto {
    source: Arc<SourceTexture>,
    output: Option<DisplayOutput>,
    egui_renderer: Arc<RwLock<Renderer>>,
}

impl Drop for PresentedPhoto {
    fn drop(&mut self) {
        if let Some(output) = &self.output {
            self.egui_renderer.write().free_texture(&output.texture_id);
        }
    }
}

impl PresentedPhoto {
    pub fn size(&self) -> [u32; 2] {
        self.source.size()
    }

    pub fn source(&self) -> &Arc<SourceTexture> {
        &self.source
    }
}

/// Bridges the engine's output to textures egui can paint.
pub struct PhotoPresenter {
    engine: Arc<Engine>,
    device: wgpu::Device,
    egui_renderer: Arc<RwLock<Renderer>>,
}

impl PhotoPresenter {
    pub fn new(render_state: &RenderState, engine: Arc<Engine>) -> Self {
        Self {
            engine,
            device: render_state.device.clone(),
            egui_renderer: render_state.renderer.clone(),
        }
    }

    pub fn present(&self, source: Arc<SourceTexture>) -> PresentedPhoto {
        PresentedPhoto {
            source,
            output: None,
            egui_renderer: self.egui_renderer.clone(),
        }
    }

    /// Renders `request` of the photo, unless its latest render already is that.
    pub fn render(&self, photo: &mut PresentedPhoto, request: DisplayRequest) -> egui::TextureId {
        let is_enhanced = photo.source.is_enhanced();
        let is_latest = |output: &&DisplayOutput| {
            output.request == request && output.is_enhanced == is_enhanced
        };
        if let Some(output) = photo.output.as_ref().filter(is_latest) {
            return output.texture_id;
        }

        let view = self
            .engine
            .render_display(&photo.source, &request)
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut egui_renderer = self.egui_renderer.write();
        let filter = wgpu::FilterMode::Nearest;
        let texture_id = match &photo.output {
            Some(previous) => {
                egui_renderer.update_egui_texture_from_wgpu_texture(
                    &self.device,
                    &view,
                    filter,
                    previous.texture_id,
                );
                previous.texture_id
            }
            None => egui_renderer.register_native_texture(&self.device, &view, filter),
        };
        photo.output = Some(DisplayOutput {
            request,
            is_enhanced,
            texture_id,
        });
        texture_id
    }
}
