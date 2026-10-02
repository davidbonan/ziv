use std::io::{Read, Seek};

use rawler::exif::Exif;
use rawler::formats::tiff::reader::TiffReader;
use rawler::formats::tiff::{GenericTiffReader, Rational};

use crate::photo::domain::shooting_data::ShootingData;

/// Sensitivities too high for the 16-bit tag say so with this value there.
const ISO_TOLD_ELSEWHERE: u16 = u16::MAX;

fn measured(value: Option<Rational>) -> Option<f32> {
    let Rational { n, d } = value?;
    (d != 0).then(|| n as f32 / d as f32)
}

pub fn shooting_data_of(exif: &Exif) -> ShootingData {
    let rated = exif
        .iso_speed_ratings
        .filter(|iso| *iso != ISO_TOLD_ELSEWHERE)
        .map(u32::from);
    ShootingData {
        iso: rated.or(exif.recommended_exposure_index).or(exif.iso_speed),
        focal_length: measured(exif.focal_length),
        aperture: measured(exif.fnumber),
        shutter_speed: measured(exif.exposure_time),
    }
}

/// What a TIFF structure says: a TIFF file, or the EXIF block of another format.
/// Nothing when it cannot be read.
pub fn shooting_data_of_tiff(mut structure: impl Read + Seek) -> ShootingData {
    GenericTiffReader::new(&mut structure, 0, 0, Some(1), &[])
        .ok()
        .and_then(|tiff| Exif::new(tiff.root_ifd()).ok())
        .map(|exif| shooting_data_of(&exif))
        .unwrap_or_default()
}
