use std::path::PathBuf;

/// Blocks on the native dialog. `None` when the user cancels.
#[cfg(target_os = "macos")]
pub fn pick_photos_or_folder() -> Option<Vec<PathBuf>> {
    rfd::FileDialog::new().pick_files_or_folders()
}

/// Other platforms' dialogs cannot mix files and folders: files only.
#[cfg(not(target_os = "macos"))]
pub fn pick_photos_or_folder() -> Option<Vec<PathBuf>> {
    rfd::FileDialog::new().pick_files()
}
