use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageFormat, RgbaImage};
use sha2::{Digest, Sha256};

use crate::photo::domain::decode_error::DecodeError;
use crate::photo::domain::thumbnail::Thumbnail;

const JPEG_QUALITY: u8 = 85;

/// Thumbnails kept on disk between two runs, one JPEG per photo as it was when decoded.
#[derive(Debug, Clone)]
pub struct ThumbnailCache {
    /// `None` when the user has no cache folder: every thumbnail is decoded.
    folder: Option<PathBuf>,
}

/// Names the photo as it is now: a photo changed on disk gets another name.
fn file_name_of(photo: &Path) -> Option<String> {
    let metadata = fs::metadata(photo).ok()?;
    let modified = metadata.modified().ok()?.duration_since(UNIX_EPOCH).ok()?;
    let mut hash = Sha256::new();
    hash.update(photo.as_os_str().as_encoded_bytes());
    hash.update(metadata.len().to_le_bytes());
    hash.update(modified.as_nanos().to_le_bytes());
    Some(format!("{:x}.jpg", hash.finalize()))
}

fn jpeg_of(thumbnail: &Thumbnail) -> Option<Vec<u8>> {
    let picture = RgbaImage::from_raw(thumbnail.width, thumbnail.height, thumbnail.rgba.clone())?;
    let mut jpeg = Vec::new();
    DynamicImage::ImageRgba8(picture)
        .into_rgb8()
        .write_with_encoder(JpegEncoder::new_with_quality(&mut jpeg, JPEG_QUALITY))
        .ok()?;
    Some(jpeg)
}

impl ThumbnailCache {
    pub fn of_this_mac() -> Self {
        Self {
            folder: dirs::cache_dir().map(|cache| cache.join("ziv").join("thumbnails")),
        }
    }

    pub fn in_folder(folder: PathBuf) -> Self {
        Self {
            folder: Some(folder),
        }
    }

    fn file_of(&self, photo: &Path) -> Option<PathBuf> {
        Some(self.folder.as_ref()?.join(file_name_of(photo)?))
    }

    fn kept(&self, photo: &Path) -> Option<Thumbnail> {
        let jpeg = fs::read(self.file_of(photo)?).ok()?;
        let picture = image::load_from_memory_with_format(&jpeg, ImageFormat::Jpeg)
            .ok()?
            .into_rgba8();
        Some(Thumbnail {
            width: picture.width(),
            height: picture.height(),
            rgba: picture.into_raw(),
        })
    }

    // Written beside, then renamed: two loaders never read half a thumbnail.
    fn keep(&self, photo: &Path, thumbnail: &Thumbnail) -> Option<()> {
        let file = self.file_of(photo)?;
        let being_written = file.with_extension("tmp");
        fs::create_dir_all(file.parent()?).ok()?;
        fs::write(&being_written, jpeg_of(thumbnail)?).ok()?;
        fs::rename(being_written, file).ok()
    }

    /// The thumbnail kept for `photo`, else the one `decode` makes, kept for the next time.
    pub fn thumbnail_of(
        &self,
        photo: &Path,
        decode: impl FnOnce(&Path) -> Result<Thumbnail, DecodeError>,
    ) -> Result<Thumbnail, DecodeError> {
        if let Some(kept) = self.kept(photo) {
            return Ok(kept);
        }
        let thumbnail = decode(photo)?;
        self.keep(photo, &thumbnail);
        Ok(thumbnail)
    }
}
