use std::fmt;

use serde::{Deserialize, Serialize};

/// The version of a build or of a release: `major.minor.patch`, nothing after.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    /// Reads `0.2.0` or `v0.2.0`; `None` for anything else.
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        let mut numbers = text.strip_prefix('v').unwrap_or(text).split('.');
        let mut next = || numbers.next()?.parse().ok();
        let version = Self {
            major: next()?,
            minor: next()?,
            patch: next()?,
        };
        numbers.next().is_none().then_some(version)
    }

    pub fn of_this_build() -> Self {
        Self::parse(env!("CARGO_PKG_VERSION")).expect("the version of Cargo.toml is x.y.z")
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            major,
            minor,
            patch,
        } = self;
        write!(formatter, "{major}.{minor}.{patch}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(text: &str) -> Version {
        Version::parse(text).expect("a version")
    }

    #[test]
    fn a_version_reads_with_or_without_its_prefix() {
        let expected = Version {
            major: 1,
            minor: 2,
            patch: 3,
        };

        assert_eq!(Version::parse("1.2.3"), Some(expected));
        assert_eq!(Version::parse("v1.2.3"), Some(expected));
        assert_eq!(Version::parse(" v1.2.3\n"), Some(expected));
    }

    #[test]
    fn anything_but_three_numbers_is_refused() {
        for text in ["", "1", "1.2", "1.2.3.4", "1.2.x", "1.2.3-beta", "nightly"] {
            assert_eq!(Version::parse(text), None, "{text:?} was read");
        }
    }

    #[test]
    fn versions_order_by_major_then_minor_then_patch() {
        assert!(version("2.0.0") > version("1.9.9"));
        assert!(version("1.10.0") > version("1.9.9"));
        assert!(version("1.0.10") > version("1.0.9"));
        assert_eq!(version("1.0.10").to_string(), "1.0.10");
    }
}
