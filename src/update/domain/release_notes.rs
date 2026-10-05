use super::version::Version;

/// What changed in each version, newest first, as Markdown.
pub const RELEASE_NOTES: &str = include_str!("../../../release-notes.md");
pub const NOTED_VERSIONS_LIMIT: usize = 10;

const VERSION_HEADING: &str = "## ";

/// The version each section of `notes` is about, in the order of the sections;
/// `None` for a section whose title is not a version.
pub fn noted_versions(notes: &str) -> Vec<Option<Version>> {
    notes
        .lines()
        .filter_map(|line| line.strip_prefix(VERSION_HEADING))
        .map(Version::parse)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_notes_of_this_build_are_about_its_latest_versions_newest_first() {
        let versions: Option<Vec<Version>> = noted_versions(RELEASE_NOTES).into_iter().collect();
        let versions = versions.expect("every section is titled by a version");

        assert!((1..=NOTED_VERSIONS_LIMIT).contains(&versions.len()));
        assert!(versions.is_sorted_by(|newer, older| newer > older));
    }
}
