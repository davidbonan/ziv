use crate::color::domain::illuminant::Illuminant;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PhotoKind {
    Raw {
        /// The light the camera balanced the photo for.
        as_shot: Illuminant,
    },
    StandardImage,
}
