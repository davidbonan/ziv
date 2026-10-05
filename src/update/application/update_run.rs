use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

use crate::update::domain::update_check::UpdateCheck;
use crate::update::domain::update_state::UpdateState;
use crate::update::domain::update_steps::{CheckError, UpdateSteps};

/// The update of this app: one check or one install at a time, each on its
/// own thread.
#[derive(Clone)]
pub struct UpdateRun {
    state: Arc<Mutex<UpdateState>>,
    steps: Arc<dyn UpdateSteps>,
    notify: Arc<dyn Fn() + Send + Sync>,
}

fn locked(state: &Mutex<UpdateState>) -> MutexGuard<'_, UpdateState> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl UpdateRun {
    /// `notify` runs each time the state changed off the caller's thread.
    pub fn new(
        steps: impl UpdateSteps + 'static,
        notify: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        Self {
            state: Arc::default(),
            steps: Arc::new(steps),
            notify: Arc::new(notify),
        }
    }

    pub fn state(&self) -> UpdateState {
        locked(&self.state).clone()
    }

    /// Checks and says why when it could not. Does nothing while busy.
    pub fn check(&self) {
        self.check_failing_as(|error| UpdateState::Failed(error.to_string()));
    }

    /// Checks and says nothing when it could not. Does nothing while busy.
    pub fn check_quietly(&self) {
        self.check_failing_as(|_| UpdateState::Idle);
    }

    fn check_failing_as(&self, state_of_failure: fn(CheckError) -> UpdateState) {
        {
            let mut state = locked(&self.state);
            if state.is_busy() {
                return;
            }
            *state = UpdateState::Checking;
        }
        let Self {
            state,
            steps,
            notify,
        } = self.clone();
        thread::spawn(move || {
            let checked = match steps.check() {
                Ok(UpdateCheck::UpToDate) => UpdateState::UpToDate,
                Ok(UpdateCheck::Available(release)) => UpdateState::Available(release),
                Err(error) => state_of_failure(error),
            };
            *locked(&state) = checked;
            notify();
        });
    }

    /// Installs the available update; does nothing without one.
    pub fn install(&self) {
        let release = {
            let mut state = locked(&self.state);
            let UpdateState::Available(release) = &*state else {
                return;
            };
            let release = release.clone();
            *state = UpdateState::Downloading(release.version);
            release
        };
        let Self {
            state,
            steps,
            notify,
        } = self.clone();
        thread::spawn(move || {
            let installed = steps.stage(&release).and_then(|staged| {
                *locked(&state) = UpdateState::Installing(release.version);
                notify();
                steps.swap(&staged)
            });
            *locked(&state) = match installed {
                Ok(app_bundle) => UpdateState::Installed(app_bundle),
                Err(error) => UpdateState::Failed(error.to_string()),
            };
            notify();
        });
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc::{Receiver, Sender, channel};
    use std::time::Duration;

    use super::*;
    use crate::update::domain::published_release::PublishedRelease;
    use crate::update::domain::update_steps::{InstallError, StagedUpdate};
    use crate::update::domain::version::Version;

    const PATIENCE: Duration = Duration::from_secs(5);

    fn newer_release() -> PublishedRelease {
        PublishedRelease {
            version: Version::parse("0.2.0").expect("a version"),
            asset_url: "https://example.invalid/ziv-macos.zip".to_owned(),
        }
    }

    fn installed_bundle() -> PathBuf {
        PathBuf::from("/Applications/ziv.app")
    }

    /// Steps that answer what they were given, the check only once `resumed` says so.
    struct ScriptedSteps {
        checked: Result<UpdateCheck, CheckError>,
        staged: Result<(), InstallError>,
        check_count: Arc<AtomicUsize>,
        swap_count: Arc<AtomicUsize>,
        resumed: Option<Mutex<Receiver<()>>>,
    }

    impl ScriptedSteps {
        fn finding(checked: Result<UpdateCheck, CheckError>) -> Self {
            Self {
                checked,
                staged: Ok(()),
                check_count: Arc::default(),
                swap_count: Arc::default(),
                resumed: None,
            }
        }

        fn finding_an_update() -> Self {
            Self::finding(Ok(UpdateCheck::Available(newer_release())))
        }
    }

    impl UpdateSteps for ScriptedSteps {
        fn check(&self) -> Result<UpdateCheck, CheckError> {
            self.check_count.fetch_add(1, Ordering::Relaxed);
            if let Some(resumed) = &self.resumed {
                let _ = locked_receiver(resumed).recv_timeout(PATIENCE);
            }
            self.checked.clone()
        }

        fn stage(&self, _: &PublishedRelease) -> Result<StagedUpdate, InstallError> {
            self.staged.clone().map(|()| StagedUpdate {
                work_folder: PathBuf::from("/tmp/work"),
                app_bundle: PathBuf::from("/tmp/work/ziv.app"),
            })
        }

        fn swap(&self, _: &StagedUpdate) -> Result<PathBuf, InstallError> {
            self.swap_count.fetch_add(1, Ordering::Relaxed);
            Ok(installed_bundle())
        }
    }

    fn locked_receiver(receiver: &Mutex<Receiver<()>>) -> MutexGuard<'_, Receiver<()>> {
        receiver
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn run_of(steps: ScriptedSteps) -> (UpdateRun, Receiver<UpdateState>) {
        let (notify, notified) = channel();
        let run = UpdateRun::new(steps, || {});
        let run = notifying_states(run, notify);
        (run, notified)
    }

    /// The same run, sending its state at each change.
    fn notifying_states(run: UpdateRun, notify: Sender<UpdateState>) -> UpdateRun {
        let state = run.state.clone();
        let notify = Mutex::new(notify);
        UpdateRun {
            notify: Arc::new(move || {
                let notify = notify
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                let _ = notify.send(locked(&state).clone());
            }),
            ..run
        }
    }

    fn next(notified: &Receiver<UpdateState>) -> UpdateState {
        notified.recv_timeout(PATIENCE).expect("the state changes")
    }

    #[test]
    fn a_check_ends_on_what_the_latest_release_means() {
        let (run, notified) = run_of(ScriptedSteps::finding_an_update());
        run.check();
        assert_eq!(next(&notified), UpdateState::Available(newer_release()));

        let (run, notified) = run_of(ScriptedSteps::finding(Ok(UpdateCheck::UpToDate)));
        run.check_quietly();
        assert_eq!(next(&notified), UpdateState::UpToDate);
    }

    #[test]
    fn a_failed_check_says_why_unless_it_was_quiet() {
        let unreachable = || Err(CheckError::Unreachable("offline".to_owned()));

        let (run, notified) = run_of(ScriptedSteps::finding(unreachable()));
        run.check();
        assert_eq!(
            next(&notified),
            UpdateState::Failed("Updates could not be checked: offline".to_owned())
        );

        let (run, notified) = run_of(ScriptedSteps::finding(unreachable()));
        run.check_quietly();
        assert_eq!(next(&notified), UpdateState::Idle);
    }

    #[test]
    fn an_install_downloads_then_installs_then_waits_for_the_relaunch() {
        let (run, notified) = run_of(ScriptedSteps::finding_an_update());
        run.check();
        let _ = next(&notified);

        run.install();

        let version = newer_release().version;
        assert_eq!(next(&notified), UpdateState::Installing(version));
        assert_eq!(next(&notified), UpdateState::Installed(installed_bundle()));
        assert!(run.state().is_busy());
    }

    #[test]
    fn a_download_that_fails_swaps_nothing_and_says_why() {
        let steps = ScriptedSteps {
            staged: Err(InstallError::Download("offline".to_owned())),
            ..ScriptedSteps::finding_an_update()
        };
        let swap_count = steps.swap_count.clone();
        let (run, notified) = run_of(steps);
        run.check();
        let _ = next(&notified);

        run.install();

        assert_eq!(
            next(&notified),
            UpdateState::Failed("The update could not be downloaded: offline".to_owned())
        );
        assert_eq!(swap_count.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn asking_while_an_operation_runs_does_nothing() {
        let (resume, resumed) = channel();
        let steps = ScriptedSteps {
            resumed: Some(Mutex::new(resumed)),
            ..ScriptedSteps::finding_an_update()
        };
        let check_count = steps.check_count.clone();
        let (run, notified) = run_of(steps);

        run.check();
        run.check();
        run.install();
        assert_eq!(run.state(), UpdateState::Checking);
        resume.send(()).expect("the check is waiting");

        assert_eq!(next(&notified), UpdateState::Available(newer_release()));
        assert_eq!(check_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn an_install_without_an_available_update_does_nothing() {
        let (run, _) = run_of(ScriptedSteps::finding_an_update());

        run.install();

        assert_eq!(run.state(), UpdateState::Idle);
    }
}
