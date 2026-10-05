use std::path::Path;

use crate::photo::domain::decode_error::DecodeError;
use crate::photo::domain::decoded_photo::DecodedPhoto;
use crate::photo::domain::photo_details::PhotoDetails;
use crate::photo::domain::thumbnail::Thumbnail;

use super::raw_file::{
    decode_raw_embedded_picture, decode_raw_file, decode_raw_thumbnail, read_raw_details,
};
use super::standard_image_file::{
    decode_standard_image_file, decode_standard_image_thumbnail, read_standard_image_details,
};

const STANDARD_IMAGE_EXTENSIONS: [&str; 5] = ["JPG", "JPEG", "PNG", "TIF", "TIFF"];

enum FileKind {
    StandardImage,
    Raw,
}

fn file_kind(path: &Path) -> Option<FileKind> {
    let extension = path.extension()?.to_str()?.to_ascii_uppercase();
    if STANDARD_IMAGE_EXTENSIONS.contains(&extension.as_str()) {
        return Some(FileKind::StandardImage);
    }
    rawler::decoders::supported_extensions()
        .contains(&extension.as_str())
        .then_some(FileKind::Raw)
}

/// Decodes photo files from disk, choosing the decoder from the file extension.
pub struct FileDecoder;

impl FileDecoder {
    /// Whether the file looks like a photo ziv can decode; does not read it.
    pub fn supports(&self, path: &Path) -> bool {
        file_kind(path).is_some()
    }

    pub fn decode(&self, path: &Path) -> Result<DecodedPhoto, DecodeError> {
        match file_kind(path) {
            Some(FileKind::StandardImage) => decode_standard_image_file(path),
            Some(FileKind::Raw) => decode_raw_file(path),
            None => Err(DecodeError::new("unsupported file type")),
        }
    }

    /// The picture a RAW embeds, read without developing it; `None` for a
    /// standard image or a RAW that embeds none.
    pub fn decode_embedded_picture(
        &self,
        path: &Path,
    ) -> Result<Option<DecodedPhoto>, DecodeError> {
        match file_kind(path) {
            Some(FileKind::Raw) => decode_raw_embedded_picture(path),
            _ => Ok(None),
        }
    }

    /// What the file says about its photo, without decoding it; nothing for
    /// what cannot be read.
    pub fn read_details(&self, path: &Path) -> PhotoDetails {
        let from_content = match file_kind(path) {
            Some(FileKind::StandardImage) => read_standard_image_details(path),
            Some(FileKind::Raw) => read_raw_details(path),
            None => Ok(PhotoDetails::default()),
        };
        PhotoDetails {
            file_bytes: std::fs::metadata(path).ok().map(|file| file.len()),
            ..from_content.unwrap_or_default()
        }
    }

    pub fn decode_thumbnail(&self, path: &Path) -> Result<Thumbnail, DecodeError> {
        match file_kind(path) {
            Some(FileKind::StandardImage) => decode_standard_image_thumbnail(path),
            Some(FileKind::Raw) => decode_raw_thumbnail(path),
            None => Err(DecodeError::new("unsupported file type")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_and_standard_extensions_are_supported_whatever_their_case() {
        for name in [
            "a.ARW", "a.arw", "a.jpg", "a.JPEG", "a.png", "a.tif", "a.Tiff", "a.dng",
        ] {
            assert!(FileDecoder.supports(Path::new(name)), "{name}");
        }
    }

    #[test]
    fn other_files_are_not_supported() {
        for name in ["a.txt", "a.xmp", "a.mp4", "ARW", ".DS_Store"] {
            assert!(!FileDecoder.supports(Path::new(name)), "{name}");
        }
    }
}
