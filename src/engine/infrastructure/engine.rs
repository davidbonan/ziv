use crate::photo::domain::working_image::WorkingImage;

use super::display_readback::{DisplayPixels, read_display_pixels};
use super::display_stage::{DisplayRequest, DisplayStage};
use super::mipmap_generator::MipmapGenerator;
use super::source_texture::{SourceTexture, WorkingTexture};

const STRIP_ROWS: u32 = 1024;

/// Renders working images on a GPU device; never needs a window.
pub struct Engine {
    device: wgpu::Device,
    queue: wgpu::Queue,
    display_stage: DisplayStage,
    mipmap_generator: MipmapGenerator,
}

impl Engine {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue) -> Self {
        let display_stage = DisplayStage::new(&device, &queue);
        let mipmap_generator = MipmapGenerator::new(&device);
        Self {
            device,
            queue,
            display_stage,
            mipmap_generator,
        }
    }

    fn uploaded_with_mipmaps(&self, image: &WorkingImage) -> WorkingTexture {
        let uploaded = WorkingTexture::upload(&self.device, &self.queue, image);
        let mipmaps = self
            .mipmap_generator
            .generate(&self.device, uploaded.texture());
        self.queue.submit([mipmaps]);
        uploaded
    }

    pub fn upload(&self, image: &WorkingImage) -> SourceTexture {
        let size = [image.width(), image.height()];
        SourceTexture::of(self.uploaded_with_mipmaps(image), size)
    }

    /// Gives `source` its enhancement, which the edit's intensity then mixes in.
    pub fn upload_enhancement(&self, source: &SourceTexture, enhanced: &WorkingImage) {
        source.set_enhancement(self.uploaded_with_mipmaps(enhanced));
    }

    pub fn render_display(
        &self,
        source: &SourceTexture,
        request: &DisplayRequest,
    ) -> wgpu::Texture {
        self.display_stage.render(source, request)
    }

    /// What `request` renders, as pixels. Rendered strip by strip so that no
    /// texture grows with the output.
    pub fn render_pixels(
        &self,
        source: &SourceTexture,
        request: &DisplayRequest,
    ) -> Result<DisplayPixels, wgpu::BufferAsyncError> {
        let [width, height] = request.size;
        let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
        for first_row in (0..height).step_by(STRIP_ROWS as usize) {
            let rows = first_row..(first_row + STRIP_ROWS).min(height);
            let strip =
                self.read_display_pixels(&self.render_display(source, &request.strip(rows)))?;
            rgba.extend_from_slice(&strip.rgba);
        }
        Ok(DisplayPixels {
            width,
            height,
            rgba,
        })
    }

    pub fn read_display_pixels(
        &self,
        output: &wgpu::Texture,
    ) -> Result<DisplayPixels, wgpu::BufferAsyncError> {
        read_display_pixels(&self.device, &self.queue, output)
    }
}
