//! A bounded blocking-I/O pool whose completions wake FerroUI's local executor.
use std::{
    future::Future,
    pin::Pin,
    sync::{mpsc, Arc, Mutex, OnceLock},
    task::{Context, Poll, Waker},
};

type Job = Box<dyn FnOnce() + Send>;
static POOL: OnceLock<Result<mpsc::Sender<Job>, String>> = OnceLock::new();
struct Completion<T> {
    result: Option<Result<T, String>>,
    waker: Option<Waker>,
}
pub struct Background<T> {
    completion: Arc<Mutex<Completion<T>>>,
}

fn pool() -> Result<&'static mpsc::Sender<Job>, String> {
    POOL.get_or_init(|| {
        let (sender, receiver) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver));
        for number in 0..4 {
            let receiver = receiver.clone();
            std::thread::Builder::new()
                .name(format!("reader-io-{number}"))
                .spawn(move || loop {
                    let task = receiver.lock().expect("I/O queue lock").recv();
                    match task {
                        Ok(task) => task(),
                        Err(_) => break,
                    }
                })
                .map_err(|error| format!("Could not start the I/O worker: {error}"))?;
        }
        Ok(sender)
    })
    .as_ref()
    .map_err(Clone::clone)
}

pub fn run<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Background<T> {
    let completion = Arc::new(Mutex::new(Completion {
        result: None,
        waker: None,
    }));
    let output = completion.clone();
    let task = Box::new(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work))
            .unwrap_or_else(|_| Err("The background operation failed unexpectedly.".into()));
        let waker = {
            let mut output = output.lock().expect("completion lock");
            output.result = Some(result);
            output.waker.take()
        };
        if let Some(waker) = waker {
            waker.wake();
        }
    });
    let queued = pool().and_then(|pool| {
        pool.send(task)
            .map_err(|_| "The I/O worker is no longer available.".into())
    });
    if let Err(error) = queued {
        completion.lock().expect("completion lock").result = Some(Err(error));
    }
    Background { completion }
}

impl<T> Future for Background<T> {
    type Output = Result<T, String>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut completion = self.completion.lock().expect("completion lock");
        if let Some(result) = completion.result.take() {
            Poll::Ready(result)
        } else {
            completion.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
