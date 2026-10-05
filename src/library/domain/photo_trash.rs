use std::path::Path;

/// Where a file the user no longer wants goes, to be put back or emptied by them.
pub trait PhotoTrash {
    /// `Err` with the reason when the file stays where it is.
    fn move_to_trash(&self, file: &Path) -> Result<(), String>;
}
