use std::fs;
use std::path::{Path, PathBuf};

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.to_string_lossy().starts_with('.'))
}

fn direct_children(folder: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect()
}

/// The photo files among `opened`: the files themselves and the direct children
/// of the folders, hidden files and those `is_photo` rejects left out.
pub fn photo_files_among(opened: &[PathBuf], is_photo: impl Fn(&Path) -> bool) -> Vec<PathBuf> {
    opened
        .iter()
        .flat_map(|path| {
            if path.is_dir() {
                direct_children(path)
            } else {
                vec![path.clone()]
            }
        })
        .filter(|path| !is_hidden(path) && is_photo(path))
        .collect()
}
