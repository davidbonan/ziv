use std::fs::OpenOptions;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder, ImageError};

use crate::engine::infrastructure::display_readback::DisplayPixels;
use crate::export::domain::export_settings::{ExportFormat, ExportSettings};

fn encoded(pixels: &DisplayPixels, settings: &ExportSettings) -> Result<Vec<u8>, ImageError> {
    let (opaque_pixels, _) = pixels.rgba.as_chunks::<4>();
    let rgb: Vec<u8> = opaque_pixels
        .iter()
        .flat_map(|[red, green, blue, _]| [*red, *green, *blue])
        .collect();
    let mut file_content = Vec::new();
    let color = ExtendedColorType::Rgb8;
    match settings.format {
        ExportFormat::Jpeg => JpegEncoder::new_with_quality(
            &mut file_content,
            settings.jpeg_quality,
        )
        .write_image(&rgb, pixels.width, pixels.height, color)?,
        ExportFormat::Png => PngEncoder::new(&mut file_content).write_image(
            &rgb,
            pixels.width,
            pixels.height,
            color,
        )?,
    }
    Ok(file_content)
}

// Creating the file fails when the name is taken: nothing is ever overwritten.
fn write_under_a_free_name(
    destination: &Path,
    name_for_attempt: impl Fn(u32) -> String,
    content: &[u8],
) -> std::io::Result<PathBuf> {
    for attempt in 0.. {
        let path = destination.join(name_for_attempt(attempt));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                file.write_all(content)?;
                return Ok(path);
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    unreachable!("attempts are unbounded")
}

/// Writes the pixels as a new file of the destination, named after `photo`.
pub fn write_exported_file(
    pixels: &DisplayPixels,
    photo: &Path,
    settings: &ExportSettings,
) -> Result<PathBuf, String> {
    let destination = settings
        .destination
        .as_deref()
        .ok_or("no destination folder was chosen")?;
    let content = encoded(pixels, settings).map_err(|error| error.to_string())?;
    let name_for_attempt = |attempt| settings.file_name(photo, attempt);
    write_under_a_free_name(destination, name_for_attempt, &content)
        .map_err(|error| error.to_string())
}
