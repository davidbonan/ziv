use crate::update::domain::update_state::UpdateState;
use crate::update::domain::version::Version;

pub const CHECKING_LABEL: &str = "Checking for updates…";
pub const UP_TO_DATE_LABEL: &str = "ziv is up to date";
pub const RELAUNCHING_LABEL: &str = "Relaunching…";

pub fn available_label(version: Version) -> String {
    format!("Version {version} is available")
}

/// What the window says of the update; nothing before the first check.
pub fn update_status_label(state: &UpdateState) -> Option<String> {
    match state {
        UpdateState::Idle => None,
        UpdateState::Checking => Some(CHECKING_LABEL.to_owned()),
        UpdateState::UpToDate => Some(UP_TO_DATE_LABEL.to_owned()),
        UpdateState::Available(release) => Some(available_label(release.version)),
        UpdateState::Downloading(version) => Some(format!("Downloading {version}…")),
        UpdateState::Installing(version) => Some(format!("Installing {version}…")),
        UpdateState::Installed(_) => Some(RELAUNCHING_LABEL.to_owned()),
        UpdateState::Failed(reason) => Some(reason.clone()),
    }
}
