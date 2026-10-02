use std::path::Path;

use image::metadata::Orientation as EncodedOrientation;
use rawler::RawImage;
use rawler::RawlerError;
use rawler::decoders::RawDecodeParams;
use rawler::imgop::develop::{Intermediate, ProcessingStep, RawDevelop};
use rawler::imgop::xyz::Illuminant;
use rawler::rawsource::RawSource;
use rayon::prelude::*;

use crate::photo::domain::camera_calibration::CameraCalibration;
use crate::photo::domain::decode_error::DecodeError;
use crate::photo::domain::decoded_photo::DecodedPhoto;
use crate::photo::domain::orientation::Orientation;
use crate::photo::domain::photo_kind::PhotoKind;
use crate::photo::domain::thumbnail::Thumbnail;
use crate::photo::domain::working_image::WorkingImage;

use super::encoded_thumbnail::thumbnail_of_encoded;

// rawler's own calibration step targets sRGB and clips; ziv converts to the working space itself.
const STEPS_UP_TO_CAMERA_RGB: [ProcessingStep; 5] = [
    ProcessingStep::Rescale,
    ProcessingStep::Demosaic,
    ProcessingStep::FujiRotate,
    ProcessingStep::CropActiveArea,
    ProcessingStep::CropDefault,
];

impl From<RawlerError> for DecodeError {
    fn from(error: RawlerError) -> Self {
        DecodeError::new(error.to_string())
    }
}

fn camera_calibration(raw: &RawImage) -> Result<CameraCalibration, DecodeError> {
    let (_, flat_matrix) = raw
        .color_matrix_find_first([Illuminant::D65])
        .ok_or_else(|| DecodeError::new("no D65 color calibration for this camera"))?;
    let &[xr, xg, xb, yr, yg, yb, zr, zg, zb] = flat_matrix.as_slice() else {
        return Err(DecodeError::new(
            "sensors with four colors are not supported",
        ));
    };
    let [red, green, blue, _] = raw.wb_coeffs;
    let white_balance = if red.is_nan() {
        [1.0; 3]
    } else {
        [red, green, blue]
    };
    Ok(CameraCalibration {
        xyz_to_camera: [[xr, xg, xb], [yr, yg, yb], [zr, zg, zb]].map(|row| row.map(f64::from)),
        white_balance,
    })
}

pub fn decode_raw_file(path: &Path) -> Result<DecodedPhoto, DecodeError> {
    let source = RawSource::new(path).map_err(|error| DecodeError::new(error.to_string()))?;
    let decoder = rawler::get_decoder(&source)?;
    let params = RawDecodeParams::default();
    let raw = decoder.raw_image(&source, &params, false)?;
    let metadata = decoder.raw_metadata(&source, &params)?;
    let orientation = metadata
        .exif
        .orientation
        .and_then(Orientation::from_exif)
        .unwrap_or_default();

    let calibration = camera_calibration(&raw)?;
    let unusable_calibration = || DecodeError::new("unusable color calibration");
    let camera_to_working = calibration.to_working().ok_or_else(unusable_calibration)?;
    let as_shot = calibration
        .as_shot_illuminant()
        .ok_or_else(unusable_calibration)?;
    let camera_rgb =
        match RawDevelop::new_with(&STEPS_UP_TO_CAMERA_RGB).develop_intermediate(&raw)? {
            Intermediate::ThreeColor(camera_rgb) => camera_rgb,
            Intermediate::Monochrome(_) | Intermediate::FourColor(_) => {
                return Err(DecodeError::new("only three-color sensors are supported"));
            }
        };

    let pixels = camera_rgb
        .data
        .par_iter()
        .map(|camera| camera_to_working.convert(*camera))
        .collect();
    let stored = WorkingImage::new(camera_rgb.width as u32, camera_rgb.height as u32, pixels);
    Ok(DecodedPhoto {
        image: stored.upright(orientation),
        kind: PhotoKind::Raw { as_shot },
    })
}

/// The preview the camera embedded in the file when there is one, else a
/// reduction of the decoded RAW.
pub fn decode_raw_thumbnail(path: &Path) -> Result<Thumbnail, DecodeError> {
    let source = RawSource::new(path).map_err(|error| DecodeError::new(error.to_string()))?;
    let decoder = rawler::get_decoder(&source)?;
    let params = RawDecodeParams::default();
    let embedded = match decoder.thumbnail_image(&source, &params)? {
        Some(thumbnail) => Some(thumbnail),
        None => decoder.preview_image(&source, &params)?,
    };
    let Some(embedded) = embedded else {
        return Ok(Thumbnail::of(&decode_raw_file(path)?.image));
    };
    let orientation = decoder
        .raw_metadata(&source, &params)?
        .exif
        .orientation
        .and_then(|value| EncodedOrientation::from_exif(u8::try_from(value).ok()?))
        .unwrap_or(EncodedOrientation::NoTransforms);
    Ok(thumbnail_of_encoded(embedded, orientation))
}
