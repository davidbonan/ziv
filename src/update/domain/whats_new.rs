use super::version::Version;

/// What a bundled app does at launch about its release notes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatsNew {
    Nothing,
    /// First install: the running version becomes the seen one, nothing is shown.
    TakeAsSeen,
    Show,
}

impl WhatsNew {
    pub fn at_launch(running: Version, seen: Option<Version>) -> Self {
        match seen {
            None => Self::TakeAsSeen,
            Some(seen) if running > seen => Self::Show,
            Some(_) => Self::Nothing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(text: &str) -> Version {
        Version::parse(text).expect("a version")
    }

    #[test]
    fn release_notes_are_shown_once_per_newer_version() {
        let running = version("0.2.0");

        assert_eq!(
            WhatsNew::at_launch(running, Some(version("0.1.0"))),
            WhatsNew::Show
        );
        assert_eq!(
            WhatsNew::at_launch(running, Some(version("0.2.0"))),
            WhatsNew::Nothing
        );
        assert_eq!(
            WhatsNew::at_launch(running, Some(version("0.3.0"))),
            WhatsNew::Nothing
        );
    }

    #[test]
    fn a_first_install_shows_nothing_and_takes_its_version_as_seen() {
        assert_eq!(
            WhatsNew::at_launch(version("0.2.0"), None),
            WhatsNew::TakeAsSeen
        );
    }
}
