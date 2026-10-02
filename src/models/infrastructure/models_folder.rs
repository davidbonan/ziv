use std::path::PathBuf;

/// Where models are kept on this machine; `None` when the user has no data folder.
pub fn models_folder() -> Option<PathBuf> {
    dirs::data_dir().map(|data| data.join("ziv").join("models"))
}
