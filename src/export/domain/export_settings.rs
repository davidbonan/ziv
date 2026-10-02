use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const JPEG_QUALITY_RANGE: RangeInclusive<u8> = 1..=100;
pub const LONG_EDGE_RANGE: RangeInclusive<u32> = 256..=12000;
const DEFAULT_JPEG_QUALITY: u8 = 90;
const DEFAULT_LONG_EDGE: u32 = 2048;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Jpeg,
    Png,
}

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportSize {
    FullResolution,
    LongEdge,
}

/// How photos are exported, as the dialog shows it and as it is remembered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ExportSettings {
    pub format: ExportFormat,
    pub jpeg_quality: u8,
    pub size: ExportSize,
    /// In pixels; used when `size` is `LongEdge`, kept otherwise.
    pub long_edge: u32,
    pub destination: Option<PathBuf>,
}

impl Default for ExportSettings {
    fn default() -> Self {
        Self {
            format: ExportFormat::Jpeg,
            jpeg_quality: DEFAULT_JPEG_QUALITY,
            size: ExportSize::FullResolution,
            long_edge: DEFAULT_LONG_EDGE,
            destination: None,
        }
    }
}

impl ExportSettings {
    /// The size of the file exported from a photo of `photo_size`: never
    /// larger than the photo, aspect ratio kept.
    pub fn output_size(&self, photo_size: [u32; 2]) -> [u32; 2] {
        let photo_long_edge = photo_size[0].max(photo_size[1]);
        if self.size == ExportSize::FullResolution || self.long_edge >= photo_long_edge {
            return photo_size;
        }
        let scaled = |side: u32| {
            let exact = u64::from(side) * u64::from(self.long_edge) / u64::from(photo_long_edge);
            (exact as u32).max(1)
        };
        photo_size.map(scaled)
    }

    /// The name of the file exported from `photo`. `attempt` 0 is the plain
    /// name; the following ones carry a number, for when the name is taken.
    pub fn file_name(&self, photo: &Path, attempt: u32) -> String {
        let stem = photo
            .file_stem()
            .map(|stem| stem.to_string_lossy())
            .unwrap_or_default();
        let extension = self.format.extension();
        if attempt == 0 {
            return format!("{stem}.{extension}");
        }
        format!("{stem}-{attempt}.{extension}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_long_edge(long_edge: u32) -> ExportSettings {
        ExportSettings {
            size: ExportSize::LongEdge,
            long_edge,
            ..ExportSettings::default()
        }
    }

    #[test]
    fn full_resolution_keeps_the_photo_size() {
        assert_eq!(
            ExportSettings::default().output_size([7008, 4672]),
            [7008, 4672]
        );
    }

    #[test]
    fn long_edge_scales_the_photo_and_keeps_its_aspect_ratio() {
        assert_eq!(with_long_edge(2048).output_size([7008, 4672]), [2048, 1365]);
        assert_eq!(with_long_edge(2048).output_size([4672, 7008]), [1365, 2048]);
    }

    #[test]
    fn a_photo_smaller_than_the_long_edge_is_not_enlarged() {
        assert_eq!(with_long_edge(2048).output_size([1200, 800]), [1200, 800]);
    }

    #[test]
    fn exported_file_is_named_after_the_photo_with_the_format_extension() {
        let png = ExportSettings {
            format: ExportFormat::Png,
            ..ExportSettings::default()
        };
        let photo = Path::new("shoot/DSC07070.ARW");

        assert_eq!(
            ExportSettings::default().file_name(photo, 0),
            "DSC07070.jpg"
        );
        assert_eq!(png.file_name(photo, 0), "DSC07070.png");
        assert_eq!(png.file_name(photo, 2), "DSC07070-2.png");
    }

    #[test]
    fn settings_are_remembered_through_their_serialized_form() {
        let settings = ExportSettings {
            format: ExportFormat::Png,
            destination: Some(PathBuf::from("/exports")),
            ..with_long_edge(1600)
        };

        let remembered: ExportSettings =
            serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();

        assert_eq!(remembered, settings);
    }
}
