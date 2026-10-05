use super::published_release::PublishedRelease;
use super::version::Version;

/// What the latest release means for the running version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateCheck {
    UpToDate,
    Available(PublishedRelease),
}

impl UpdateCheck {
    /// A build ahead of the latest release is up to date.
    pub fn of(latest: PublishedRelease, running: Version) -> Self {
        match latest.version > running {
            true => Self::Available(latest),
            false => Self::UpToDate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(text: &str) -> Version {
        Version::parse(text).expect("a version")
    }

    fn release(text: &str) -> PublishedRelease {
        PublishedRelease {
            version: version(text),
            asset_url: "https://example.invalid/ziv-macos.zip".to_owned(),
        }
    }

    #[test]
    fn only_a_newer_release_is_an_available_update() {
        let latest = release("0.2.0");

        assert_eq!(
            UpdateCheck::of(latest.clone(), version("0.1.9")),
            UpdateCheck::Available(latest.clone())
        );
        assert_eq!(
            UpdateCheck::of(latest.clone(), version("0.2.0")),
            UpdateCheck::UpToDate
        );
        assert_eq!(
            UpdateCheck::of(latest, version("0.3.0")),
            UpdateCheck::UpToDate
        );
    }
}
