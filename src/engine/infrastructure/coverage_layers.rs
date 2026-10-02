use std::mem;
use std::sync::Arc;

use crate::develop::domain::brush::BrushMask;
use crate::develop::domain::brush_coverage::{BrushCoverage, Texels};
use crate::develop::domain::coverage_image::{CoverageImage, coverage_image_size};
use crate::develop::domain::mask::CoverageSource;

const COVERAGE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

fn layers_texture(device: &wgpu::Device, size: [u32; 2], layers: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ziv mask coverages"),
        size: wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: layers,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: COVERAGE_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

pub(super) fn layers_view(texture: &wgpu::Texture) -> wgpu::TextureView {
    texture.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2Array),
        ..Default::default()
    })
}

/// A texture of one layer and one texel, for a photo without coverage image.
pub(super) fn no_coverage_layer(device: &wgpu::Device) -> wgpu::TextureView {
    layers_view(&layers_texture(device, [1, 1], 1))
}

enum Shown {
    Nothing,
    Painted(Box<BrushCoverage>),
    Detected(Arc<CoverageImage>),
}

struct Layers {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    size: [u32; 2],
    shown: Vec<Shown>,
}

/// The coverage of the masks of one photo that are rendered from an image,
/// one texture layer a mask, as last rendered.
#[derive(Default)]
pub struct CoverageLayers {
    layers: Option<Layers>,
}

struct WrittenTexels<'a> {
    layer: u32,
    texels: &'a Texels,
    // The whole image the texels are taken from, row after row.
    image: &'a [u8],
}

fn write_texels(queue: &wgpu::Queue, layers: &Layers, written: &WrittenTexels<'_>) {
    let WrittenTexels {
        layer,
        texels,
        image,
    } = written;
    if texels.is_empty() {
        return;
    }
    let width = layers.size[0] as usize;
    let columns = texels.columns.start as usize..texels.columns.end as usize;
    let values: Vec<u8> = texels
        .rows
        .clone()
        .flat_map(|row| {
            let row_start = row as usize * width;
            &image[row_start + columns.start..row_start + columns.end]
        })
        .copied()
        .collect();
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &layers.texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: texels.columns.start,
                y: texels.rows.start,
                z: *layer,
            },
            aspect: wgpu::TextureAspect::All,
        },
        &values,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(texels.columns.len() as u32),
            rows_per_image: None,
        },
        wgpu::Extent3d {
            width: texels.columns.len() as u32,
            height: texels.rows.len() as u32,
            depth_or_array_layers: 1,
        },
    );
}

impl Layers {
    fn everything(&self) -> Texels {
        Texels {
            columns: 0..self.size[0],
            rows: 0..self.size[1],
        }
    }

    fn show_strokes(&mut self, queue: &wgpu::Queue, layer: usize, mask: &BrushMask) {
        let (mut coverage, was_painted) = match mem::replace(&mut self.shown[layer], Shown::Nothing)
        {
            Shown::Painted(coverage) => (coverage, true),
            _ => (Box::new(BrushCoverage::of_size(self.size)), false),
        };
        let repainted = coverage.show(mask);
        let texels = match was_painted {
            true => repainted,
            false => Some(coverage.everything()),
        };
        if let Some(texels) = texels {
            let written = WrittenTexels {
                layer: layer as u32,
                texels: &texels,
                image: coverage.values(),
            };
            write_texels(queue, self, &written);
        }
        self.shown[layer] = Shown::Painted(coverage);
    }

    fn show_image(&mut self, queue: &wgpu::Queue, layer: usize, image: &Arc<CoverageImage>) {
        let is_shown =
            matches!(&self.shown[layer], Shown::Detected(shown) if Arc::ptr_eq(shown, image));
        if is_shown {
            return;
        }
        let written = WrittenTexels {
            layer: layer as u32,
            texels: &self.everything(),
            image: &image.values_at_size(self.size),
        };
        write_texels(queue, self, &written);
        self.shown[layer] = Shown::Detected(Arc::clone(image));
    }
}

impl CoverageLayers {
    /// Brings the layers up to date with `sources` and returns them; `None`
    /// when no mask is rendered from an image.
    pub(super) fn showing(
        &mut self,
        (device, queue): (&wgpu::Device, &wgpu::Queue),
        photo_size: [u32; 2],
        sources: &[CoverageSource<'_>],
    ) -> Option<&wgpu::TextureView> {
        if sources.is_empty() {
            self.layers = None;
            return None;
        }
        let size = coverage_image_size(photo_size);
        let layers = self
            .layers
            .take()
            .filter(|layers| layers.shown.len() == sources.len())
            .unwrap_or_else(|| {
                let texture = layers_texture(device, size, sources.len() as u32);
                Layers {
                    view: layers_view(&texture),
                    texture,
                    size,
                    shown: sources.iter().map(|_| Shown::Nothing).collect(),
                }
            });
        let layers = self.layers.insert(layers);
        for (layer, source) in sources.iter().enumerate() {
            match source {
                CoverageSource::Strokes(mask) => layers.show_strokes(queue, layer, mask),
                CoverageSource::Image(image) => layers.show_image(queue, layer, image),
            }
        }
        Some(&layers.view)
    }
}
