use std::path::{Path, PathBuf};
use std::process::Command;

/// Blocks on the native dialog. `None` when the user cancels.
pub fn pick_series_folder() -> Option<PathBuf> {
    rfd::FileDialog::new().pick_folder()
}

/// `Err` with the reason when the Finder could not be asked.
pub fn show_in_finder(folder: &Path) -> Result<(), String> {
    let asked = Command::new("open").arg(folder).spawn();
    asked.map(|_| ()).map_err(|error| error.to_string())
}

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
