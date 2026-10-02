use std::ops::ControlFlow;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::sync::{Arc, Mutex};
use std::thread;

use super::enhancer::{EnhancementError, EnhancementStep, OnStep};

const STOPPED_UNEXPECTEDLY: &str = "the enhancement stopped unexpectedly";

/// One enhancement running on its own thread.
pub struct EnhancementRun {
    step: Arc<Mutex<Option<EnhancementStep>>>,
    is_cancel_asked: Arc<AtomicBool>,
    result: Receiver<Result<(), EnhancementError>>,
}

impl EnhancementRun {
    /// `enhance` is told to report its steps, and is answered `Break` once
    /// Cancel was asked; `notify` runs at each step and when it is over.
    pub fn start(
        enhance: impl FnOnce(OnStep<'_>) -> Result<(), EnhancementError> + Send + 'static,
        notify: impl Fn() + Send + 'static,
    ) -> Self {
        let step = Arc::new(Mutex::new(None));
        let is_cancel_asked = Arc::new(AtomicBool::new(false));
        let (send_result, result) = channel();
        let run = Self {
            step: step.clone(),
            is_cancel_asked: is_cancel_asked.clone(),
            result,
        };
        thread::spawn(move || {
            let mut on_step = |reported| {
                *step.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(reported);
                notify();
                match is_cancel_asked.load(Ordering::Relaxed) {
                    true => ControlFlow::Break(()),
                    false => ControlFlow::Continue(()),
                }
            };
            let enhanced = enhance(&mut on_step);
            let _ = send_result.send(enhanced);
            notify();
        });
        run
    }

    /// What the enhancement is busy with; `None` before its first step.
    pub fn step(&self) -> Option<EnhancementStep> {
        *self
            .step
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Stops at the next step.
    pub fn cancel(&self) {
        self.is_cancel_asked.store(true, Ordering::Relaxed);
    }

    /// How it ended, once; `None` as long as it runs.
    pub fn take_result(&self) -> Option<Result<(), EnhancementError>> {
        match self.result.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Err(EnhancementError::Inference(
                STOPPED_UNEXPECTEDLY.to_owned(),
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::Sender;
    use std::time::Duration;

    use super::*;

    const PATIENCE: Duration = Duration::from_secs(5);
    const HALFWAY: EnhancementStep = EnhancementStep::Enhancing { share: 0.5 };

    fn notifying(notified: Sender<()>) -> impl Fn() + Send + 'static {
        move || {
            let _ = notified.send(());
        }
    }

    fn result_of(run: &EnhancementRun, notified: &Receiver<()>) -> Result<(), EnhancementError> {
        loop {
            if let Some(result) = run.take_result() {
                return result;
            }
            let _ = notified.recv_timeout(Duration::from_millis(10));
        }
    }

    /// Reports one step, waits to be resumed, then ends as its second step is answered.
    fn two_step_run(notify: Sender<()>, resumed: Receiver<()>) -> EnhancementRun {
        let enhance = move |on_step: OnStep<'_>| {
            let _ = on_step(HALFWAY);
            let _ = resumed.recv_timeout(PATIENCE);
            match on_step(EnhancementStep::Enhancing { share: 1.0 }) {
                ControlFlow::Continue(()) => Ok(()),
                ControlFlow::Break(()) => Err(EnhancementError::Cancelled),
            }
        };
        EnhancementRun::start(enhance, notifying(notify))
    }

    #[test]
    fn run_reports_its_step_then_that_it_ended() {
        let (notify, notified) = channel();
        let (resume, resumed) = channel();
        let run = two_step_run(notify, resumed);

        notified.recv_timeout(PATIENCE).expect("a step is reported");
        assert_eq!(run.step(), Some(HALFWAY));
        assert!(run.take_result().is_none());

        resume.send(()).unwrap();
        assert_eq!(result_of(&run, &notified), Ok(()));
    }

    #[test]
    fn cancelled_run_is_stopped_at_its_next_step() {
        let (notify, notified) = channel();
        let (resume, resumed) = channel();
        let run = two_step_run(notify, resumed);
        notified.recv_timeout(PATIENCE).expect("a step is reported");

        run.cancel();
        resume.send(()).unwrap();

        assert_eq!(result_of(&run, &notified), Err(EnhancementError::Cancelled));
    }

    #[test]
    fn enhancement_that_panics_is_a_failed_enhancement() {
        let (notify, notified) = channel();

        let run = EnhancementRun::start(|_| panic!("model bug"), notifying(notify));

        let stopped = EnhancementError::Inference(STOPPED_UNEXPECTEDLY.to_owned());
        assert_eq!(result_of(&run, &notified), Err(stopped));
    }
}
