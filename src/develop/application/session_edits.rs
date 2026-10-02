use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::develop::domain::edit::Edit;
use crate::develop::domain::edit_history::EditHistory;
use crate::develop::domain::edit_storage::EditStorage;

/// How long an edit stays unchanged before it is stored.
pub const SAVE_DELAY_SECONDS: f64 = 0.5;

struct PhotoEdit {
    history: EditHistory,
    /// Why the stored edit could not be used; the photo is then left alone.
    unusable_storage: Option<String>,
}

struct Unsaved {
    photo: PathBuf,
    changed_at: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SaveFailure {
    pub photo: PathBuf,
    pub reason: String,
}

/// The edits of the photos seen in this session, kept stored as they change.
pub struct SessionEdits<Storage> {
    storage: Storage,
    photos: HashMap<PathBuf, PhotoEdit>,
    unsaved: Option<Unsaved>,
}

impl<Storage: EditStorage> SessionEdits<Storage> {
    pub fn new(storage: Storage) -> Self {
        Self {
            storage,
            photos: HashMap::new(),
            unsaved: None,
        }
    }

    /// Brings in the stored edit of a photo the first time it is seen.
    pub fn load(&mut self, photo: &Path) {
        if self.photos.contains_key(photo) {
            return;
        }
        let loaded = match self.storage.stored_edit(photo) {
            Ok(stored) => PhotoEdit {
                history: EditHistory::starting_from(stored.unwrap_or_default()),
                unusable_storage: None,
            },
            Err(reason) => PhotoEdit {
                history: EditHistory::default(),
                unusable_storage: Some(reason),
            },
        };
        self.photos.insert(photo.to_owned(), loaded);
    }

    pub fn edit_of(&self, photo: &Path) -> Edit {
        self.photos
            .get(photo)
            .map_or_else(Edit::default, |photo| photo.history.current().clone())
    }

    /// Why this photo cannot be edited, when what is stored for it is unusable.
    pub fn unusable_storage_of(&self, photo: &Path) -> Option<&str> {
        self.photos.get(photo)?.unusable_storage.as_deref()
    }

    fn editable(&mut self, photo: &Path) -> Option<&mut EditHistory> {
        let known = self.photos.get_mut(photo)?;
        known
            .unusable_storage
            .is_none()
            .then_some(&mut known.history)
    }

    /// Steps the photo's history and, when the edit changed, keeps it
    /// to be saved, saving first what was pending for another photo.
    fn stepped(
        &mut self,
        photo: &Path,
        now: f64,
        is_changed_by: impl FnOnce(&mut EditHistory) -> bool,
    ) -> Option<SaveFailure> {
        if !is_changed_by(self.editable(photo)?) {
            return None;
        }
        let is_another_photo_unsaved = self
            .unsaved
            .as_ref()
            .is_some_and(|unsaved| unsaved.photo != photo);
        let failure = if is_another_photo_unsaved {
            self.save_now()
        } else {
            None
        };
        self.unsaved = Some(Unsaved {
            photo: photo.to_owned(),
            changed_at: now,
        });
        failure
    }

    pub fn change(&mut self, photo: &Path, edit: Edit, now: f64) -> Option<SaveFailure> {
        self.stepped(photo, now, |history| history.change(edit))
    }

    pub fn undo(&mut self, photo: &Path, now: f64) -> Option<SaveFailure> {
        self.stepped(photo, now, EditHistory::undo)
    }

    pub fn redo(&mut self, photo: &Path, now: f64) -> Option<SaveFailure> {
        self.stepped(photo, now, EditHistory::redo)
    }

    /// While a gesture is ongoing, the changes of a photo make one undo step.
    pub fn set_gesture_ongoing(&mut self, photo: &Path, is_ongoing: bool) {
        if let Some(history) = self.editable(photo) {
            history.set_gesture_ongoing(is_ongoing);
        }
    }

    pub fn seconds_until_save(&self, now: f64) -> Option<f64> {
        let unsaved = self.unsaved.as_ref()?;
        Some((unsaved.changed_at + SAVE_DELAY_SECONDS - now).max(0.0))
    }

    pub fn save_when_due(&mut self, now: f64) -> Option<SaveFailure> {
        if self.seconds_until_save(now)? > 0.0 {
            return None;
        }
        self.save_now()
    }

    pub fn save_now(&mut self) -> Option<SaveFailure> {
        let Unsaved { photo, .. } = self.unsaved.take()?;
        let reason = self
            .storage
            .store_edit(&photo, &self.edit_of(&photo))
            .err()?;
        Some(SaveFailure { photo, reason })
    }

    /// Saves what is pending, then forgets every photo.
    pub fn close_session(&mut self) -> Option<SaveFailure> {
        let failure = self.save_now();
        self.photos.clear();
        failure
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::develop::domain::adjustments::Adjustments;

    const PHOTO: &str = "shoot/a.arw";
    const OTHER_PHOTO: &str = "shoot/b.arw";

    #[derive(Default)]
    struct MemoryStorage {
        stored: RefCell<HashMap<PathBuf, Result<Edit, String>>>,
        refuses_writes: bool,
    }

    impl EditStorage for &MemoryStorage {
        fn stored_edit(&self, photo: &Path) -> Result<Option<Edit>, String> {
            self.stored.borrow().get(photo).cloned().transpose()
        }

        fn store_edit(&self, photo: &Path, edit: &Edit) -> Result<(), String> {
            if self.refuses_writes {
                return Err("read-only folder".to_owned());
            }
            if *edit == Edit::default() {
                self.stored.borrow_mut().remove(photo);
            } else {
                self.stored
                    .borrow_mut()
                    .insert(photo.to_owned(), Ok(edit.clone()));
            }
            Ok(())
        }
    }

    fn brighter() -> Edit {
        Edit::from(Adjustments {
            exposure: 1.0,
            ..Adjustments::default()
        })
    }

    fn storage_holding(photo: &str, stored: Result<Edit, String>) -> MemoryStorage {
        let storage = MemoryStorage::default();
        storage
            .stored
            .borrow_mut()
            .insert(PathBuf::from(photo), stored);
        storage
    }

    fn stored(storage: &MemoryStorage, photo: &str) -> Option<Edit> {
        (&storage).stored_edit(Path::new(photo)).unwrap()
    }

    fn loaded<'a>(storage: &'a MemoryStorage, photos: &[&str]) -> SessionEdits<&'a MemoryStorage> {
        let mut edits = SessionEdits::new(storage);
        for photo in photos {
            edits.load(Path::new(photo));
        }
        edits
    }

    #[test]
    fn a_photo_comes_with_its_stored_edit() {
        let storage = storage_holding(PHOTO, Ok(brighter()));

        let edits = loaded(&storage, &[PHOTO, OTHER_PHOTO]);

        assert_eq!(edits.edit_of(Path::new(PHOTO)), brighter());
        assert_eq!(edits.edit_of(Path::new(OTHER_PHOTO)), Edit::default());
    }

    #[test]
    fn a_change_is_stored_once_it_has_settled() {
        let storage = MemoryStorage::default();
        let mut edits = loaded(&storage, &[PHOTO]);

        edits.change(Path::new(PHOTO), brighter(), 10.0);
        edits.save_when_due(10.0 + SAVE_DELAY_SECONDS / 2.0);
        assert_eq!(stored(&storage, PHOTO), None);

        edits.save_when_due(10.0 + SAVE_DELAY_SECONDS);
        assert_eq!(stored(&storage, PHOTO), Some(brighter()));
    }

    #[test]
    fn an_edit_back_to_default_leaves_nothing_stored() {
        let storage = storage_holding(PHOTO, Ok(brighter()));
        let mut edits = loaded(&storage, &[PHOTO]);

        edits.change(Path::new(PHOTO), Edit::default(), 10.0);
        edits.save_now();

        assert_eq!(stored(&storage, PHOTO), None);
    }

    #[test]
    fn changing_another_photo_stores_the_pending_one_first() {
        let storage = MemoryStorage::default();
        let mut edits = loaded(&storage, &[PHOTO, OTHER_PHOTO]);

        edits.change(Path::new(PHOTO), brighter(), 10.0);
        edits.change(Path::new(OTHER_PHOTO), brighter(), 10.1);

        assert_eq!(stored(&storage, PHOTO), Some(brighter()));
    }

    #[test]
    fn a_failed_save_is_reported_and_the_edit_stays() {
        let storage = MemoryStorage {
            refuses_writes: true,
            ..MemoryStorage::default()
        };
        let mut edits = loaded(&storage, &[PHOTO]);

        edits.change(Path::new(PHOTO), brighter(), 10.0);
        let failure = edits.save_now();

        let expected = SaveFailure {
            photo: PathBuf::from(PHOTO),
            reason: "read-only folder".to_owned(),
        };
        assert_eq!(failure, Some(expected));
        assert_eq!(edits.edit_of(Path::new(PHOTO)), brighter());
        assert_eq!(edits.save_now(), None);
    }

    #[test]
    fn a_photo_with_unusable_storage_is_left_alone() {
        let storage = storage_holding(PHOTO, Err("written by a newer ziv".to_owned()));
        let mut edits = loaded(&storage, &[PHOTO]);

        edits.change(Path::new(PHOTO), brighter(), 10.0);
        edits.save_now();

        assert_eq!(
            edits.unusable_storage_of(Path::new(PHOTO)),
            Some("written by a newer ziv")
        );
        assert_eq!(edits.edit_of(Path::new(PHOTO)), Edit::default());
        assert!((&storage).stored_edit(Path::new(PHOTO)).is_err());
    }

    #[test]
    fn an_undone_edit_is_stored_like_any_change_and_each_photo_has_its_own_history() {
        let storage = MemoryStorage::default();
        let mut edits = loaded(&storage, &[PHOTO, OTHER_PHOTO]);
        edits.change(Path::new(PHOTO), brighter(), 10.0);
        edits.change(Path::new(OTHER_PHOTO), brighter(), 11.0);

        edits.undo(Path::new(PHOTO), 12.0);
        edits.save_now();

        assert_eq!(stored(&storage, PHOTO), None);
        assert_eq!(edits.edit_of(Path::new(OTHER_PHOTO)), brighter());

        edits.redo(Path::new(PHOTO), 13.0);
        assert_eq!(edits.edit_of(Path::new(PHOTO)), brighter());
    }

    #[test]
    fn closing_the_session_stores_what_is_pending() {
        let storage = MemoryStorage::default();
        let mut edits = loaded(&storage, &[PHOTO]);

        edits.change(Path::new(PHOTO), brighter(), 10.0);
        edits.close_session();

        assert_eq!(stored(&storage, PHOTO), Some(brighter()));
    }
}
