use std::fmt;

use serde::Deserialize;

use super::version::Version;

/// The file of a release that holds the zipped app bundle.
pub const RELEASE_ASSET_NAME: &str = "ziv-macos.zip";

/// A version published on GitHub, with where its app bundle is downloaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedRelease {
    pub version: Version,
    pub asset_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseError {
    NotARelease(String),
    MalformedTag(String),
    MissingAsset,
}

impl fmt::Display for ReleaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotARelease(reason) => write!(formatter, "unexpected answer ({reason})"),
            Self::MalformedTag(tag) => {
                write!(formatter, "the release tag {tag:?} is not a version")
            }
            Self::MissingAsset => write!(formatter, "the release has no {RELEASE_ASSET_NAME}"),
        }
    }
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

impl PublishedRelease {
    /// Reads what GitHub answers when asked for the latest release.
    pub fn from_github_answer(answer: &str) -> Result<Self, ReleaseError> {
        let release: GithubRelease = serde_json::from_str(answer)
            .map_err(|error| ReleaseError::NotARelease(error.to_string()))?;
        let version = Version::parse(&release.tag_name)
            .ok_or(ReleaseError::MalformedTag(release.tag_name))?;
        let asset = release
            .assets
            .into_iter()
            .find(|asset| asset.name == RELEASE_ASSET_NAME)
            .ok_or(ReleaseError::MissingAsset)?;
        Ok(Self {
            version,
            asset_url: asset.browser_download_url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn github_answer(tag: &str, asset_names: &[&str]) -> String {
        let assets: Vec<String> = asset_names
            .iter()
            .map(|name| {
                format!(
                    r#"{{"name":"{name}","browser_download_url":"https://example.invalid/{name}","size":1}}"#
                )
            })
            .collect();
        format!(
            r#"{{"tag_name":"{tag}","draft":false,"assets":[{}],"body":"notes"}}"#,
            assets.join(",")
        )
    }

    #[test]
    fn an_answer_gives_the_version_and_where_the_app_bundle_is() {
        let answer = github_answer("v0.2.0", &["source.tar.gz", RELEASE_ASSET_NAME]);

        let expected = PublishedRelease {
            version: Version::parse("0.2.0").expect("a version"),
            asset_url: format!("https://example.invalid/{RELEASE_ASSET_NAME}"),
        };
        assert_eq!(PublishedRelease::from_github_answer(&answer), Ok(expected));
    }

    #[test]
    fn a_tag_that_is_not_a_version_is_refused() {
        let answer = github_answer("nightly", &[RELEASE_ASSET_NAME]);

        assert_eq!(
            PublishedRelease::from_github_answer(&answer),
            Err(ReleaseError::MalformedTag("nightly".to_owned()))
        );
    }

    #[test]
    fn a_release_without_the_app_bundle_is_refused() {
        let answer = github_answer("v0.2.0", &["other.zip"]);

        assert_eq!(
            PublishedRelease::from_github_answer(&answer),
            Err(ReleaseError::MissingAsset)
        );
    }

    #[test]
    fn an_answer_that_is_not_a_release_is_refused() {
        for answer in [r#"{"message":"Not Found"}"#, "not json"] {
            assert!(matches!(
                PublishedRelease::from_github_answer(answer),
                Err(ReleaseError::NotARelease(_))
            ));
        }
    }
}
