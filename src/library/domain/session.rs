use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::photo::domain::photo_name::photo_name;

use super::natural_order::natural_order;

/// The photos of a series, in display order, and the one being looked at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    photos: Vec<PathBuf>,
    selected: Option<usize>,
}

fn in_display_order(mut photos: Vec<PathBuf>) -> Vec<PathBuf> {
    photos.sort_by(|left, right| {
        natural_order(&photo_name(left), &photo_name(right)).then_with(|| left.cmp(right))
    });
    photos.dedup();
    photos
}

impl Session {
    /// `photos` sorted by file name, the first one selected. `None` when there is no photo.
    pub fn of(photos: Vec<PathBuf>) -> Option<Self> {
        (!photos.is_empty()).then(|| Self {
            photos: in_display_order(photos),
            selected: Some(0),
        })
    }

    /// Brings in the photos of `photos` that are not there yet; the selected photo stays selected.
    pub fn add(&mut self, photos: Vec<PathBuf>) {
        let selected = self.selected_photo().map(Path::to_owned);
        let mut all = std::mem::take(&mut self.photos);
        all.extend(photos);
        self.photos = in_display_order(all);
        self.selected = selected.and_then(|photo| self.index_of(&photo));
    }

    /// Points every photo whose file name `is_there` in `folder` to that
    /// folder; the selected photo stays selected. Returns how many were found.
    pub fn relocate(&mut self, folder: &Path, is_there: impl Fn(&Path) -> bool) -> usize {
        let selected = self.selected;
        let mut found = 0;
        let mut selected_photo = None;
        for (index, photo) in self.photos.iter_mut().enumerate() {
            let there = folder.join(photo.file_name().unwrap_or_default());
            if is_there(&there) {
                *photo = there;
                found += 1;
            }
            if selected == Some(index) {
                selected_photo = Some(photo.clone());
            }
        }
        self.photos = in_display_order(std::mem::take(&mut self.photos));
        self.selected = selected_photo.and_then(|photo| self.index_of(&photo));
        found
    }

    pub fn is_selection_inside(&self) -> bool {
        self.selected.is_none_or(|index| index < self.photos.len())
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
    fn photos_are_in_natural_name_order_with_the_first_selected() {
        let session = Session::of(paths(&["/b/DSC10.ARW", "/a/dsc2.arw", "/b/DSC1.ARW"])).unwrap();

        assert_eq!(
            session.photos(),
            paths(&["/b/DSC1.ARW", "/a/dsc2.arw", "/b/DSC10.ARW"])
        );
        assert_eq!(session.selected_photo(), Some(Path::new("/b/DSC1.ARW")));
    }

    #[test]
    fn no_photo_makes_no_session() {
        assert_eq!(Session::of(Vec::new()), None);
    }

    #[test]
    fn the_same_file_given_twice_is_there_once() {
        let session = Session::of(paths(&["/a/x.jpg", "/a/x.jpg"])).unwrap();

        assert_eq!(session.photos().len(), 1);
    }

    #[test]
    fn added_photos_take_their_place_and_the_selected_photo_stays_selected() {
        let mut session = Session::of(paths(&["/p/2.jpg", "/p/3.jpg"])).unwrap();
        session.select(1);

        session.add(paths(&["/p/1.jpg", "/p/3.jpg", "/p/4.jpg"]));

        assert_eq!(
            session.photos(),
            paths(&["/p/1.jpg", "/p/2.jpg", "/p/3.jpg", "/p/4.jpg"])
        );
        assert_eq!(session.selected_photo(), Some(Path::new("/p/3.jpg")));
    }

    #[test]
    fn relocation_moves_the_photos_found_and_leaves_the_others() {
        let mut session = Session::of(paths(&["/old/1.jpg", "/old/2.jpg", "/old/3.jpg"])).unwrap();
        session.select(2);

        let found = session.relocate(Path::new("/new"), |photo| photo != Path::new("/new/2.jpg"));

        assert_eq!(found, 2);
        assert_eq!(
            session.photos(),
            paths(&["/new/1.jpg", "/old/2.jpg", "/new/3.jpg"])
        );
        assert_eq!(session.selected_photo(), Some(Path::new("/new/3.jpg")));
    }

    fn three_photos() -> Session {
        Session::of(paths(&["/p/1.jpg", "/p/2.jpg", "/p/3.jpg"])).unwrap()
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
