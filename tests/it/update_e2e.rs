use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;

use tempfile::TempDir;
use ziv::update::domain::published_release::RELEASE_ASSET_NAME;
use ziv::update::domain::update_check::UpdateCheck;
use ziv::update::domain::update_steps::{CheckError, InstallError, StagedUpdate, UpdateSteps};
use ziv::update::domain::version::Version;
use ziv::update::infrastructure::app_bundle_swap::swap_app_bundle;
use ziv::update::infrastructure::app_update::AppUpdate;
use ziv::update::infrastructure::github_releases::GithubReleases;

const LATEST_RELEASE_PATH: &str = "/latest";
const ASSET_PATH: &str = "/ziv-macos.zip";

fn answer(mut request: TcpStream, files: &HashMap<&'static str, Vec<u8>>) {
    let mut request_line = String::new();
    let _ = BufReader::new(&request).read_line(&mut request_line);
    let path = request_line.split(' ').nth(1).unwrap_or_default();
    let (status, body) = match files.get(path) {
        Some(body) => ("200 OK", body.as_slice()),
        None => ("404 Not Found", &[][..]),
    };
    let head = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = request.write_all(head.as_bytes());
    let _ = request.write_all(body);
}

/// Serves `files` over HTTP on this machine; returns the address they are under.
fn serving(files: HashMap<&'static str, Vec<u8>>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a free port");
    let address = format!("http://{}", listener.local_addr().expect("an address"));
    thread::spawn(move || {
        for request in listener.incoming().flatten() {
            answer(request, &files);
        }
    });
    address
}

fn version(text: &str) -> Version {
    Version::parse(text).expect("a version")
}

fn run(tool: &str, arguments: &[&Path]) {
    let output = Command::new(tool).args(arguments).output().expect(tool);
    let said = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{tool} failed: {said}");
}

/// An app bundle macOS can sign, telling which one it is by its marker.
fn app_bundle(folder: &Path, marker: &str) -> PathBuf {
    let bundle = folder.join("ziv.app");
    fs::create_dir_all(bundle.join("Contents/MacOS")).unwrap();
    fs::create_dir_all(bundle.join("Contents/Resources")).unwrap();
    fs::copy("/bin/ls", bundle.join("Contents/MacOS/ziv")).unwrap();
    let info = r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>ziv</string>
<key>CFBundleIdentifier</key><string>io.github.davidbonan.ziv.test</string>
<key>CFBundlePackageType</key><string>APPL</string>
</dict></plist>"#;
    fs::write(bundle.join("Contents/Info.plist"), info).unwrap();
    fs::write(bundle.join("Contents/Resources/marker.txt"), marker).unwrap();
    bundle
}

fn marker_of(bundle: &Path) -> String {
    fs::read_to_string(bundle.join("Contents/Resources/marker.txt")).unwrap()
}

fn signed(bundle: PathBuf) -> PathBuf {
    let sign = ["--force", "--sign", "-"].map(Path::new);
    run("codesign", &[&sign[..], &[&bundle]].concat());
    bundle
}

fn zipped(bundle: &Path) -> Vec<u8> {
    let zip = bundle.with_extension("zip");
    let keep_parent = ["-c", "-k", "--keepParent"].map(Path::new);
    run("ditto", &[&keep_parent[..], &[bundle, &zip]].concat());
    fs::read(zip).unwrap()
}

/// An installed app at version 0.1.0 and the server its updates come from.
struct Installed {
    folder: TempDir,
    update: AppUpdate,
}

impl Installed {
    /// `new_bundle` is what version `latest` of the served release unzips to.
    fn offered(latest: &str, new_bundle: impl FnOnce(&Path) -> PathBuf) -> Self {
        let folder = tempfile::tempdir().unwrap();
        let zip = zipped(&new_bundle(&folder.path().join("published")));
        let address = serving(HashMap::from([(ASSET_PATH, zip)]));
        let release = format!(
            r#"{{"tag_name":"v{latest}","assets":[{{"name":"{RELEASE_ASSET_NAME}","browser_download_url":"{address}{ASSET_PATH}"}}]}}"#
        );
        let releases = serving(HashMap::from([(LATEST_RELEASE_PATH, release.into_bytes())]));
        let work_root = folder.path().join("work");
        fs::create_dir_all(&work_root).unwrap();
        let update = AppUpdate {
            releases: GithubReleases::answering_at(format!("{releases}{LATEST_RELEASE_PATH}")),
            running: version("0.1.0"),
            app_bundle: app_bundle(&folder.path().join("installed"), "0.1.0"),
            work_root,
        };
        Self { folder, update }
    }

    fn offered_a_signed(latest: &str) -> Self {
        Self::offered(latest, |folder| signed(app_bundle(folder, "new")))
    }

    fn installed_marker(&self) -> String {
        marker_of(&self.update.app_bundle)
    }

    fn work_left(&self) -> usize {
        fs::read_dir(&self.update.work_root).unwrap().count()
    }

    fn staged_from_check(&self) -> Result<StagedUpdate, InstallError> {
        let Ok(UpdateCheck::Available(release)) = self.update.check() else {
            panic!("an update is offered");
        };
        self.update.stage(&release)
    }
}

#[test]
fn a_newer_served_release_is_an_available_update() {
    let installed = Installed::offered_a_signed("0.2.0");

    let Ok(UpdateCheck::Available(release)) = installed.update.check() else {
        panic!("an update is offered");
    };

    assert_eq!(release.version, version("0.2.0"));
}

#[test]
fn a_served_release_that_is_not_newer_leaves_the_app_up_to_date() {
    let installed = Installed::offered_a_signed("0.1.0");

    assert_eq!(installed.update.check(), Ok(UpdateCheck::UpToDate));
}

#[test]
fn a_server_that_cannot_be_reached_fails_the_check() {
    let closed = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap();
    let releases = GithubReleases::answering_at(format!("http://{closed}/latest"));

    assert!(matches!(
        releases.latest_release(),
        Err(CheckError::Unreachable(_))
    ));
}

#[test]
fn an_install_puts_the_downloaded_app_bundle_in_place_of_the_installed_one() {
    let installed = Installed::offered_a_signed("0.2.0");

    let staged = installed.staged_from_check().expect("the update is staged");
    let relaunched = installed.update.swap(&staged);

    assert_eq!(relaunched.as_ref(), Ok(&installed.update.app_bundle));
    assert_eq!(installed.installed_marker(), "new");
    assert_eq!(marker_of(&staged.work_folder.join("replaced.app")), "0.1.0");
}

#[test]
fn an_app_bundle_changed_after_its_signature_is_refused_and_leaves_nothing() {
    let installed = Installed::offered("0.2.0", |folder| {
        let bundle = signed(app_bundle(folder, "new"));
        fs::copy("/bin/cat", bundle.join("Contents/MacOS/ziv")).unwrap();
        bundle
    });

    let staged = installed.staged_from_check();

    assert!(
        matches!(staged, Err(InstallError::InvalidSignature(_))),
        "{staged:?}"
    );
    assert_eq!(installed.installed_marker(), "0.1.0");
    assert_eq!(installed.work_left(), 0);
}

#[test]
fn a_swap_that_cannot_end_puts_the_installed_app_bundle_back() {
    let installed = Installed::offered_a_signed("0.2.0");
    let nothing_staged = StagedUpdate {
        work_folder: installed.update.work_root.clone(),
        app_bundle: installed.folder.path().join("nowhere.app"),
    };

    let swapped = swap_app_bundle(&installed.update.app_bundle, &nothing_staged);

    assert!(matches!(swapped, Err(InstallError::Swap(_))), "{swapped:?}");
    assert_eq!(installed.installed_marker(), "0.1.0");
}

#[test]
fn an_app_bundle_in_a_folder_that_cannot_be_written_asks_to_be_moved() {
    let installed = Installed::offered_a_signed("0.2.0");
    let staged = installed.staged_from_check().expect("the update is staged");
    let applications = installed.folder.path().join("installed");
    fs::set_permissions(&applications, fs::Permissions::from_mode(0o555)).unwrap();

    let swapped = installed.update.swap(&staged);

    fs::set_permissions(&applications, fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(swapped, Err(InstallError::NotWritable));
    assert_eq!(installed.installed_marker(), "0.1.0");
}
