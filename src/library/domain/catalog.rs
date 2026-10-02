use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::series::Series;
use super::session::Session;

/// The series ziv remembers, the most recently imported first, and the open one.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Catalog {
    series: Vec<Series>,
    open: Option<usize>,
}

impl Catalog {
    pub fn series(&self) -> &[Series] {
        &self.series
    }

    pub fn open_index(&self) -> Option<usize> {
        self.open
    }

    pub fn open_series(&self) -> Option<&Series> {
        self.open.map(|index| &self.series[index])
    }

    fn open_session_mut(&mut self) -> Option<&mut Session> {
        self.open.map(|index| &mut self.series[index].session)
    }

    /// False for a catalog read from a file someone damaged.
    pub fn is_consistent(&self) -> bool {
        self.open.is_none_or(|index| index < self.series.len())
            && self
                .series
                .iter()
                .all(|series| series.session.is_selection_inside())
    }

    fn folder_series_of(&self, folder: Option<&Path>) -> Option<usize> {
        let folder = folder?;
        self.series
            .iter()
            .position(|series| series.folder.as_deref() == Some(folder))
    }

    /// Opens `imported`, put first. A folder the catalog already has as a
    /// series is that series instead, opened with the photos it did not have yet.
    pub fn import(&mut self, imported: Series) {
        match self.folder_series_of(imported.folder.as_deref()) {
            Some(known) => {
                let photos = imported.session.photos().to_vec();
                self.series[known].session.add(photos);
                self.open = Some(known);
            }
            None => {
                self.series.insert(0, imported);
                self.open = Some(0);
            }
        }
    }

    /// Opens the series at `index`, on the photo it was left on; an index
    /// outside the catalog changes nothing.
    pub fn open(&mut self, index: usize) {
        if index < self.series.len() {
            self.open = Some(index);
        }
    }

    /// Returns how many photos of the series at `index` were found in `folder`.
    pub fn relocate(
        &mut self,
        index: usize,
        folder: &Path,
        is_there: impl Fn(&Path) -> bool,
    ) -> usize {
        let Some(series) = self.series.get_mut(index) else {
            return 0;
        };
        series.relocate(folder, is_there)
    }

    /// A name that is empty once trimmed changes nothing.
    pub fn rename(&mut self, index: usize, name: &str) {
        let name = name.trim();
        if let Some(series) = self.series.get_mut(index).filter(|_| !name.is_empty()) {
            series.name = name.to_owned();
        }
    }

    /// Takes the series at `index` out. When it was the open one, the series
    /// below it is opened, else the one above it.
    pub fn remove(&mut self, index: usize) {
        if index >= self.series.len() {
            return;
        }
        self.series.remove(index);
        let last = self.series.len().checked_sub(1);
        self.open = match self.open {
            Some(open) if open > index => Some(open - 1),
            Some(open) if open == index => last.map(|last| index.min(last)),
            open => open,
        };
    }

    /// The photos of the open series; none when no series is open.
    pub fn photos(&self) -> &[PathBuf] {
        self.open_series()
            .map_or(&[], |series| series.session.photos())
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.open_series()?.session.selected_index()
    }

    pub fn selected_photo(&self) -> Option<&Path> {
        self.open_series()?.session.selected_photo()
    }

    /// Where `photo` is in the open series.
    pub fn index_of(&self, photo: &Path) -> Option<usize> {
        self.open_series()?.session.index_of(photo)
    }

    pub fn select(&mut self, index: usize) {
        if let Some(session) = self.open_session_mut() {
            session.select(index);
        }
    }

    pub fn select_previous(&mut self) {
        if let Some(session) = self.open_session_mut() {
            session.select_previous();
        }
    }

    pub fn select_next(&mut self) {
        if let Some(session) = self.open_session_mut() {
            session.select_next();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::domain::import_day::ImportDay;
    use crate::library::domain::series::Import;

    fn series_of(folder: &str, names: &[&str]) -> Series {
        let import = Import {
            folder: Some(PathBuf::from(folder)),
            photos: names
                .iter()
                .map(|name| Path::new(folder).join(name))
                .collect(),
        };
        let day = ImportDay {
            year: 2026,
            month: 9,
            day: 14,
        };
        Series::imported(import, day).unwrap()
    }

    #[test]
    fn empty_catalog_has_no_photo() {
        let catalog = Catalog::default();

        assert!(catalog.photos().is_empty());
        assert_eq!(catalog.selected_photo(), None);
    }

    #[test]
    fn added_series_is_first_and_open_and_the_earlier_ones_stay() {
        let mut catalog = Catalog::default();
        catalog.import(series_of("/old", &["a.jpg"]));

        catalog.import(series_of("/new", &["b.jpg", "c.jpg"]));

        let names: Vec<&str> = catalog
            .series()
            .iter()
            .map(|series| series.name.as_str())
            .collect();
        assert_eq!(names, ["new", "old"]);
        assert_eq!(catalog.open_index(), Some(0));
        assert_eq!(catalog.selected_photo(), Some(Path::new("/new/b.jpg")));
    }

    #[test]
    fn folder_imported_again_brings_its_new_photos_into_its_series() {
        let mut catalog = Catalog::default();
        let mut lofoten = series_of("/lofoten", &["1.jpg", "2.jpg"]);
        lofoten.name = "Renamed".to_owned();
        catalog.import(lofoten);
        catalog.select_next();
        catalog.import(series_of("/new", &["c.jpg"]));

        catalog.import(series_of("/lofoten", &["1.jpg", "2.jpg", "3.jpg"]));

        assert_eq!(catalog.series().len(), 2);
        assert_eq!(catalog.open_index(), Some(1));
        assert_eq!(catalog.series()[1].name, "Renamed");
        assert_eq!(catalog.photos().len(), 3);
        assert_eq!(catalog.selected_photo(), Some(Path::new("/lofoten/2.jpg")));
    }

    #[test]
    fn photo_set_imported_again_is_another_series() {
        let mut catalog = Catalog::default();
        let photo_set = || Series {
            folder: None,
            ..series_of("/loose", &["a.jpg"])
        };
        catalog.import(photo_set());

        catalog.import(photo_set());

        assert_eq!(catalog.series().len(), 2);
    }

    fn three_series() -> Catalog {
        let mut catalog = Catalog::default();
        for folder in ["/c", "/b", "/a"] {
            catalog.import(series_of(folder, &["1.jpg"]));
        }
        catalog
    }

    fn names(catalog: &Catalog) -> Vec<&str> {
        let series = catalog.series().iter();
        series.map(|series| series.name.as_str()).collect()
    }

    #[test]
    fn removing_the_open_series_opens_the_one_below_then_the_one_above() {
        let mut catalog = three_series();
        catalog.open(1);

        catalog.remove(1);
        assert_eq!(names(&catalog), ["a", "c"]);
        assert_eq!(catalog.open_series().unwrap().name, "c");

        catalog.remove(1);
        assert_eq!(catalog.open_series().unwrap().name, "a");

        catalog.remove(0);
        assert_eq!(catalog.open_series(), None);
    }

    #[test]
    fn removing_another_series_keeps_the_open_one_open() {
        let mut catalog = three_series();
        catalog.open(2);

        catalog.remove(0);

        assert_eq!(catalog.open_series().unwrap().name, "c");
    }

    #[test]
    fn renaming_trims_the_name_and_an_empty_name_changes_nothing() {
        let mut catalog = three_series();

        catalog.rename(0, "  Lofoten ");
        catalog.rename(1, "   ");

        assert_eq!(names(&catalog), ["Lofoten", "b", "c"]);
    }

    #[test]
    fn relocated_folder_series_remembers_its_new_folder() {
        let mut catalog = three_series();

        let found = catalog.relocate(0, Path::new("/moved"), |_| true);

        assert_eq!(found, 1);
        assert_eq!(catalog.series()[0].folder, Some(PathBuf::from("/moved")));
        assert_eq!(catalog.photos(), [PathBuf::from("/moved/1.jpg")]);
    }

    #[test]
    fn folder_without_the_photos_relocates_nothing() {
        let mut catalog = three_series();
        let before = catalog.clone();

        let found = catalog.relocate(0, Path::new("/elsewhere"), |_| false);

        assert_eq!(found, 0);
        assert_eq!(catalog, before);
    }

    #[test]
    fn reopened_series_is_on_the_photo_it_was_left_on() {
        let mut catalog = Catalog::default();
        catalog.import(series_of("/old", &["a.jpg", "b.jpg"]));
        catalog.select_next();
        catalog.import(series_of("/new", &["c.jpg"]));

        catalog.open(1);

        assert_eq!(catalog.selected_photo(), Some(Path::new("/old/b.jpg")));
    }

    #[test]
    fn opening_a_series_outside_the_catalog_changes_nothing() {
        let mut catalog = Catalog::default();
        catalog.import(series_of("/old", &["a.jpg"]));

        catalog.open(4);

        assert_eq!(catalog.open_index(), Some(0));
    }

    #[test]
    fn selection_moves_in_the_open_series_only() {
        let mut catalog = Catalog::default();
        catalog.import(series_of("/old", &["a.jpg", "b.jpg"]));
        catalog.import(series_of("/new", &["c.jpg", "d.jpg"]));

        catalog.select_next();

        assert_eq!(catalog.selected_photo(), Some(Path::new("/new/d.jpg")));
        assert_eq!(catalog.series()[1].session.selected_index(), Some(0));
    }
}
