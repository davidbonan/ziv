use std::path::PathBuf;

use super::app_bundle_swap::swap_app_bundle;
use super::github_releases::GithubReleases;
use super::update_staging::stage_update;
use crate::update::domain::published_release::PublishedRelease;
use crate::update::domain::update_check::UpdateCheck;
use crate::update::domain::update_steps::{CheckError, InstallError, StagedUpdate, UpdateSteps};
use crate::update::domain::version::Version;

/// The update of an app bundle of this Mac from its releases on GitHub.
pub struct AppUpdate {
    pub releases: GithubReleases,
    pub running: Version,
    pub app_bundle: PathBuf,
    /// Where downloads are unpacked and replaced app bundles are left.
    pub work_root: PathBuf,
}

impl AppUpdate {
    /// The update of the running ziv, installed in `app_bundle`.
    pub fn of_ziv_in(app_bundle: PathBuf) -> Self {
        Self {
            releases: GithubReleases::of_ziv(),
            running: Version::of_this_build(),
            app_bundle,
            work_root: std::env::temp_dir(),
        }
    }
}

impl UpdateSteps for AppUpdate {
    fn check(&self) -> Result<UpdateCheck, CheckError> {
        let latest = self.releases.latest_release()?;
        Ok(UpdateCheck::of(latest, self.running))
    }

    fn stage(&self, release: &PublishedRelease) -> Result<StagedUpdate, InstallError> {
        stage_update(&self.work_root, |zip| {
            self.releases.download(&release.asset_url, zip)
        })
    }

    fn swap(&self, staged: &StagedUpdate) -> Result<PathBuf, InstallError> {
        swap_app_bundle(&self.app_bundle, staged)?;
        Ok(self.app_bundle.clone())
    }
}
