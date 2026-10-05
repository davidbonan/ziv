use std::sync::Arc;

use eframe::egui_wgpu::{RenderState, Renderer};
use egui::mutex::RwLock;

use crate::develop::domain::development::Development;
use crate::develop::domain::mask::Mask;
use crate::engine::infrastructure::display_stage::{DisplayRequest, Sampling};
use crate::engine::infrastructure::engine::Engine;
use crate::engine::infrastructure::source_texture::SourceTexture;
use crate::viewport::domain::view::{ACTUAL_SIZE, Placement};

/// How a photo is shown: developed, and under the veil of a mask when one is
/// being looked at.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ShownPhoto {
    pub development: Development,
    pub overlaid_mask: Option<Mask>,
}

struct DisplayOutput {
    placement: Placement,
    shown: ShownPhoto,
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

    /// Renders what `placement` shows of the developed photo, unless its latest
    /// render already is that.
    pub fn render_at(
        &self,
        photo: &mut PresentedPhoto,
        placement: &Placement,
        shown: &ShownPhoto,
    ) -> egui::TextureId {
        let is_enhanced = photo.source.is_enhanced();
        let is_latest = |output: &&DisplayOutput| {
            output.placement == *placement
                && output.shown == *shown
                && output.is_enhanced == is_enhanced
        };
        if let Some(output) = photo.output.as_ref().filter(is_latest) {
            return output.texture_id;
        }

        let view = self
            .engine
            .render_display(
                &photo.source,
                &display_request(photo.size(), placement, shown),
            )
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
            placement: *placement,
            shown: shown.clone(),
            is_enhanced,
            texture_id,
        });
        texture_id
    }
}

fn display_request(
    photo_size: [u32; 2],
    placement: &Placement,
    shown: &ShownPhoto,
) -> DisplayRequest {
    let photo_size = photo_size.map(|side| side as f32);
    let in_texture_coordinates =
        |position: [f32; 2]| [0, 1].map(|axis| position[axis] / photo_size[axis]);
    DisplayRequest {
        region_min: in_texture_coordinates(placement.photo_min),
        region_size: in_texture_coordinates(placement.photo_size),
        size: placement.screen_size.map(|side| side.ceil() as u32),
        sampling: if placement.scale > ACTUAL_SIZE {
            Sampling::Pixelated
        } else {
            Sampling::Smooth
        },
        development: shown.development.clone(),
        overlaid_mask: shown.overlaid_mask.clone(),
    }
}
