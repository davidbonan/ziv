use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::photo::application::photo_loader::{Backlog, LoadedPhoto, PhotoLoader};
use crate::photo::domain::decode_error::DecodeError;

/// Photos loaded before they are asked for, on a worker thread: the ones
/// wanted at the moment, and no other.
pub struct PhotosAhead<Photo> {
    /// A request the worker reaches once it is no longer wanted loads nothing.
    loader: PhotoLoader<Option<Photo>>,
    wanted: Arc<Mutex<Vec<PathBuf>>>,
    loading: HashSet<PathBuf>,
    ready: HashMap<PathBuf, Photo>,
}

impl<Photo: Send + 'static> PhotosAhead<Photo> {
    /// `load` runs on the worker; `notify` is called from it after each request it went through.
    pub fn spawn(
        load: impl Fn(&Path) -> Result<Photo, DecodeError> + Send + 'static,
        notify: impl Fn() + Send + 'static,
    ) -> Self {
        let wanted = Arc::new(Mutex::new(Vec::new()));
        let load_if_wanted = {
            let wanted = wanted.clone();
            move |path: &Path| {
                if !is_wanted(&wanted, path) {
                    return Ok(None);
                }
                load(path).map(Some)
            }
        };
        Self {
            loader: PhotoLoader::spawn(Backlog::LoadAll, load_if_wanted, notify),
            wanted,
            loading: HashSet::new(),
            ready: HashMap::new(),
        }
    }

    /// Loads `photos` in that order, unless ready or loading already; every other photo is let go.
    pub fn want(&mut self, photos: Vec<PathBuf>) {
        self.receive();
        self.ready.retain(|path, _| photos.contains(path));
        self.loading.retain(|path| photos.contains(path));
        *self
            .wanted
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = photos.clone();
        for photo in photos {
            if self.ready.contains_key(&photo) || self.loading.contains(&photo) {
                continue;
            }
            self.loading.insert(photo.clone());
            self.loader.request(photo);
        }
    }

    /// The photo when it is ready; it is no longer kept here.
    pub fn take(&mut self, path: &Path) -> Option<Photo> {
        self.receive();
        self.ready.remove(path)
    }

    /// Keeps a photo loaded elsewhere until the next `want` says whether it is wanted.
    pub fn keep(&mut self, path: PathBuf, photo: Photo) {
        self.ready.insert(path, photo);
    }

    fn receive(&mut self) {
        for LoadedPhoto { path, result } in self.loader.take_loaded() {
            if !self.loading.remove(&path) {
                continue;
            }
            if let Ok(Some(photo)) = result {
                self.ready.insert(path, photo);
            }
        }
    }
}

fn is_wanted(wanted: &Mutex<Vec<PathBuf>>, path: &Path) -> bool {
    wanted
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .iter()
        .any(|wanted| wanted == path)
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::{Receiver, Sender, channel};
    use std::time::Duration;

    use super::*;

    const PATIENCE: Duration = Duration::from_secs(5);

    /// Loads only when the test lets it, and tells which photos it loaded.
    struct GatedPhotosAhead {
        ahead: PhotosAhead<&'static str>,
        loads: Receiver<PathBuf>,
        open_gate: Sender<()>,
        notified: Receiver<()>,
    }

    fn gated_photos_ahead() -> GatedPhotosAhead {
        let (open_gate, gate) = channel();
        let (notify, notified) = channel();
        let (tell_load, loads) = channel();
        let load = move |path: &Path| {
            tell_load.send(path.to_owned()).unwrap();
            gate.recv_timeout(PATIENCE).unwrap();
            match path.to_str() {
                Some("broken") => Err(DecodeError::new("corrupt file")),
                _ => Ok("developed"),
            }
        };
        let ahead = PhotosAhead::spawn(load, move || {
            let _ = notify.send(());
        });
        GatedPhotosAhead {
            ahead,
            loads,
            open_gate,
            notified,
        }
    }

    impl GatedPhotosAhead {
        fn want(&mut self, photos: &[&str]) {
            self.ahead.want(photos.iter().map(PathBuf::from).collect());
        }

        fn take(&mut self, photo: &str) -> Option<&'static str> {
            self.ahead.take(Path::new(photo))
        }

        fn let_requests_through(&self, count: usize) {
            for _ in 0..count {
                self.open_gate.send(()).unwrap();
                self.notified.recv_timeout(PATIENCE).unwrap();
            }
        }

        fn loaded(&self) -> Vec<PathBuf> {
            self.loads.try_iter().collect()
        }
    }

    #[test]
    fn wanted_photo_is_taken_once_loaded_and_only_once() {
        let mut gated = gated_photos_ahead();
        gated.want(&["next"]);
        assert_eq!(gated.take("next"), None);

        gated.let_requests_through(1);

        assert_eq!(gated.take("next"), Some("developed"));
        assert_eq!(gated.take("next"), None);
    }

    #[test]
    fn photo_no_longer_wanted_when_its_turn_comes_is_not_loaded() {
        let mut gated = gated_photos_ahead();
        gated.want(&["busy", "obsolete"]);
        gated.loads.recv_timeout(PATIENCE).unwrap();
        gated.want(&["newest"]);

        gated.let_requests_through(3);

        assert_eq!(gated.loaded(), [PathBuf::from("newest")]);
        assert_eq!(gated.take("busy"), None);
        assert_eq!(gated.take("newest"), Some("developed"));
    }

    #[test]
    fn photo_still_wanted_is_not_loaded_again() {
        let mut gated = gated_photos_ahead();
        gated.want(&["next"]);
        gated.let_requests_through(1);

        gated.want(&["next", "previous"]);
        gated.let_requests_through(1);

        assert_eq!(gated.loaded(), ["next", "previous"].map(PathBuf::from));
    }

    #[test]
    fn kept_photo_stays_while_wanted_and_is_let_go_after() {
        let mut gated = gated_photos_ahead();
        gated.ahead.keep(PathBuf::from("previous"), "kept");
        gated.want(&["previous"]);
        gated.ahead.keep(PathBuf::from("other"), "kept");
        gated.want(&["previous"]);

        assert_eq!(gated.take("other"), None);
        assert_eq!(gated.take("previous"), Some("kept"));
        assert!(gated.loaded().is_empty());
    }

    #[test]
    fn photo_that_fails_to_load_is_not_ready() {
        let mut gated = gated_photos_ahead();
        gated.want(&["broken"]);

        gated.let_requests_through(1);

        assert_eq!(gated.take("broken"), None);
    }
}
