use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::library::domain::catalog::Catalog;
use crate::library::domain::catalog_document::{catalog_document, catalog_of_document};
use crate::library::domain::catalog_storage::CatalogStorage;

const FILE_NAME: &str = "catalog.json";
const BEING_WRITTEN_NAME: &str = "catalog.json.tmp";

const NO_DATA_FOLDER: &str = "this Mac has no data folder";

/// One JSON file in ziv's data folder.
pub struct CatalogFile {
    /// `None` when the user has no data folder: nothing can be kept.
    folder: Option<PathBuf>,
}

impl CatalogFile {
    pub fn of_this_mac() -> Self {
        Self {
            folder: dirs::data_dir().map(|data| data.join("ziv")),
        }
    }

    pub fn in_folder(folder: PathBuf) -> Self {
        Self {
            folder: Some(folder),
        }
    }

    pub fn path(&self) -> Option<PathBuf> {
        self.folder.as_ref().map(|folder| folder.join(FILE_NAME))
    }
}

// Written beside, then renamed: a crash never leaves half a catalog.
fn replace_atomically(folder: &Path, text: &str) -> std::io::Result<()> {
    fs::create_dir_all(folder)?;
    let being_written = folder.join(BEING_WRITTEN_NAME);
    fs::write(&being_written, text)?;
    fs::rename(being_written, folder.join(FILE_NAME))
}

impl CatalogStorage for CatalogFile {
    fn stored_catalog(&self) -> Result<Option<Catalog>, String> {
        let path = self.path().ok_or(NO_DATA_FOLDER)?;
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("it cannot be read ({error})")),
        };
        catalog_of_document(&text)
            .map(Some)
            .map_err(|error| error.to_string())
    }

    fn store_catalog(&self, catalog: &Catalog) -> Result<(), String> {
        let folder = self.folder.as_ref().ok_or(NO_DATA_FOLDER)?;
        let text = catalog_document(catalog)?;
        replace_atomically(folder, &text).map_err(|error| error.to_string())
    }
}
