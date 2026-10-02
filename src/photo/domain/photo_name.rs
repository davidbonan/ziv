use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The name a photo is shown under: its file name.
pub fn photo_name(path: &Path) -> String {
    path.file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned()
}

/// The file next to `photo` named after its whole file name plus `suffix`.
pub fn file_beside(photo: &Path, suffix: &str) -> PathBuf {
    let mut name = OsString::from(photo.file_name().unwrap_or_default());
    name.push(suffix);
    photo.with_file_name(name)
}
