use std::path::{Path, PathBuf};

/// The app bundle `executable` runs from: `<name>.app/Contents/MacOS/<executable>`.
/// `None` for an executable outside one, as under `cargo run`.
pub fn app_bundle_of(executable: &Path) -> Option<PathBuf> {
    let executables = executable.parent()?;
    let contents = executables.parent()?;
    let bundle = contents.parent()?;
    let is_bundle = executables.file_name()? == "MacOS"
        && contents.file_name()? == "Contents"
        && bundle.extension()? == "app";
    is_bundle.then(|| bundle.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_executable_inside_an_app_bundle_names_it() {
        assert_eq!(
            app_bundle_of(Path::new("/Applications/ziv.app/Contents/MacOS/ziv")),
            Some(PathBuf::from("/Applications/ziv.app"))
        );
    }

    #[test]
    fn an_executable_outside_an_app_bundle_has_none() {
        for executable in [
            "/Users/me/dev/ziv/target/debug/ziv",
            "/Applications/ziv.app/Contents/Helpers/ziv",
            "/Applications/ziv/Contents/MacOS/ziv",
            "ziv",
        ] {
            assert_eq!(app_bundle_of(Path::new(executable)), None, "{executable}");
        }
    }
}
