use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::import_day::ImportDay;
use super::session::Session;

/// What the user opened at once: the photos found, and the folder they come
/// from when exactly one folder was opened.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Import {
    pub folder: Option<PathBuf>,
    pub photos: Vec<PathBuf>,
}

/// How many photos stand for a series in its cover.
pub const COVER_PHOTO_COUNT: usize = 4;

/// What one import brought in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Series {
    pub name: String,
    /// The folder of a folder series; a photo set has none.
    pub folder: Option<PathBuf>,
    pub imported_on: ImportDay,
    pub session: Session,
}

fn folder_name(folder: &Path) -> String {
    folder
        .file_name()
        .unwrap_or(folder.as_os_str())
        .to_string_lossy()
        .into_owned()
}

fn photo_set_name(count: usize, day: ImportDay) -> String {
    match count {
        1 => format!("1 photo, {day}"),
        _ => format!("{count} photos, {day}"),
    }
}

impl Series {
    /// `None` when the import found no photo.
    pub fn imported(import: Import, day: ImportDay) -> Option<Self> {
        let session = Session::of(import.photos)?;
        let name = match &import.folder {
            Some(folder) => folder_name(folder),
            None => photo_set_name(session.photos().len(), day),
        };
        Some(Self {
            name,
            folder: import.folder,
            imported_on: day,
            session,
        })
    }

    /// The first photos of the series.
    pub fn cover_photos(&self) -> &[PathBuf] {
        let photos = self.session.photos();
        &photos[..photos.len().min(COVER_PHOTO_COUNT)]
    }

    /// Whether none of its photos `is_there`.
    pub fn is_missing(&self, is_there: impl Fn(&Path) -> bool) -> bool {
        !self.session.photos().iter().any(|photo| is_there(photo))
    }

    /// The folder the Finder shows for this series: its own, or the first photo's.
    pub fn folder_to_show(&self) -> Option<&Path> {
        let of_first_photo = || self.session.photos().first()?.parent();
        self.folder.as_deref().or_else(of_first_photo)
    }

    /// Brings back the photos found by name in `folder`; a folder series then
    /// remembers that folder. Returns how many were found.
    pub fn relocate(&mut self, folder: &Path, is_there: impl Fn(&Path) -> bool) -> usize {
        let found = self.session.relocate(folder, is_there);
        if found > 0 && self.folder.is_some() {
            self.folder = Some(folder.to_owned());
        }
        found
    }

    pub fn edited_count(&self, is_edited: impl Fn(&Path) -> bool) -> usize {
        let photos = self.session.photos().iter();
        photos.filter(|photo| is_edited(photo)).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: ImportDay = ImportDay {
        year: 2026,
        month: 9,
        day: 14,
    };

    fn photos(names: &[&str]) -> Vec<PathBuf> {
        names.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn folder_series_is_named_after_its_folder() {
        let import = Import {
            folder: Some(PathBuf::from("/shoots/Lofoten")),
            photos: photos(&["/shoots/Lofoten/2.jpg", "/shoots/Lofoten/1.jpg"]),
        };

        let series = Series::imported(import, DAY).unwrap();

        assert_eq!(series.name, "Lofoten");
        assert_eq!(series.folder, Some(PathBuf::from("/shoots/Lofoten")));
        assert_eq!(
            series.session.selected_photo(),
            Some(Path::new("/shoots/Lofoten/1.jpg"))
        );
    }

    #[test]
    fn photo_set_is_named_after_its_size_and_its_day() {
        let several = Import {
            folder: None,
            photos: photos(&["/a/1.jpg", "/b/2.jpg"]),
        };
        let single = Import {
            folder: None,
            photos: photos(&["/a/1.jpg"]),
        };

        assert_eq!(
            Series::imported(several, DAY).unwrap().name,
            "2 photos, 14 Sep"
        );
        assert_eq!(
            Series::imported(single, DAY).unwrap().name,
            "1 photo, 14 Sep"
        );
    }

    #[test]
    fn series_is_missing_when_none_of_its_photos_is_there() {
        let import = Import {
            folder: None,
            photos: photos(&["/a/1.jpg", "/a/2.jpg"]),
        };
        let series = Series::imported(import, DAY).unwrap();

        assert!(series.is_missing(|_| false));
        assert!(!series.is_missing(|photo| photo == Path::new("/a/2.jpg")));
    }

    #[test]
    fn finder_shows_the_folder_of_the_first_photo_of_a_photo_set() {
        let import = Import {
            folder: None,
            photos: photos(&["/b/2.jpg", "/a/1.jpg"]),
        };
        let series = Series::imported(import, DAY).unwrap();

        assert_eq!(series.folder_to_show(), Some(Path::new("/a")));
    }

    #[test]
    fn import_without_photo_makes_no_series() {
        assert_eq!(Series::imported(Import::default(), DAY), None);
    }
}
