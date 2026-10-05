use std::fs;
use std::io;
use std::path::Path;

use crate::update::domain::update_steps::{InstallError, StagedUpdate};

const REPLACED_APP_BUNDLE_NAME: &str = "replaced.app";

fn swap_error(error: io::Error) -> InstallError {
    match error.kind() {
        io::ErrorKind::PermissionDenied => InstallError::NotWritable,
        _ => InstallError::Swap(error.to_string()),
    }
}

/// Puts the staged app bundle where `running` is; `running` goes to the work
/// folder of the update, and comes back when the staged one cannot take its place.
pub fn swap_app_bundle(running: &Path, staged: &StagedUpdate) -> Result<(), InstallError> {
    let replaced = staged.work_folder.join(REPLACED_APP_BUNDLE_NAME);
    fs::rename(running, &replaced).map_err(swap_error)?;
    fs::rename(&staged.app_bundle, running).map_err(|error| {
        let _ = fs::rename(&replaced, running);
        swap_error(error)
    })
}
