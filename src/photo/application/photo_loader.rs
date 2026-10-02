use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

use crate::photo::domain::decode_error::DecodeError;

pub struct LoadedPhoto<Photo> {
    pub path: PathBuf,
    pub result: Result<Photo, DecodeError>,
}

/// What the worker does with requests that piled up while it was busy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backlog {
    /// Load every request, oldest first.
    LoadAll,
    /// Load only the newest request; the ones before it are dropped.
    LoadNewestOnly,
}

/// Loads photos on a worker thread.
pub struct PhotoLoader<Photo> {
    requests: Sender<PathBuf>,
    loaded: Receiver<LoadedPhoto<Photo>>,
}

// Third-party decoders panic on files they do not expect; that is one failed photo, not a dead worker.
fn load_without_unwinding<Photo>(
    load: &impl Fn(&Path) -> Result<Photo, DecodeError>,
    path: &Path,
) -> Result<Photo, DecodeError> {
    catch_unwind(AssertUnwindSafe(|| load(path)))
        .unwrap_or_else(|_| Err(DecodeError::new("the decoder crashed on this file")))
}

impl<Photo: Send + 'static> PhotoLoader<Photo> {
    /// `load` runs on the worker; `notify` is called from it each time a photo is ready to be taken.
    pub fn spawn(
        backlog: Backlog,
        load: impl Fn(&Path) -> Result<Photo, DecodeError> + Send + 'static,
        notify: impl Fn() + Send + 'static,
    ) -> Self {
        let (requests, pending) = channel::<PathBuf>();
        let (results, loaded) = channel();
        thread::Builder::new()
            .name("ziv photo loader".to_owned())
            .spawn(move || {
                for path in &pending {
                    let path = match backlog {
                        Backlog::LoadAll => path,
                        Backlog::LoadNewestOnly => pending.try_iter().last().unwrap_or(path),
                    };
                    let result = load_without_unwinding(&load, &path);
                    if results.send(LoadedPhoto { path, result }).is_err() {
                        return;
                    }
                    notify();
                }
            })
            .expect("the OS can start a thread");
        Self { requests, loaded }
    }

    pub fn request(&self, path: PathBuf) {
        self.requests
            .send(path)
            .expect("the worker lives as long as the loader");
    }

    /// Photos loaded since the last call; never blocks.
    pub fn take_loaded(&self) -> Vec<LoadedPhoto<Photo>> {
        self.loaded.try_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    const PATIENCE: Duration = Duration::from_secs(5);

    /// Loads only when the test lets it, so "still loading" is observable.
    struct GatedLoader {
        loader: PhotoLoader<&'static str>,
        started: Receiver<PathBuf>,
        open_gate: Sender<()>,
        notified: Receiver<()>,
    }

    fn gated_loader() -> GatedLoader {
        gated_loader_with(Backlog::LoadAll)
    }

    fn gated_loader_with(backlog: Backlog) -> GatedLoader {
        let (open_gate, gate) = channel();
        let (notify, notified) = channel();
        let (start, started) = channel();
        let load = move |path: &Path| {
            start.send(path.to_owned()).unwrap();
            gate.recv_timeout(PATIENCE).unwrap();
            match path.to_str() {
                Some("broken") => Err(DecodeError::new("corrupt file")),
                Some("crashing") => panic!("decoder bug"),
                _ => Ok("decoded"),
            }
        };
        let loader = PhotoLoader::spawn(backlog, load, move || {
            let _ = notify.send(());
        });
        GatedLoader {
            loader,
            started,
            open_gate,
            notified,
        }
    }

    impl GatedLoader {
        fn finish_one_load(&self) -> LoadedPhoto<&'static str> {
            self.open_gate.send(()).unwrap();
            self.notified.recv_timeout(PATIENCE).unwrap();
            self.loader.take_loaded().pop().unwrap()
        }
    }

    #[test]
    fn requesting_a_photo_does_not_wait_for_its_loading() {
        let gated = gated_loader();

        gated.loader.request(PathBuf::from("photo"));

        assert!(gated.loader.take_loaded().is_empty());
    }

    #[test]
    fn loaded_photo_is_handed_over_after_the_notification() {
        let gated = gated_loader();
        gated.loader.request(PathBuf::from("photo"));

        let loaded = gated.finish_one_load();

        assert_eq!(loaded.path, PathBuf::from("photo"));
        assert_eq!(loaded.result, Ok("decoded"));
    }

    #[test]
    fn failed_loading_is_reported_with_its_reason() {
        let gated = gated_loader();
        gated.loader.request(PathBuf::from("broken"));

        let loaded = gated.finish_one_load();

        assert_eq!(loaded.result.unwrap_err().to_string(), "corrupt file");
    }

    #[test]
    fn crashing_decoder_fails_that_photo_and_keeps_loading_the_next() {
        let gated = gated_loader();
        gated.loader.request(PathBuf::from("crashing"));
        gated.loader.request(PathBuf::from("photo"));

        let crashed = gated.finish_one_load();
        let next = gated.finish_one_load();

        assert!(crashed.result.is_err());
        assert_eq!(next.result, Ok("decoded"));
    }

    #[test]
    fn newest_only_backlog_skips_requests_made_obsolete_while_busy() {
        let gated = gated_loader_with(Backlog::LoadNewestOnly);
        gated.loader.request(PathBuf::from("busy"));
        gated.started.recv_timeout(PATIENCE).unwrap();
        gated.loader.request(PathBuf::from("obsolete"));
        gated.loader.request(PathBuf::from("newest"));

        let loaded = [gated.finish_one_load(), gated.finish_one_load()];

        assert_eq!(
            loaded.map(|photo| photo.path),
            ["busy", "newest"].map(PathBuf::from)
        );
    }
}
