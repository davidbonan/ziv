use std::sync::{Mutex, MutexGuard, OnceLock};

use half::f16;

use crate::photo::domain::working_image::WorkingImage;

use super::coverage_layers::CoverageLayers;

pub(super) const SOURCE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const CHANNELS: u32 = 4;
const BYTES_PER_PIXEL: u32 = CHANNELS * size_of::<f16>() as u32;

/// A working image on the GPU, still in the linear working space.
pub(super) struct WorkingTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

fn mip_level_count(width: u32, height: u32) -> u32 {
    width.max(height).ilog2() + 1
}

impl WorkingTexture {
    /// Writes the full-resolution level only; the mip chain is left to fill.
    pub fn upload(device: &wgpu::Device, queue: &wgpu::Queue, image: &WorkingImage) -> Self {
        let size = wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ziv source"),
            size,
            mip_level_count: mip_level_count(image.width(), image.height()),
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: SOURCE_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let texels: Vec<f16> = image
            .pixels()
            .iter()
            .flat_map(|[red, green, blue]| [*red, *green, *blue, 1.0].map(f16::from_f32))
            .collect();
        queue.write_texture(
            texture.as_image_copy(),
            bytemuck::cast_slice(&texels),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(image.width() * BYTES_PER_PIXEL),
                rows_per_image: None,
            },
            size,
        );
        Self {
            view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
            texture,
        }
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

/// A photo on the GPU: its working image, its enhancement once it has one,
/// and the coverage images of its masks as last rendered.
pub struct SourceTexture {
    original: WorkingTexture,
    enhancement: OnceLock<WorkingTexture>,
    size: [u32; 2],
    coverage_layers: Mutex<CoverageLayers>,
}

impl SourceTexture {
    pub(super) fn of(original: WorkingTexture, size: [u32; 2]) -> Self {
        Self {
            original,
            enhancement: OnceLock::new(),
            size,
            coverage_layers: Mutex::default(),
        }
    }

    pub fn size(&self) -> [u32; 2] {
        self.size
    }

    pub fn is_enhanced(&self) -> bool {
        self.enhancement.get().is_some()
    }

    /// A photo is enhanced once: a later enhancement is left aside.
    pub(super) fn set_enhancement(&self, enhancement: WorkingTexture) {
        let _ = self.enhancement.set(enhancement);
    }

    pub(super) fn view(&self) -> &wgpu::TextureView {
        self.original.view()
    }

    /// What the enhancement intensity mixes in; the original itself on a
    /// photo without enhancement.
    pub(super) fn enhancement_view(&self) -> &wgpu::TextureView {
        self.enhancement.get().unwrap_or(&self.original).view()
    }

    pub(super) fn coverage_layers(&self) -> MutexGuard<'_, CoverageLayers> {
        self.coverage_layers
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
