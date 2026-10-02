use super::photo_kind::PhotoKind;
use super::working_image::WorkingImage;

#[derive(Debug, Clone, PartialEq)]
pub struct DecodedPhoto {
    pub image: WorkingImage,
    pub kind: PhotoKind,
}
