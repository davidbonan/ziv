use std::io::{Read, Seek};

use rawler::exif::Exif;
use rawler::formats::tiff::reader::TiffReader;
use rawler::formats::tiff::{GenericTiffReader, Rational};

use crate::photo::domain::photo_details::ShotAt;
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

pub fn shot_at_of(exif: &Exif) -> Option<ShotAt> {
    ShotAt::from_exif(exif.date_time_original.as_deref()?)
}

/// The EXIF of a TIFF structure: a TIFF file, or the EXIF block of another
/// format. `None` when it cannot be read.
pub fn exif_of_tiff(mut structure: impl Read + Seek) -> Option<Exif> {
    let tiff = GenericTiffReader::new(&mut structure, 0, 0, Some(1), &[]).ok()?;
    Exif::new(tiff.root_ifd()).ok()
}
