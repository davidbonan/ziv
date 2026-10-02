use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;

use image::{DynamicImage, ImageDecoder, ImageError, ImageFormat, ImageReader};

use crate::color::domain::working_space::SrgbInput;
use crate::photo::domain::decode_error::DecodeError;
use crate::photo::domain::decoded_photo::DecodedPhoto;
use crate::photo::domain::orientation::Orientation;
use crate::photo::domain::photo_kind::PhotoKind;
use crate::photo::domain::shooting_data::ShootingData;
use crate::photo::domain::thumbnail::Thumbnail;
use crate::photo::domain::working_image::WorkingImage;

use super::encoded_thumbnail::thumbnail_of_encoded;
use super::exif_shooting_data::shooting_data_of_tiff;

impl From<ImageError> for DecodeError {
    fn from(error: ImageError) -> Self {
        DecodeError::new(error.to_string())
    }
}

fn decoder_of(path: &Path) -> Result<impl ImageDecoder, DecodeError> {
    let reader = ImageReader::open(path)
        .and_then(ImageReader::with_guessed_format)
        .map_err(|error| DecodeError::new(error.to_string()))?;
    Ok(reader.into_decoder()?)
}

pub fn decode_standard_image_thumbnail(path: &Path) -> Result<Thumbnail, DecodeError> {
    let mut decoder = decoder_of(path)?;
    let orientation = decoder.orientation()?;
    Ok(thumbnail_of_encoded(
        DynamicImage::from_decoder(decoder)?,
        orientation,
    ))
}

/// A TIFF file is its own EXIF structure; the other formats embed one.
fn shooting_data(path: &Path, embedded_exif: Option<Vec<u8>>) -> ShootingData {
    if let Some(exif) = embedded_exif {
        return shooting_data_of_tiff(Cursor::new(exif));
    }
    let is_tiff = ImageFormat::from_path(path).is_ok_and(|format| format == ImageFormat::Tiff);
    match File::open(path) {
        Ok(file) if is_tiff => shooting_data_of_tiff(BufReader::new(file)),
        _ => ShootingData::default(),
    }
}

/// JPEG, PNG or TIFF, recognised by content. Pixels are read as sRGB.
pub fn decode_standard_image_file(path: &Path) -> Result<DecodedPhoto, DecodeError> {
    let mut decoder = decoder_of(path)?;
    let orientation =
        Orientation::from_exif(decoder.orientation()?.to_exif().into()).unwrap_or_default();
    let shooting_data = shooting_data(path, decoder.exif_metadata()?);

    let encoded = DynamicImage::from_decoder(decoder)?.into_rgb32f();
    let srgb_input = SrgbInput::default();
    let pixels = encoded
        .pixels()
        .map(|pixel| srgb_input.to_working(pixel.0))
        .collect();
    let stored = WorkingImage::new(encoded.width(), encoded.height(), pixels);
    Ok(DecodedPhoto {
        image: stored.upright(orientation),
        kind: PhotoKind::StandardImage,
        shooting_data,
    })
}
