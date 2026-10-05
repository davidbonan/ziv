use std::path::{Path, PathBuf};

use crate::library::domain::photo_trash::PhotoTrash;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TrashOutcome {
    /// The photos whose file went to the Trash.
    pub trashed: Vec<PathBuf>,
    /// How many photos stay where they are.
    pub failed_count: usize,
}

/// Moves each of `photos` to the Trash, then the files `kept_beside` it. A
/// photo whose own file cannot be moved keeps everything.
pub fn trash_photos(
    trash: &dyn PhotoTrash,
    photos: &[PathBuf],
    kept_beside: impl Fn(&Path) -> Vec<PathBuf>,
) -> TrashOutcome {
    let mut outcome = TrashOutcome::default();
    for photo in photos {
        if trash.move_to_trash(photo).is_err() {
            outcome.failed_count += 1;
            continue;
        }
        for file in kept_beside(photo) {
            // The photo is gone: a file left behind is an orphan, not a photo that failed.
            let _ = trash.move_to_trash(&file);
        }
        outcome.trashed.push(photo.clone());
    }
    outcome
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    #[derive(Default)]
    struct RecordingTrash {
        moved: RefCell<Vec<PathBuf>>,
        refused: Option<&'static str>,
    }

    impl PhotoTrash for RecordingTrash {
        fn move_to_trash(&self, file: &Path) -> Result<(), String> {
            if self
                .refused
                .is_some_and(|refused| file == Path::new(refused))
            {
                return Err("in use".to_owned());
            }
            self.moved.borrow_mut().push(file.to_owned());
            Ok(())
        }
    }

    fn sidecar_beside(photo: &Path) -> Vec<PathBuf> {
        vec![photo.with_extension("jpg.ziv.json")]
    }

    fn paths(names: &[&str]) -> Vec<PathBuf> {
        names.iter().map(PathBuf::from).collect()
    }

    #[test]
    fn each_photo_goes_to_the_trash_with_the_files_kept_beside_it() {
        let trash = RecordingTrash::default();

        let outcome = trash_photos(&trash, &paths(&["/p/a.jpg"]), sidecar_beside);

        assert_eq!(outcome.trashed, paths(&["/p/a.jpg"]));
        assert_eq!(outcome.failed_count, 0);
        assert_eq!(
            *trash.moved.borrow(),
            paths(&["/p/a.jpg", "/p/a.jpg.ziv.json"])
        );
    }

    #[test]
    fn a_photo_that_cannot_be_moved_keeps_its_files_and_is_counted() {
        let trash = RecordingTrash {
            refused: Some("/p/a.jpg"),
            ..RecordingTrash::default()
        };

        let outcome = trash_photos(&trash, &paths(&["/p/a.jpg", "/p/b.jpg"]), sidecar_beside);

        assert_eq!(outcome.trashed, paths(&["/p/b.jpg"]));
        assert_eq!(outcome.failed_count, 1);
        assert_eq!(
            *trash.moved.borrow(),
            paths(&["/p/b.jpg", "/p/b.jpg.ziv.json"])
        );
    }
}
