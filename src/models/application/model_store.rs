use std::fmt;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::models::domain::model::{Model, RemoteFile};
use crate::models::domain::model_source::ModelSource;

const BEING_DOWNLOADED_SUFFIX: &str = ".part";
const CHUNK_SIZE: usize = 1 << 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
    Download(String),
    Corrupted,
    Storage(String),
}

impl fmt::Display for ModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Download(reason) => write!(formatter, "it could not be downloaded ({reason})"),
            Self::Corrupted => write!(formatter, "the downloaded file is corrupted"),
            Self::Storage(reason) => write!(formatter, "it could not be stored ({reason})"),
        }
    }
}

/// The models on this machine: each is downloaded the first time it is asked
/// for, checked, then kept.
pub struct ModelStore<Source> {
    folder: PathBuf,
    source: Source,
}

fn hexadecimal(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn storage_error(error: io::Error) -> ModelError {
    ModelError::Storage(error.to_string())
}

// Copies the download to `file` and returns the SHA-256 of what was copied.
fn copy_hashing(
    mut download: Box<dyn Read>,
    file: &mut File,
    on_received: &mut dyn FnMut(u64),
) -> Result<String, ModelError> {
    let mut hash = Sha256::new();
    let mut chunk = vec![0; CHUNK_SIZE];
    let mut received = 0;
    loop {
        let read = download
            .read(&mut chunk)
            .map_err(|error| ModelError::Download(error.to_string()))?;
        if read == 0 {
            return Ok(hexadecimal(&hash.finalize()));
        }
        hash.update(&chunk[..read]);
        file.write_all(&chunk[..read]).map_err(storage_error)?;
        received += read as u64;
        on_received(received);
    }
}

impl<Source: ModelSource> ModelStore<Source> {
    pub fn new(folder: PathBuf, source: Source) -> Self {
        Self { folder, source }
    }

    fn download_checked(
        &self,
        remote: &RemoteFile,
        being_downloaded: &Path,
        on_received: &mut dyn FnMut(u64),
    ) -> Result<(), ModelError> {
        let download = self.source.download(remote).map_err(ModelError::Download)?;
        fs::create_dir_all(&self.folder).map_err(storage_error)?;
        let mut file = File::create(being_downloaded).map_err(storage_error)?;
        let sha256 = copy_hashing(download, &mut file, on_received)?;
        if sha256 != remote.sha256 {
            return Err(ModelError::Corrupted);
        }
        file.sync_all().map_err(storage_error)
    }

    /// The file an inference opens for `model`, downloaded first when it is
    /// not on this machine. `on_received` is told how many bytes have arrived.
    pub fn model_file(
        &self,
        model: &Model,
        on_received: &mut dyn FnMut(u64),
    ) -> Result<PathBuf, ModelError> {
        let file = self.folder.join(model.file_name());
        if file.is_file() {
            return Ok(file);
        }
        let being_downloaded = self
            .folder
            .join(model.file_name() + BEING_DOWNLOADED_SUFFIX);
        let downloaded = self
            .download_checked(&model.file, &being_downloaded, on_received)
            .and_then(|()| fs::rename(&being_downloaded, &file).map_err(storage_error));
        if downloaded.is_err() {
            let _ = fs::remove_file(&being_downloaded);
        }
        downloaded.map(|()| file)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::io::Cursor;

    use super::*;

    const WEIGHTS: &[u8] = b"model weights";
    const MODEL: Model = Model {
        name: "subject",
        file: RemoteFile {
            url: "https://example.invalid/model.onnx",
            sha256: "a2d42c4aa884e21216cbb8da4c7ba2fcf9b6033b2331666e666145c24caf7a38",
            size: WEIGHTS.len() as u64,
        },
    };

    struct CutShort;

    impl Read for CutShort {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("connection reset"))
        }
    }

    enum Served {
        Weights(&'static [u8]),
        CutShort,
        Unreachable,
    }

    struct Source {
        served: Served,
        downloads: Cell<u32>,
    }

    impl ModelSource for &Source {
        fn download(&self, _: &RemoteFile) -> Result<Box<dyn Read>, String> {
            self.downloads.set(self.downloads.get() + 1);
            match self.served {
                Served::Weights(weights) => Ok(Box::new(Cursor::new(weights))),
                Served::CutShort => Ok(Box::new(Cursor::new(&WEIGHTS[..4]).chain(CutShort))),
                Served::Unreachable => Err("no network".to_owned()),
            }
        }
    }

    fn serving(served: Served) -> Source {
        Source {
            served,
            downloads: Cell::new(0),
        }
    }

    fn files_of(folder: &Path) -> Vec<String> {
        let Ok(entries) = fs::read_dir(folder) else {
            return Vec::new();
        };
        entries
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn model_is_downloaded_once_then_reused() {
        let folder = tempfile::tempdir().unwrap();
        let source = serving(Served::Weights(WEIGHTS));
        let store = ModelStore::new(folder.path().join("models"), &source);
        let mut received = Vec::new();

        let first = store.model_file(&MODEL, &mut |bytes| received.push(bytes));
        let second = store.model_file(&MODEL, &mut |_| {});

        let file = first.unwrap();
        assert_eq!(fs::read(&file).unwrap(), WEIGHTS);
        assert_eq!(second, Ok(file));
        assert_eq!(source.downloads.get(), 1);
        assert_eq!(received, [MODEL.file.size]);
    }

    #[test]
    fn failed_downloads_leave_nothing_behind_and_say_why() {
        let failures = [
            (Served::Weights(b"other weights"), ModelError::Corrupted),
            (
                Served::CutShort,
                ModelError::Download("connection reset".to_owned()),
            ),
            (
                Served::Unreachable,
                ModelError::Download("no network".to_owned()),
            ),
        ];
        for (served, expected) in failures {
            let folder = tempfile::tempdir().unwrap();
            let source = serving(served);
            let store = ModelStore::new(folder.path().to_owned(), &source);

            let file = store.model_file(&MODEL, &mut |_| {});

            assert_eq!(file, Err(expected));
            assert!(files_of(folder.path()).is_empty());
        }
    }

    #[test]
    fn download_is_tried_again_after_a_failure() {
        let folder = tempfile::tempdir().unwrap();
        let unreachable = serving(Served::Unreachable);
        let reachable = serving(Served::Weights(WEIGHTS));
        let models = folder.path().to_owned();

        let failed = ModelStore::new(models.clone(), &unreachable).model_file(&MODEL, &mut |_| {});
        let retried = ModelStore::new(models, &reachable).model_file(&MODEL, &mut |_| {});

        assert!(failed.is_err());
        assert!(retried.is_ok());
    }
}
