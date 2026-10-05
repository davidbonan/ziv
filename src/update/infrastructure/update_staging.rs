use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::update::domain::published_release::RELEASE_ASSET_NAME;
use crate::update::domain::update_steps::{InstallError, StagedUpdate};

fn run(tool: &str, arguments: &[&OsStr]) -> Result<(), String> {
    let output = Command::new(tool)
        .args(arguments)
        .output()
        .map_err(|error| format!("{tool} could not be run ({error})"))?;
    match output.status.success() {
        true => Ok(()),
        false => Err(String::from_utf8_lossy(&output.stderr).trim().to_owned()),
    }
}

fn new_work_folder(work_root: &Path) -> PathBuf {
    let now = SystemTime::now().duration_since(UNIX_EPOCH);
    let nanoseconds = now.map_or(0, |since_epoch| since_epoch.as_nanos());
    work_root.join(format!("ziv-update-{}-{nanoseconds}", process::id()))
}

fn unpacked_app_bundle(zip: &Path, into: &Path) -> Result<PathBuf, String> {
    run(
        "ditto",
        &["-x".as_ref(), "-k".as_ref(), zip.as_ref(), into.as_ref()],
    )?;
    let unpacked = fs::read_dir(into).map_err(|error| error.to_string())?;
    unpacked
        .filter_map(|entry| Some(entry.ok()?.path()))
        .find(|path| path.extension().is_some_and(|extension| extension == "app"))
        .ok_or_else(|| "no app bundle in the archive".to_owned())
}

fn staged_in(
    work_folder: &Path,
    download: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<PathBuf, InstallError> {
    fs::create_dir_all(work_folder).map_err(|error| InstallError::Download(error.to_string()))?;
    let zip = work_folder.join(RELEASE_ASSET_NAME);
    download(&zip).map_err(InstallError::Download)?;
    let app_bundle =
        unpacked_app_bundle(&zip, &work_folder.join("unpacked")).map_err(InstallError::Unpack)?;
    let verification = [
        "--verify".as_ref(),
        "--strict".as_ref(),
        app_bundle.as_os_str(),
    ];
    run("codesign", &verification).map_err(InstallError::InvalidSignature)?;
    Ok(app_bundle)
}

/// Downloads a zipped app bundle into a new folder of `work_root`, unpacks it
/// and validates its signature. Nothing is left of a download that fails.
pub fn stage_update(
    work_root: &Path,
    download: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<StagedUpdate, InstallError> {
    let work_folder = new_work_folder(work_root);
    match staged_in(&work_folder, download) {
        Ok(app_bundle) => Ok(StagedUpdate {
            work_folder,
            app_bundle,
        }),
        Err(error) => {
            let _ = fs::remove_dir_all(&work_folder);
            Err(error)
        }
    }
}
