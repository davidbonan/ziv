/// A file to fetch: where it comes from and what it must be.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteFile {
    pub url: &'static str,
    pub sha256: &'static str,
    pub size: u64,
}

/// What an inference needs, fetched once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Model {
    /// What the user reads: "the subject model".
    pub name: &'static str,
    pub file: RemoteFile,
}

const SHA_SHARE_IN_FILE_NAME: usize = 12;

impl Model {
    /// Named after its content: another version of a model is another file.
    pub fn file_name(&self) -> String {
        let sha256 = self.file.sha256;
        let sha_share = &sha256[..SHA_SHARE_IN_FILE_NAME.min(sha256.len())];
        format!("{}-{sha_share}.onnx", self.name)
    }
}
