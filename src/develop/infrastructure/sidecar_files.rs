use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::develop::domain::edit::Edit;
use crate::develop::domain::edit_document::{edit_document, edit_of_document};
use crate::develop::domain::edit_storage::EditStorage;
use crate::photo::domain::photo_name::file_beside;

const SIDECAR_SUFFIX: &str = ".ziv.json";
const BEING_WRITTEN_SUFFIX: &str = ".ziv.json.tmp";

pub fn sidecar_path(photo: &Path) -> PathBuf {
    file_beside(photo, SIDECAR_SUFFIX)
}

/// Whether `photo` is edited: an unedited photo has no sidecar.
pub fn has_sidecar(photo: &Path) -> bool {
    sidecar_path(photo).exists()
}

fn remove_if_present(path: &Path) -> std::io::Result<()> {
    match fs::remove_file(path) {
        Err(error) if error.kind() != ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

// Written beside, then renamed: a crash never leaves half a sidecar.
fn replace_atomically(sidecar: &Path, being_written: &Path, text: &str) -> std::io::Result<()> {
    fs::write(being_written, text)?;
    fs::rename(being_written, sidecar)
}

/// One JSON file next to each edited photo.
pub struct SidecarFiles;

impl EditStorage for SidecarFiles {
    fn stored_edit(&self, photo: &Path) -> Result<Option<Edit>, String> {
        let text = match fs::read_to_string(sidecar_path(photo)) {
            Ok(text) => text,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("it cannot be read ({error})")),
        };
        edit_of_document(&text)
            .map(Some)
            .map_err(|error| error.to_string())
    }

    fn store_edit(&self, photo: &Path, edit: &Edit) -> Result<(), String> {
        let sidecar = sidecar_path(photo);
        let written = if *edit == Edit::default() {
            remove_if_present(&sidecar)
        } else {
            let being_written = file_beside(photo, BEING_WRITTEN_SUFFIX);
            replace_atomically(&sidecar, &being_written, &edit_document(edit))
        };
        written.map_err(|error| error.to_string())
    }
}
