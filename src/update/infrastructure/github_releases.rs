use std::fs::File;
use std::io;
use std::path::Path;
use std::time::Duration;

use crate::update::domain::published_release::PublishedRelease;
use crate::update::domain::update_steps::CheckError;

const LATEST_ZIV_RELEASE_URL: &str = "https://api.github.com/repos/davidbonan/ziv/releases/latest";
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(20);

/// The releases of an app on GitHub, asked over HTTPS without a token.
pub struct GithubReleases {
    latest_release_url: String,
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(CONNECTION_TIMEOUT))
        .timeout_recv_response(Some(CONNECTION_TIMEOUT))
        .build()
        .into()
}

impl GithubReleases {
    pub fn of_ziv() -> Self {
        Self::answering_at(LATEST_ZIV_RELEASE_URL)
    }

    /// The releases whose latest one is answered at `latest_release_url`.
    pub fn answering_at(latest_release_url: impl Into<String>) -> Self {
        Self {
            latest_release_url: latest_release_url.into(),
        }
    }

    pub fn latest_release(&self) -> Result<PublishedRelease, CheckError> {
        let answer = agent()
            .get(&self.latest_release_url)
            .call()
            .and_then(|response| response.into_body().read_to_string())
            .map_err(|error| CheckError::Unreachable(error.to_string()))?;
        PublishedRelease::from_github_answer(&answer).map_err(CheckError::Release)
    }

    /// Writes what `url` serves into `file`.
    pub fn download(&self, url: &str, file: &Path) -> Result<(), String> {
        let response = agent().get(url).call().map_err(|error| error.to_string())?;
        let mut download = response.into_body().into_reader();
        let mut file = File::create(file).map_err(|error| error.to_string())?;
        io::copy(&mut download, &mut file).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())
    }
}
