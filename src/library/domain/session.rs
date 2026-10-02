use std::path::{Path, PathBuf};

use crate::photo::domain::photo_name::photo_name;

use super::natural_order::natural_order;

/// The photos currently open, in display order, and the one being looked at.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Session {
    photos: Vec<PathBuf>,
    selected: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoPhotoFound;

impl Session {
    /// Replaces the open photos by `photos`, sorted by file name, the first one
    /// selected. Nothing changes when `photos` is empty.
    pub fn open(&mut self, mut photos: Vec<PathBuf>) -> Result<(), NoPhotoFound> {
        if photos.is_empty() {
            return Err(NoPhotoFound);
        }
        photos.sort_by(|left, right| {
            natural_order(&photo_name(left), &photo_name(right)).then_with(|| left.cmp(right))
        });
        photos.dedup();
        self.photos = photos;
        self.selected = Some(0);
        Ok(())
    }

    pub fn photos(&self) -> &[PathBuf] {
        &self.photos
    }

    pub fn selected_index(&self) -> Option<usize> {
        self.selected
    }

    pub fn selected_photo(&self) -> Option<&Path> {
        self.selected.map(|index| self.photos[index].as_path())
    }

    pub fn index_of(&self, photo: &Path) -> Option<usize> {
        self.photos.iter().position(|path| path == photo)
    }

    /// Selects the photo at `index`; an index outside the session changes nothing.
    pub fn select(&mut self, index: usize) {
        if index < self.photos.len() {
            self.selected = Some(index);
        }
    }

    /// Stops at the first photo.
    pub fn select_previous(&mut self) {
        if let Some(index) = self.selected {
            self.select(index.saturating_sub(1));
        }
    }

    /// Stops at the last photo.
    pub fn select_next(&mut self) {
        if let Some(index) = self.selected {
            self.select(index + 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(names: &[&str]) -> Vec<PathBuf> {
        names.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn opened_photos_are_in_natural_name_order_with_the_first_selected() {
        let mut session = Session::default();

        session
            .open(paths(&["/b/DSC10.ARW", "/a/dsc2.arw", "/b/DSC1.ARW"]))
            .unwrap();

        assert_eq!(
            session.photos(),
            paths(&["/b/DSC1.ARW", "/a/dsc2.arw", "/b/DSC10.ARW"])
        );
        assert_eq!(session.selected_photo(), Some(Path::new("/b/DSC1.ARW")));
    }

    #[test]
    fn opening_replaces_the_previous_photos() {
        let mut session = Session::default();
        session.open(paths(&["/old/a.jpg", "/old/b.jpg"])).unwrap();

        session.open(paths(&["/new/c.jpg"])).unwrap();

        assert_eq!(session.photos(), paths(&["/new/c.jpg"]));
    }

    #[test]
    fn opening_nothing_keeps_the_session_as_it_was() {
        let mut session = Session::default();
        session.open(paths(&["/old/a.jpg"])).unwrap();

        let outcome = session.open(Vec::new());

        assert_eq!(outcome, Err(NoPhotoFound));
        assert_eq!(session.selected_photo(), Some(Path::new("/old/a.jpg")));
    }

    #[test]
    fn the_same_file_given_twice_is_opened_once() {
        let mut session = Session::default();

        session.open(paths(&["/a/x.jpg", "/a/x.jpg"])).unwrap();

        assert_eq!(session.photos().len(), 1);
    }

    #[test]
    fn empty_session_has_no_selected_photo() {
        assert_eq!(Session::default().selected_photo(), None);
    }

    fn three_photos() -> Session {
        let mut session = Session::default();
        session
            .open(paths(&["/p/1.jpg", "/p/2.jpg", "/p/3.jpg"]))
            .unwrap();
        session
    }

    #[test]
    fn next_and_previous_move_the_selection_by_one() {
        let mut session = three_photos();

        session.select_next();
        assert_eq!(session.selected_index(), Some(1));

        session.select_previous();
        assert_eq!(session.selected_index(), Some(0));
    }

    #[test]
    fn selection_stops_at_both_ends() {
        let mut session = three_photos();

        session.select_previous();
        assert_eq!(session.selected_index(), Some(0));

        session.select(2);
        session.select_next();
        assert_eq!(session.selected_index(), Some(2));
    }

    #[test]
    fn selecting_outside_the_session_changes_nothing() {
        let mut session = three_photos();

        session.select(7);

        assert_eq!(session.selected_index(), Some(0));
    }
}
