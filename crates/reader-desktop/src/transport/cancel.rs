//! The UI can stop curl without waiting for a blocked download worker.
use std::{
    io::Read,
    process::{Child, ChildStdout, Output},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

#[derive(Default)]
struct State {
    cancelled: AtomicBool,
    child: Mutex<Option<Child>>,
}

#[derive(Clone, Default)]
pub struct DownloadCancel(Arc<State>);

impl DownloadCancel {
    pub fn cancel(&self) {
        self.0.cancelled.store(true, Ordering::Release);
        if let Some(child) = self.0.child.lock().expect("download process lock").as_mut() {
            let _ = child.kill();
        }
    }

    pub fn check(&self) -> Result<(), String> {
        if self.0.cancelled.load(Ordering::Acquire) {
            Err("Article download cancelled.".into())
        } else {
            Ok(())
        }
    }

    pub(super) fn attach(&self, mut child: Child) -> Result<ChildStdout, String> {
        let stdout = child
            .stdout
            .take()
            .ok_or("The download stream is unavailable.")?;
        let mut slot = self.0.child.lock().expect("download process lock");
        if self.check().is_err() {
            let _ = child.kill();
        }
        *slot = Some(child);
        Ok(stdout)
    }

    /// Keep the process cancellable until it exits. The worker drains stdout
    /// first; only these brief try_wait calls share the UI's cancellation lock.
    pub(super) fn finish(&self) -> Result<Output, String> {
        loop {
            let finished = {
                let mut slot = self.0.child.lock().expect("download process lock");
                let status = slot
                    .as_mut()
                    .ok_or("The download process is unavailable.")?
                    .try_wait()
                    .map_err(|error| error.to_string())?;
                status.map(|status| {
                    let mut child = slot.take().expect("completed download process");
                    let stderr = child.stderr.take();
                    (status, stderr)
                })
            };
            if let Some((status, stderr)) = finished {
                let mut error = Vec::new();
                if let Some(stderr) = stderr {
                    stderr
                        .take(8192)
                        .read_to_end(&mut error)
                        .map_err(|e| e.to_string())?;
                }
                return Ok(Output {
                    status,
                    stdout: Vec::new(),
                    stderr: error,
                });
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.get_mut().expect("download process lock").take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
