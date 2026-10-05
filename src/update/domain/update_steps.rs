use std::fmt;
use std::path::PathBuf;

use super::published_release::{PublishedRelease, ReleaseError};
use super::update_check::UpdateCheck;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckError {
    Unreachable(String),
    Release(ReleaseError),
}

impl fmt::Display for CheckError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreachable(reason) => {
                write!(formatter, "Updates could not be checked: {reason}")
            }
            Self::Release(reason) => write!(formatter, "Updates could not be checked: {reason}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallError {
    Download(String),
    Unpack(String),
    InvalidSignature(String),
    /// The app bundle sits where it cannot be replaced.
    NotWritable,
    Swap(String),
}

impl fmt::Display for InstallError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Download(reason) => {
                write!(formatter, "The update could not be downloaded: {reason}")
            }
            Self::Unpack(reason) => write!(formatter, "The update could not be unpacked: {reason}"),
            Self::InvalidSignature(reason) => {
                write!(
                    formatter,
                    "The update was refused, its signature is not valid: {reason}"
                )
            }
            Self::NotWritable => write!(
                formatter,
                "ziv cannot be replaced where it is: move it to /Applications and try again"
            ),
            Self::Swap(reason) => write!(formatter, "The update could not be installed: {reason}"),
        }
    }
}

/// A downloaded app bundle whose signature was validated, waiting in its work folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedUpdate {
    pub work_folder: PathBuf,
    pub app_bundle: PathBuf,
}

/// What an update is made of, each step slow enough to stay off the UI thread.
pub trait UpdateSteps: Send + Sync {
    fn check(&self) -> Result<UpdateCheck, CheckError>;

    /// Downloads, unpacks and validates `release`; the running app is untouched.
    fn stage(&self, release: &PublishedRelease) -> Result<StagedUpdate, InstallError>;

    /// Puts the staged app bundle in place of the running one, which it returns.
    fn swap(&self, staged: &StagedUpdate) -> Result<PathBuf, InstallError>;
}
