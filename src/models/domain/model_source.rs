use std::io::Read;

use super::model::RemoteFile;

/// Where the files of models come from the first time they are needed.
pub trait ModelSource {
    /// The bytes of the file, read as they arrive; `Err` with the reason
    /// when they cannot be reached.
    fn download(&self, file: &RemoteFile) -> Result<Box<dyn Read>, String>;
}
