use super::photo_kind::PhotoKind;
use super::shooting_data::ShootingData;
use super::working_image::WorkingImage;

#[derive(Debug, Clone, PartialEq)]
pub struct DecodedPhoto {
    pub image: WorkingImage,
    pub kind: PhotoKind,
    pub shooting_data: ShootingData,
}
