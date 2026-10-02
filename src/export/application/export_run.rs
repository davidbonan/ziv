use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExportProgress {
    pub exported: usize,
    pub failed: usize,
    pub total: usize,
    /// Every photo was handled, or the export was cancelled.
    pub is_over: bool,
}

/// One export of several photos, running on its own thread.
pub struct ExportRun {
    progress: Arc<Mutex<ExportProgress>>,
    is_cancel_asked: Arc<AtomicBool>,
}

fn updated(progress: &Mutex<ExportProgress>, update: impl FnOnce(&mut ExportProgress)) {
    let mut progress = progress
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    update(&mut progress);
}

impl ExportRun {
    /// `export_one` writes one photo; a photo it fails on, or panics on, is
    /// counted as failed and the others still go. `notify` runs after each photo.
    pub fn start<Exported>(
        photos: Vec<PathBuf>,
        export_one: impl Fn(&Path) -> Result<Exported, String> + Send + 'static,
        notify: impl Fn() + Send + 'static,
    ) -> Self {
        let total = photos.len();
        let progress = Arc::new(Mutex::new(ExportProgress {
            total,
            ..ExportProgress::default()
        }));
        let is_cancel_asked = Arc::new(AtomicBool::new(false));
        let run = Self {
            progress: progress.clone(),
            is_cancel_asked: is_cancel_asked.clone(),
        };
        thread::spawn(move || {
            for photo in &photos {
                if is_cancel_asked.load(Ordering::Relaxed) {
                    break;
                }
                let is_exported = matches!(
                    catch_unwind(AssertUnwindSafe(|| export_one(photo))),
                    Ok(Ok(_))
                );
                updated(&progress, |progress| match is_exported {
                    true => progress.exported += 1,
                    false => progress.failed += 1,
                });
                notify();
            }
            updated(&progress, |progress| progress.is_over = true);
            notify();
        });
        run
    }

    pub fn progress(&self) -> ExportProgress {
        *self
            .progress
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Stops after the photo being written.
    pub fn cancel(&self) {
        self.is_cancel_asked.store(true, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::{Sender, channel};
    use std::time::Duration;

    use super::*;

    const PATIENCE: Duration = Duration::from_secs(5);

    fn photos(names: &[&str]) -> Vec<PathBuf> {
        names.iter().map(PathBuf::from).collect()
    }

    fn over(run: &ExportRun, notified: &std::sync::mpsc::Receiver<()>) -> ExportProgress {
        while !run.progress().is_over {
            notified.recv_timeout(PATIENCE).expect("the export ends");
        }
        run.progress()
    }

    fn notifying(notified: Sender<()>) -> impl Fn() + Send + 'static {
        move || {
            let _ = notified.send(());
        }
    }

    #[test]
    fn every_photo_is_exported_and_failures_are_counted_apart() {
        let (notify, notified) = channel();
        let export_one = |photo: &Path| match photo.to_str() {
            Some("broken.jpg") => Err("not a photo".to_owned()),
            Some("cursed.jpg") => panic!("encoder bug"),
            _ => Ok(()),
        };

        let run = ExportRun::start(
            photos(&["a.arw", "broken.jpg", "cursed.jpg", "b.arw"]),
            export_one,
            notifying(notify),
        );

        let expected = ExportProgress {
            exported: 2,
            failed: 2,
            total: 4,
            is_over: true,
        };
        assert_eq!(over(&run, &notified), expected);
    }

    #[test]
    fn cancel_stops_after_the_photo_being_written() {
        let (notify, notified) = channel();
        let (started, has_started) = channel();
        let (resume, resumed) = channel::<()>();
        let export_one = move |_: &Path| {
            let _ = started.send(());
            let _ = resumed.recv_timeout(PATIENCE);
            Ok(())
        };
        let run = ExportRun::start(
            photos(&["a.arw", "b.arw", "c.arw"]),
            export_one,
            notifying(notify),
        );

        has_started
            .recv_timeout(PATIENCE)
            .expect("first photo starts");
        run.cancel();
        resume.send(()).unwrap();

        let expected = ExportProgress {
            exported: 1,
            failed: 0,
            total: 3,
            is_over: true,
        };
        assert_eq!(over(&run, &notified), expected);
    }
}
