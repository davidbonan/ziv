use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

/// Starts a new instance of `app_bundle`, which outlives this process.
pub fn relaunch(app_bundle: &Path) -> io::Result<()> {
    Command::new("open")
        .arg("-n")
        .arg(app_bundle)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
}
