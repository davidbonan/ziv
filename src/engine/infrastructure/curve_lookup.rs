use half::f16;

use crate::develop::domain::tone_curve::{LOOKUP_SIZE, ToneCurves};

const LOOKUP_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
const BYTES_PER_TEXEL: u32 = 4 * size_of::<f16>() as u32;
const LOOKUP_EXTENT: wgpu::Extent3d = wgpu::Extent3d {
    width: LOOKUP_SIZE as u32,
    height: 1,
    depth_or_array_layers: 1,
};

/// The tone curves of the photo being rendered, as one row of texels the
/// display stage looks each channel up in.
pub(super) struct CurveLookup {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl CurveLookup {
    pub fn new(device: &wgpu::Device) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ziv tone curve lookup"),
            size: LOOKUP_EXTENT,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: LOOKUP_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        Self {
            view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
            texture,
        }
    }

    pub fn write(&self, queue: &wgpu::Queue, curves: &ToneCurves) {
        let texels: Vec<f16> = curves
            .lookup()
            .into_iter()
            .flat_map(|[red, green, blue]| [red, green, blue, 1.0].map(f16::from_f32))
            .collect();
        queue.write_texture(
            self.texture.as_image_copy(),
            bytemuck::cast_slice(&texels),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(LOOKUP_SIZE as u32 * BYTES_PER_TEXEL),
                rows_per_image: None,
            },
            LOOKUP_EXTENT,
        );
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}
