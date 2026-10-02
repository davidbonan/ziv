use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder, ImageFormat};

use crate::enhance::domain::enhancement::Enhancement;
use crate::enhance::domain::enhancement_storage::EnhancementStorage;
use crate::enhance::domain::model_encoding::ModelEncoding;
use crate::photo::domain::photo_name::file_beside;

const FILE_SUFFIX: &str = ".ziv.enhanced";
const BEING_WRITTEN_SUFFIX: &str = ".ziv.enhanced.tmp";
const SIGNATURE: [u8; 4] = *b"ZIVE";
const VERSION: u32 = 1;
const HEADER_LENGTH: usize = 20;

pub fn enhancement_path(photo: &Path) -> PathBuf {
    file_beside(photo, FILE_SUFFIX)
}

fn header(enhancement: &Enhancement) -> Vec<u8> {
    let [width, height] = enhancement.size();
    let ceiling = enhancement.encoding().ceiling();
    let numbers = [VERSION, width, height, ceiling.to_bits()];
    let mut header = SIGNATURE.to_vec();
    header.extend(numbers.iter().flat_map(|number| number.to_le_bytes()));
    header
}

// The difference levels as a lossless picture: a lossy one would put the noise back.
fn file_content(enhancement: &Enhancement) -> Result<Vec<u8>, String> {
    let [width, height] = enhancement.size();
    let mut content = header(enhancement);
    PngEncoder::new(&mut content)
        .write_image(
            enhancement.difference_levels(),
            width,
            height,
            ExtendedColorType::Rgb8,
        )
        .map_err(|error| error.to_string())?;
    Ok(content)
}

fn enhancement_of(content: &[u8], size: [u32; 2]) -> Result<Option<Enhancement>, String> {
    let (header, picture) = content
        .split_at_checked(HEADER_LENGTH)
        .filter(|(header, _)| header.starts_with(&SIGNATURE))
        .ok_or("it is not an enhancement file")?;
    let numbers = header[SIGNATURE.len()..].as_chunks::<4>().0;
    let [version, width, height, ceiling] =
        [0, 1, 2, 3].map(|index| u32::from_le_bytes(numbers[index]));
    if version != VERSION {
        return Err(format!(
            "it was written by another version of ziv ({version})"
        ));
    }
    if [width, height] != size {
        return Ok(None);
    }
    let levels = image::load_from_memory_with_format(picture, ImageFormat::Png)
        .map_err(|error| error.to_string())?
        .into_rgb8();
    let encoding = ModelEncoding::with_ceiling(f32::from_bits(ceiling));
    let enhancement = Enhancement::new(size, encoding, levels.into_raw());
    enhancement
        .map(Some)
        .ok_or_else(|| "its picture has another size".to_owned())
}

/// One file next to each enhanced photo.
pub struct EnhancementFiles;

impl EnhancementStorage for EnhancementFiles {
    fn stored_enhancement(
        &self,
        photo: &Path,
        size: [u32; 2],
    ) -> Result<Option<Enhancement>, String> {
        match fs::read(enhancement_path(photo)) {
            Ok(content) => enhancement_of(&content, size),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(format!("it cannot be read ({error})")),
        }
    }

    // Written beside, then renamed: a crash never leaves half a file.
    fn store_enhancement(&self, photo: &Path, enhancement: &Enhancement) -> Result<(), String> {
        let being_written = file_beside(photo, BEING_WRITTEN_SUFFIX);
        let content = file_content(enhancement)?;
        fs::write(&being_written, content)
            .and_then(|()| fs::rename(&being_written, enhancement_path(photo)))
            .map_err(|error| error.to_string())
    }
}
