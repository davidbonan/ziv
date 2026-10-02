use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::sync::{Arc, Mutex};
use std::thread;

use super::zone_detector::{DetectionError, DetectionStep};

const STOPPED_UNEXPECTEDLY: &str = "the detection stopped unexpectedly";

/// One detection running on its own thread. Dropping the run abandons it:
/// what it finds is thrown away.
pub struct DetectionRun<Detected> {
    step: Arc<Mutex<Option<DetectionStep>>>,
    result: Receiver<Result<Detected, DetectionError>>,
}

impl<Detected: Send + 'static> DetectionRun<Detected> {
    /// `detect` is told to report its steps; `notify` runs at each step and
    /// when the detection is over.
    pub fn start(
        detect: impl FnOnce(&mut dyn FnMut(DetectionStep)) -> Result<Detected, DetectionError>
        + Send
        + 'static,
        notify: impl Fn() + Send + 'static,
    ) -> Self {
        let step = Arc::new(Mutex::new(None));
        let (send_result, result) = channel();
        let reported_step = step.clone();
        thread::spawn(move || {
            let mut on_step = |step| {
                *reported_step
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(step);
                notify();
            };
            let detected = detect(&mut on_step);
            let _ = send_result.send(detected);
            notify();
        });
        Self { step, result }
    }

    /// What the detection is busy with; `None` before its first step.
    pub fn step(&self) -> Option<DetectionStep> {
        *self
            .step
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// What was detected, once; `None` as long as the detection runs.
    pub fn take_result(&self) -> Option<Result<Detected, DetectionError>> {
        match self.result.try_recv() {
            Ok(result) => Some(result),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => Some(Err(DetectionError::Inference(
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

    fn notifying(notified: Sender<()>) -> impl Fn() + Send + 'static {
        move || {
            let _ = notified.send(());
        }
    }

    fn result_of<Detected: Send + 'static>(
        run: &DetectionRun<Detected>,
        notified: &Receiver<()>,
    ) -> Result<Detected, DetectionError> {
        loop {
            if let Some(result) = run.take_result() {
                return result;
            }
            notified.recv_timeout(PATIENCE).expect("the detection ends");
        }
    }

    #[test]
    fn run_reports_its_step_then_what_it_detected() {
        let (notify, notified) = channel();
        let (resume, resumed) = channel::<()>();
        let run = DetectionRun::start(
            move |on_step| {
                on_step(DetectionStep::Detecting);
                let _ = resumed.recv_timeout(PATIENCE);
                Ok("a mask")
            },
            notifying(notify),
        );

        notified.recv_timeout(PATIENCE).expect("a step is reported");
        assert_eq!(run.step(), Some(DetectionStep::Detecting));
        assert!(run.take_result().is_none());

        resume.send(()).unwrap();
        assert_eq!(result_of(&run, &notified), Ok("a mask"));
    }

    #[test]
    fn detection_that_panics_is_a_failed_detection() {
        let (notify, notified) = channel();
        let run = DetectionRun::<()>::start(|_| panic!("model bug"), notifying(notify));

        while run.take_result().is_none() {
            let _ = notified.recv_timeout(Duration::from_millis(10));
        }
    }
}
