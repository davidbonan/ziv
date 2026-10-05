use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::mark::{Mark, Rating};
use super::series::Series;
use super::series_filter::SeriesFilter;
use super::session::Session;

/// The series ziv remembers, the most recently imported first, and the open one.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Catalog {
    series: Vec<Series>,
    open: Option<usize>,
    /// The marks of the photos that carry one, shared by the series holding the photo.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    marks: BTreeMap<PathBuf, Mark>,
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
                self.series[known].add(photos);
                self.open = Some(known);
            }
            None => {
                self.series.insert(0, imported);
                self.open = Some(0);
            }
        }
        self.keep_selected_photo_shown();
    }

    /// Opens the series at `index`, on the photo it was left on; an index
    /// outside the catalog changes nothing.
    pub fn open(&mut self, index: usize) {
        if index < self.series.len() {
            self.open = Some(index);
            self.keep_selected_photo_shown();
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
        let before = series.session.photos().to_vec();
        let found = series.relocate(folder, is_there);
        let photos_now = self.series[index].session.photos();
        for photo in before {
            let there = folder.join(photo.file_name().unwrap_or_default());
            let mark = self.marks.get(&photo).copied();
            if let Some(mark) = mark.filter(|_| photos_now.contains(&there)) {
                self.marks.insert(there, mark);
            }
        }
        found
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

    /// Takes the photos at `photos` out of the open series, which remembers
    /// them when it is a folder series.
    pub fn remove_photos(&mut self, photos: &[usize]) {
        self.take_out(photos, Series::remove);
    }

    /// Takes out of the open series the photos at `photos`, whose files went
    /// to the Trash: one put back in its folder is new to the series.
    pub fn drop_trashed_photos(&mut self, photos: &[usize]) {
        self.take_out(photos, Series::forget);
    }

    /// The selected photo, when it leaves, gives way to the next shown photo,
    /// else to the previous one. A series left without photo leaves the catalog.
    fn take_out(&mut self, photos: &[usize], take: impl FnOnce(&mut Series, &[PathBuf])) {
        let Some(open) = self.open else {
            return;
        };
        let stays = |index: &usize| !photos.contains(index);
        let shown = self.shown_photos();
        let to_select = self.selected_index().and_then(|selected| {
            let after = shown
                .iter()
                .find(|index| **index >= selected && stays(index));
            after.or_else(|| {
                shown
                    .iter()
                    .rev()
                    .find(|index| **index < selected && stays(index))
            })
        });
        let to_select = to_select.map(|index| self.photos()[*index].clone());
        let removed: Vec<PathBuf> = photos
            .iter()
            .filter_map(|index| self.photos().get(*index).cloned())
            .collect();

        let series = &mut self.series[open];
        take(series, &removed);
        if series.session.photos().is_empty() {
            return self.remove(open);
        }
        match to_select.and_then(|photo| series.session.index_of(&photo)) {
            Some(index) => series.session.select(index),
            None => series.session.select_none(),
        }
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

    pub fn mark_of(&self, photo: &Path) -> Mark {
        self.marks.get(photo).copied().unwrap_or_default()
    }

    fn change_marks(&mut self, photos: &[usize], change: impl Fn(Mark) -> Mark) {
        let marked: Vec<PathBuf> = photos
            .iter()
            .filter_map(|index| self.photos().get(*index).cloned())
            .collect();
        for photo in marked {
            let mark = change(self.mark_of(&photo));
            match mark.is_blank() {
                true => self.marks.remove(&photo),
                false => self.marks.insert(photo, mark),
            };
        }
        self.keep_selected_photo_shown();
    }

    /// The filter of the open series; every photo when no series is open.
    pub fn filter(&self) -> SeriesFilter {
        self.open_series()
            .map_or(SeriesFilter::All, |series| series.filter)
    }

    pub fn set_filter(&mut self, filter: SeriesFilter) {
        if let Some(open) = self.open {
            self.series[open].filter = filter;
            self.keep_selected_photo_shown();
        }
    }

    /// The photos of the open series its filter shows, by their place in the series.
    pub fn shown_photos(&self) -> Vec<usize> {
        let filter = self.filter();
        let photos = self.photos().iter().enumerate();
        photos
            .filter(|(_, photo)| filter.shows(&self.mark_of(photo)))
            .map(|(index, _)| index)
            .collect()
    }

    /// The rejected photos of the open series, shown or not, by their place in the series.
    pub fn rejected_photos(&self) -> Vec<usize> {
        let photos = self.photos().iter().enumerate();
        photos
            .filter(|(_, photo)| self.mark_of(photo).is_rejected)
            .map(|(index, _)| index)
            .collect()
    }

    /// A selected photo that is no longer shown gives way to the next shown
    /// one, else to the previous one; with nothing shown, nothing is selected.
    fn keep_selected_photo_shown(&mut self) {
        let shown = self.shown_photos();
        let to_select = match self.selected_index() {
            Some(selected) if shown.contains(&selected) => return,
            Some(selected) => {
                let next = shown.iter().find(|index| **index > selected);
                next.or_else(|| shown.iter().rev().find(|index| **index < selected))
            }
            None => shown.first(),
        };
        let to_select = to_select.copied();
        let Some(session) = self.open_session_mut() else {
            return;
        };
        match to_select {
            Some(index) => session.select(index),
            None => session.select_none(),
        }
    }

    /// Where the selected photo is among the shown ones.
    fn selected_among(&self, shown: &[usize]) -> Option<usize> {
        let selected = self.selected_index()?;
        shown.iter().position(|index| *index == selected)
    }

    /// The shown photos next to the selected one: the one after it, then the one before it.
    pub fn photos_beside_selected(&self) -> Vec<&Path> {
        let shown = self.shown_photos();
        let Some(at) = self.selected_among(&shown) else {
            return Vec::new();
        };
        let beside = [Some(at + 1), at.checked_sub(1)];
        let photos = self.photos();
        beside
            .into_iter()
            .flatten()
            .filter_map(|at| shown.get(at))
            .map(|index| photos[*index].as_path())
            .collect()
    }

    /// Gives `rating` to the photos at `photos` in the open series.
    pub fn rate(&mut self, photos: &[usize], rating: Rating) {
        self.change_marks(photos, |mark| Mark { rating, ..mark });
    }

    /// Rejects the photos at `photos` in the open series; when all of them
    /// already are, clears their rejected mark instead.
    pub fn toggle_rejected(&mut self, photos: &[usize]) {
        let is_rejected_at = |index: &usize| {
            let photo = self.photos().get(*index);
            photo.is_some_and(|photo| self.mark_of(photo).is_rejected)
        };
        let is_rejected = !photos.iter().all(is_rejected_at);
        self.change_marks(photos, |mark| Mark {
            is_rejected,
            ..mark
        });
    }

    pub fn select(&mut self, index: usize) {
        if let Some(session) = self.open_session_mut() {
            session.select(index);
        }
    }

    /// Selects the shown photo before the selected one; stops at the first.
    pub fn select_previous(&mut self) {
        let shown = self.shown_photos();
        let previous = self.selected_among(&shown).and_then(|at| at.checked_sub(1));
        if let Some(previous) = previous {
            self.select(shown[previous]);
        }
    }

    /// Selects the shown photo after the selected one; stops at the last.
    pub fn select_next(&mut self) {
        let shown = self.shown_photos();
        let next = self.selected_among(&shown).map(|at| at + 1);
        if let Some(index) = next.and_then(|next| shown.get(next)) {
            self.select(*index);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::domain::import_day::ImportDay;
    use crate::library::domain::series::Import;
    use crate::photo::domain::photo_name::photo_name;

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
    fn rating_marks_the_photos_asked_and_leaves_the_others() {
        let mut catalog = Catalog::default();
        catalog.import(series_of("/shoot", &["a.jpg", "b.jpg", "c.jpg"]));

        catalog.rate(&[0, 2], Rating::of(3));

        let stars = |name| {
            catalog
                .mark_of(&Path::new("/shoot").join(name))
                .rating
                .stars()
        };
        assert_eq!([stars("a.jpg"), stars("b.jpg"), stars("c.jpg")], [3, 0, 3]);
    }

    #[test]
    fn rejecting_rejects_all_unless_all_already_are_and_keeps_the_rating() {
        let mut catalog = Catalog::default();
        catalog.import(series_of("/shoot", &["a.jpg", "b.jpg"]));
        catalog.rate(&[0], Rating::of(4));
        let rejected = |catalog: &Catalog| {
            ["a.jpg", "b.jpg"]
                .map(|name| catalog.mark_of(&Path::new("/shoot").join(name)).is_rejected)
        };

        catalog.toggle_rejected(&[0]);
        catalog.toggle_rejected(&[0, 1]);
        assert_eq!(rejected(&catalog), [true, true]);

        catalog.toggle_rejected(&[0, 1]);
        assert_eq!(rejected(&catalog), [false, false]);
        assert_eq!(catalog.mark_of(Path::new("/shoot/a.jpg")).rating.stars(), 4);
    }

    #[test]
    fn a_mark_belongs_to_the_photo_whatever_the_series() {
        let mut catalog = Catalog::default();
        catalog.import(series_of("/shoot", &["a.jpg"]));
        catalog.rate(&[0], Rating::of(2));

        catalog.import(Series {
            folder: None,
            ..series_of("/shoot", &["a.jpg"])
        });

        let selected = catalog.selected_photo().unwrap();
        assert_eq!(catalog.mark_of(selected).rating.stars(), 2);
    }

    #[test]
    fn a_photo_back_to_unrated_and_not_rejected_leaves_no_mark() {
        let mut catalog = Catalog::default();
        catalog.import(series_of("/shoot", &["a.jpg"]));
        let unmarked = catalog.clone();

        catalog.rate(&[0], Rating::of(5));
        catalog.rate(&[0], Rating::of(0));

        assert_eq!(catalog, unmarked);
    }

    #[test]
    fn marks_follow_the_photos_of_a_relocated_series() {
        let mut catalog = three_series();
        catalog.rate(&[0], Rating::of(3));

        catalog.relocate(0, Path::new("/moved"), |_| true);

        assert_eq!(catalog.mark_of(Path::new("/moved/1.jpg")).rating.stars(), 3);
    }

    fn names_of(photos: &[&Path]) -> Vec<String> {
        photos.iter().map(|photo| photo_name(photo)).collect()
    }

    fn names_of_photos(catalog: &Catalog) -> Vec<String> {
        let photos = catalog.photos().iter();
        photos.map(|photo| photo_name(photo)).collect()
    }

    #[test]
    fn removed_photos_leave_the_series_and_the_next_one_is_selected_then_the_previous() {
        let mut catalog = Catalog::default();
        catalog.import(series_of("/shoot", &["0.jpg", "1.jpg", "2.jpg", "3.jpg"]));
        catalog.select(1);

        catalog.remove_photos(&[1, 2]);
        assert_eq!(names_of_photos(&catalog), ["0.jpg", "3.jpg"]);
        assert_eq!(catalog.selected_photo(), Some(Path::new("/shoot/3.jpg")));

        catalog.remove_photos(&[1]);
        assert_eq!(catalog.selected_photo(), Some(Path::new("/shoot/0.jpg")));
    }

    #[test]
    fn removing_other_photos_keeps_the_selected_one_selected() {
        let mut catalog = Catalog::default();
        catalog.import(series_of("/shoot", &["0.jpg", "1.jpg", "2.jpg"]));
        catalog.select(2);

        catalog.remove_photos(&[0]);

        assert_eq!(catalog.selected_photo(), Some(Path::new("/shoot/2.jpg")));
    }

    #[test]
    fn photo_removed_from_a_folder_series_stays_out_when_the_folder_is_imported_again() {
        let mut catalog = Catalog::default();
        catalog.import(series_of("/shoot", &["0.jpg", "1.jpg"]));
        catalog.remove_photos(&[0]);

        catalog.import(series_of("/shoot", &["0.jpg", "1.jpg", "2.jpg"]));

        assert_eq!(names_of_photos(&catalog), ["1.jpg", "2.jpg"]);
    }

    #[test]
    fn trashed_photo_put_back_in_its_folder_joins_the_series_at_the_next_import() {
        let mut catalog = Catalog::default();
        catalog.import(series_of("/shoot", &["0.jpg", "1.jpg"]));
        catalog.drop_trashed_photos(&[0]);
        assert_eq!(names_of_photos(&catalog), ["1.jpg"]);

        catalog.import(series_of("/shoot", &["0.jpg", "1.jpg"]));

        assert_eq!(names_of_photos(&catalog), ["0.jpg", "1.jpg"]);
    }

    #[test]
    fn removed_photos_come_back_once_their_series_was_removed_and_imported_again() {
        let mut catalog = Catalog::default();
        catalog.import(series_of("/shoot", &["0.jpg", "1.jpg"]));
        catalog.remove_photos(&[0]);

        catalog.remove(0);
        catalog.import(series_of("/shoot", &["0.jpg", "1.jpg"]));

        assert_eq!(names_of_photos(&catalog), ["0.jpg", "1.jpg"]);
    }

    #[test]
    fn series_left_without_photo_leaves_the_catalog_and_its_neighbour_opens() {
        let mut catalog = three_series();
        catalog.open(1);

        catalog.remove_photos(&[0]);

        assert_eq!(names(&catalog), ["a", "c"]);
        assert_eq!(catalog.open_series().unwrap().name, "c");
    }

    fn rated_series() -> Catalog {
        let mut catalog = Catalog::default();
        catalog.import(series_of(
            "/shoot",
            &["0.jpg", "1.jpg", "2.jpg", "3.jpg", "4.jpg"],
        ));
        for (photo, stars) in [(1, 1), (2, 2), (3, 3)] {
            catalog.rate(&[photo], Rating::of(stars));
        }
        catalog.rate(&[4], Rating::of(5));
        catalog.toggle_rejected(&[4]);
        catalog
    }

    #[test]
    fn filter_shows_the_photos_rated_enough_and_not_rejected() {
        let mut catalog = rated_series();
        assert_eq!(catalog.shown_photos(), [0, 1, 2, 3, 4]);

        catalog.set_filter(SeriesFilter::at_least(2));

        assert_eq!(catalog.shown_photos(), [2, 3]);
    }

    #[test]
    fn rejected_photos_are_told_whatever_the_filter_shows() {
        let mut catalog = rated_series();

        catalog.set_filter(SeriesFilter::at_least(2));

        assert_eq!(catalog.rejected_photos(), [4]);
    }

    #[test]
    fn selected_photo_that_stops_being_shown_gives_way_to_the_next_then_the_previous() {
        let mut catalog = rated_series();
        catalog.set_filter(SeriesFilter::at_least(2));
        assert_eq!(catalog.selected_index(), Some(2));

        catalog.rate(&[2], Rating::of(0));
        assert_eq!(catalog.selected_index(), Some(3));

        catalog.rate(&[2], Rating::of(4));
        catalog.toggle_rejected(&[3]);
        assert_eq!(catalog.selected_index(), Some(2));
    }

    #[test]
    fn nothing_is_selected_when_nothing_is_shown_until_photos_are_shown_again() {
        let mut catalog = rated_series();

        catalog.set_filter(SeriesFilter::at_least(4));
        assert_eq!(catalog.shown_photos(), [] as [usize; 0]);
        assert_eq!(catalog.selected_index(), None);

        catalog.set_filter(SeriesFilter::All);
        assert_eq!(catalog.selected_index(), Some(0));
    }

    #[test]
    fn photos_beside_the_selected_one_are_the_shown_one_after_then_the_one_before() {
        let mut catalog = rated_series();
        catalog.set_filter(SeriesFilter::at_least(1));
        assert_eq!(names_of(&catalog.photos_beside_selected()), ["2.jpg"]);

        catalog.select_next();

        assert_eq!(
            names_of(&catalog.photos_beside_selected()),
            ["3.jpg", "1.jpg"]
        );
    }

    #[test]
    fn next_and_previous_move_among_the_shown_photos_and_stop_at_the_ends() {
        let mut catalog = rated_series();
        catalog.set_filter(SeriesFilter::at_least(1));
        assert_eq!(catalog.selected_index(), Some(1));

        catalog.select_previous();
        assert_eq!(catalog.selected_index(), Some(1));

        catalog.select_next();
        catalog.select_next();
        catalog.select_next();
        assert_eq!(catalog.selected_index(), Some(3));
    }

    #[test]
    fn each_series_keeps_its_own_filter() {
        let mut catalog = rated_series();
        catalog.set_filter(SeriesFilter::at_least(2));

        catalog.import(series_of("/other", &["a.jpg"]));
        assert_eq!(catalog.filter(), SeriesFilter::All);

        catalog.open(1);
        assert_eq!(catalog.filter(), SeriesFilter::at_least(2));
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
