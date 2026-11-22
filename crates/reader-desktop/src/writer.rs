//! One writer, a replaceable pending snapshot and a durable shutdown barrier.
use crate::persistence;
use std::{
    path::PathBuf,
    sync::{Arc, Condvar, Mutex},
    thread::JoinHandle,
};

#[derive(Default)]
struct Pending {
    json: Option<String>,
    writing: bool,
    stopped: bool,
    error: Option<String>,
}
pub struct Writer {
    state: Arc<(Mutex<Pending>, Condvar)>,
    thread: Mutex<Option<JoinHandle<()>>>,
    writable: bool,
    demo: bool,
}

impl Writer {
    pub fn new(directory: PathBuf, writable: bool, demo: bool) -> Self {
        let state = Arc::new((Mutex::new(Pending::default()), Condvar::new()));
        let worker_state = state.clone();
        let worker = if writable && !demo {
            std::thread::Builder::new()
                .name("reader-save".into())
                .spawn(move || loop {
                    let (lock, signal) = &*worker_state;
                    let json = {
                        let mut pending = lock.lock().expect("save queue lock");
                        while pending.json.is_none() && !pending.stopped {
                            pending = signal.wait(pending).expect("save queue lock");
                        }
                        let Some(json) = pending.json.take() else {
                            break;
                        };
                        pending.writing = true;
                        json
                    };
                    let result =
                        persistence::atomic_write(&directory.join("reader.json"), json.as_bytes());
                    let mut pending = lock.lock().expect("save queue lock");
                    pending.error = result.err();
                    pending.writing = false;
                    signal.notify_all();
                })
                .map(Some)
        } else {
            Ok(None)
        };
        let thread = match worker {
            Ok(thread) => thread,
            Err(error) => {
                state.0.lock().expect("save queue lock").error = Some(error.to_string());
                None
            }
        };
        Self {
            state,
            thread: Mutex::new(thread),
            writable,
            demo,
        }
    }

    pub fn submit(&self, json: &str) -> Result<(), String> {
        if self.demo {
            return Ok(());
        }
        if !self.writable {
            return Err("Saving is disabled until the original library has been backed up.".into());
        }
        let (lock, signal) = &*self.state;
        let mut pending = lock.lock().expect("save queue lock");
        if pending.stopped || self.thread.lock().expect("writer thread lock").is_none() {
            return Err(pending
                .error
                .clone()
                .unwrap_or_else(|| "The library writer has stopped.".into()));
        }
        pending.json = Some(json.to_owned());
        signal.notify_one();
        Ok(())
    }

    pub fn wait(&self) -> Result<(), String> {
        let (lock, signal) = &*self.state;
        let mut pending = lock.lock().expect("save queue lock");
        while pending.writing || pending.json.is_some() {
            pending = signal.wait(pending).expect("save queue lock");
        }
        pending.error.clone().map_or(Ok(()), Err)
    }

    pub fn error(&self) -> Option<String> {
        self.state.0.lock().expect("save queue lock").error.clone()
    }

    pub fn stop(&self) {
        {
            let mut pending = self.state.0.lock().expect("save queue lock");
            pending.stopped = true;
            self.state.1.notify_all();
        }
        if let Some(thread) = self.thread.lock().expect("writer thread lock").take() {
            let _ = thread.join();
        }
    }
}
impl Drop for Writer {
    fn drop(&mut self) {
        self.stop();
    }
}
