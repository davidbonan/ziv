use std::path::Path;

use super::edit::Edit;

/// Where the edits of photos are kept between two runs of the app.
pub trait EditStorage {
    /// `Ok(None)` when nothing is stored for the photo; `Err` with the reason
    /// when what is stored cannot be used.
    fn stored_edit(&self, photo: &Path) -> Result<Option<Edit>, String>;

    /// A default edit leaves nothing stored.
    fn store_edit(&self, photo: &Path, edit: &Edit) -> Result<(), String>;
}
