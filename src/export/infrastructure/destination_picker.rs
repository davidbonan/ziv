use std::path::PathBuf;

/// Blocks on the native dialog. `None` when the user cancels.
pub fn pick_destination() -> Option<PathBuf> {
    rfd::FileDialog::new().pick_folder()
}
