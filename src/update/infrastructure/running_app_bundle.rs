use std::path::PathBuf;

use crate::update::domain::app_bundle::app_bundle_of;

/// The app bundle this process runs from; `None` under `cargo run` and tests.
pub fn running_app_bundle() -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?;
    app_bundle_of(&executable.canonicalize().unwrap_or(executable))
}
