//! Exercises native data-loss and transport boundaries without opening a window.
#![allow(dead_code)]

#[path = "../src/background.rs"]
mod background;
#[path = "../src/discovery.rs"]
mod discovery;
#[path = "../src/persistence.rs"]
mod persistence;
#[path = "../src/transport.rs"]
mod transport;
#[path = "../src/writer.rs"]
mod writer;

use std::{
    future::Future,
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    pin::pin,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    task::{Context, Poll, Wake, Waker},
    time::Duration,
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "reader-io-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn replacement_commits_a_complete_document_and_cleans_temporary_files() {
    let directory = Directory::new();
    let target = directory.0.join("reader.json");
    persistence::atomic_write(&target, b"original").unwrap();
    let replacement = "new document".repeat(4096);
    persistence::atomic_write(&target, replacement.as_bytes()).unwrap();
    assert_eq!(std::fs::read_to_string(&target).unwrap(), replacement);
    assert_eq!(std::fs::read_dir(&directory.0).unwrap().count(), 1);

    let blocked = directory.0.join("directory-not-a-file");
    std::fs::create_dir(&blocked).unwrap();
    std::fs::write(blocked.join("keep"), "preserved").unwrap();
    assert!(persistence::atomic_write(&blocked, b"cannot replace directory").is_err());
    assert_eq!(
        std::fs::read_to_string(blocked.join("keep")).unwrap(),
        "preserved"
    );
    assert_eq!(std::fs::read_dir(&directory.0).unwrap().count(), 2);
}

#[test]
fn malformed_library_is_preserved_before_new_saves_are_allowed() {
    let directory = Directory::new();
    let malformed = "{broken saved document";
    std::fs::write(directory.0.join("reader.json"), malformed).unwrap();
    let loaded = persistence::load(&directory.0, 1_700_000_000);
    assert!(loaded.writable);
    assert!(loaded.warning.as_deref().unwrap().contains("preserved"));
    assert!(loaded.state.feeds.is_empty());
    let recovery = std::fs::read_dir(&directory.0)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("reader-recovery-")
        })
        .unwrap();
    assert_eq!(std::fs::read_to_string(recovery).unwrap(), malformed);
    assert_eq!(
        std::fs::read_to_string(directory.0.join("reader.json")).unwrap(),
        malformed
    );
}

#[test]
fn shutdown_barrier_persists_the_latest_coalesced_snapshot_and_reports_failure() {
    let directory = Directory::new();
    let writer = writer::Writer::new(directory.0.clone(), true, false);
    for number in 0..100 {
        writer.submit(&format!("{{\"number\":{number}}}")).unwrap();
    }
    writer.wait().unwrap();
    assert_eq!(
        std::fs::read_to_string(directory.0.join("reader.json")).unwrap(),
        "{\"number\":99}"
    );
    writer.stop();
    assert!(writer.submit("after shutdown").is_err());

    let invalid = directory.0.join("ordinary-file");
    std::fs::write(&invalid, "original").unwrap();
    let writer = writer::Writer::new(invalid.clone(), true, false);
    writer.submit("replacement").unwrap();
    assert!(writer.wait().is_err());
    assert!(writer.error().is_some());
    assert_eq!(std::fs::read_to_string(invalid).unwrap(), "original");
}

#[test]
fn discovery_respects_base_urls_quoted_attributes_and_non_executing_content() {
    let source = r#"<!doctype html><head>
        <link rel="alternate" type="application/rss+xml">
        <!-- <link rel="alternate" type="application/rss+xml" href="/comment"> -->
        <script>var a = '<link rel="alternate" type="application/rss+xml" href="/script">';</script>
        <base href="../content/">
        <link title="A > B" rel="alternate stylesheet" type="application/atom+xml; charset=utf-8" href="feeds/latest?x=1&amp;y=2">
        </head>"#;
    assert_eq!(
        discovery::feed_link(source, "https://example.org/blog/index.html").as_deref(),
        Some("https://example.org/content/feeds/latest?x=1&y=2")
    );
    assert_eq!(
        discovery::feed_link(
            "<link rel=alternate type=application/rss+xml href=/feed/>",
            "https://example.org"
        )
        .as_deref(),
        Some("https://example.org/feed/")
    );
    assert!(discovery::feed_link(
        "<link rel=alternate type=application/rss+xml href=javascript:bad()>",
        "https://example.org"
    )
    .is_none());
}

struct WakeThread(std::thread::Thread);
impl Wake for WakeThread {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}
fn block_on<T>(future: impl Future<Output = T>) -> T {
    let waker = Waker::from(Arc::new(WakeThread(std::thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = pin!(future);
    loop {
        if let Poll::Ready(result) = future.as_mut().poll(&mut context) {
            return result;
        }
        std::thread::park_timeout(Duration::from_secs(5));
    }
}

#[test]
fn background_completion_wakes_the_waiter_and_keeps_work_off_the_calling_thread() {
    let owner = std::thread::current().id();
    let result = block_on(background::run(move || {
        assert_ne!(std::thread::current().id(), owner);
        Ok("completed")
    }));
    assert_eq!(result.unwrap(), "completed");
    assert_eq!(
        block_on(background::run(|| Err::<(), _>("request failed".into()))).unwrap_err(),
        "request failed"
    );
}

#[test]
fn native_transport_follows_redirects_and_resolves_relative_entry_links() {
    if std::process::Command::new("curl")
        .arg("--version")
        .output()
        .is_err()
    {
        return;
    }
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        for _ in 0..5 {
            let (mut connection, _) = listener.accept().unwrap();
            connection
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = [0; 4096];
            let length = connection.read(&mut request).unwrap();
            let path = std::str::from_utf8(&request[..length])
                .unwrap()
                .split_whitespace()
                .nth(1)
                .unwrap();
            if path == "/start" {
                connection.write_all(b"HTTP/1.1 302 Found\r\nLocation: /blog/index.html\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
            } else if path == "/old-feed.xml" {
                connection.write_all(b"HTTP/1.1 301 Moved Permanently\r\nLocation: /blog/feed.xml\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
            } else {
                let body = if path == "/blog/index.html" {
                    "<html><head><link rel='alternate' type='application/rss+xml' href='feed.xml'></head></html>"
                } else {
                    assert_eq!(path, "/blog/feed.xml");
                    "<rss version='2.0'><channel><title>Fixture</title><link>./</link><item><guid>item-1</guid><title>Article</title><link>articles/one</link><description>Body</description></item></channel></rss>"
                };
                write!(connection, "HTTP/1.1 200 OK\r\nContent-Type: application/xml\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            }
        }
    });
    let result = transport::feed(format!("http://{address}/start"), 1_700_000_000, true).unwrap();
    assert_eq!(result.url, format!("http://{address}/blog/feed.xml"));
    assert_eq!(result.feed.title, "Fixture");
    assert_eq!(
        result.feed.articles[0].url,
        format!("http://{address}/blog/articles/one")
    );
    let original = format!("http://{address}/old-feed.xml");
    let refreshed = transport::feed(original.clone(), 1_700_000_000, false).unwrap();
    assert_eq!(refreshed.url, original);
    assert_eq!(
        refreshed.feed.articles[0].url,
        format!("http://{address}/blog/articles/one")
    );
    server.join().unwrap();
}
