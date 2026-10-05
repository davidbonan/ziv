use std::path::PathBuf;

use super::published_release::PublishedRelease;
use super::version::Version;

/// Where the update stands, as the window shows it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum UpdateState {
    #[default]
    Idle,
    Checking,
    UpToDate,
    Available(PublishedRelease),
    Downloading(Version),
    Installing(Version),
    /// The new version is in place in this app bundle: time to relaunch.
    Installed(PathBuf),
    Failed(String),
}

impl UpdateState {
    /// A check or an install is running, or over with a relaunch to come.
    pub fn is_busy(&self) -> bool {
        matches!(
            self,
            Self::Checking | Self::Downloading(_) | Self::Installing(_) | Self::Installed(_)
        )
    }

    pub fn is_installing(&self) -> bool {
        matches!(self, Self::Downloading(_) | Self::Installing(_))
    }
}
