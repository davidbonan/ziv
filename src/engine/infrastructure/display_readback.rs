use std::sync::mpsc::channel;

const BYTES_PER_PIXEL: u32 = 4;

/// Display-encoded RGBA8 pixels read back from the GPU, row by row from the top-left.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayPixels {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub fn read_display_pixels(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) -> Result<DisplayPixels, wgpu::BufferAsyncError> {
    let (width, height) = (texture.width(), texture.height());
    let row_bytes = width * BYTES_PER_PIXEL;
    let padded_row_bytes = row_bytes.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ziv display readback"),
        size: u64::from(padded_row_bytes) * u64::from(height),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("ziv display readback"),
    });
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_row_bytes),
                rows_per_image: None,
            },
        },
        texture.size(),
    );
    queue.submit([encoder.finish()]);

    let (sender, receiver) = channel();
    buffer.map_async(wgpu::MapMode::Read, .., move |mapped| {
        let _ = sender.send(mapped);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("device is alive while reading back");
    receiver.recv().expect("map callback runs during poll")?;

    let mapped = buffer.get_mapped_range(..).expect("buffer was just mapped");
    let rgba = mapped
        .chunks_exact(padded_row_bytes as usize)
        .flat_map(|row| &row[..row_bytes as usize])
        .copied()
        .collect();
    Ok(DisplayPixels {
        width,
        height,
        rgba,
    })
}
