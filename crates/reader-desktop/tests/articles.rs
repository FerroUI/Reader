//! Local HTTP fixtures verify worker extraction and cancellation without a UI.
#![allow(dead_code)]

#[path = "../src/discovery.rs"]
mod discovery;
#[path = "../src/transport.rs"]
mod transport;

use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    time::{Duration, Instant},
};

fn curl_available() -> bool {
    std::process::Command::new("curl")
        .arg("--version")
        .output()
        .is_ok()
}

#[test]
fn native_article_extraction_uses_redirected_url_and_document_base() {
    if !curl_available() {
        return;
    }
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        for _ in 0..2 {
            let (mut connection, _) = listener.accept().unwrap();
            connection
                .set_read_timeout(Some(Duration::from_secs(3)))
                .unwrap();
            let mut request = [0; 4096];
            let count = connection.read(&mut request).unwrap();
            let path = std::str::from_utf8(&request[..count])
                .unwrap()
                .split_whitespace()
                .nth(1)
                .unwrap();
            if path == "/start" {
                connection.write_all(b"HTTP/1.1 302 Found\r\nLocation: /journal/story\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
            } else {
                assert_eq!(path, "/journal/story");
                let text = "An article should explain its subject clearly, provide examples, and retain meaningful formatting for the reader. This paragraph supplies enough publisher text for reliable article extraction. ".repeat(8);
                let body = format!(
                    "<!doctype html><title>Useful technical article</title><base href='../assets/'><nav>Unrelated navigation</nav><article><h1>Useful technical article</h1><p>{text}</p><p><a href='detail'>More detail</a></p><script>neverExecuted()</script></article>"
                );
                write!(connection, "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            }
        }
    });
    let content = transport::article(
        &format!("http://{address}/start"),
        &transport::DownloadCancel::default(),
    )
    .unwrap();
    assert_eq!(content.base_url, format!("http://{address}/assets/"));
    assert!(content.html.contains("An article should explain"));
    assert!(
        content
            .html
            .contains(&format!("http://{address}/assets/detail"))
    );
    assert!(!content.html.contains("Unrelated navigation"));
    assert!(!content.html.contains("neverExecuted"));
    server.join().unwrap();
}

#[test]
fn cancellation_stops_a_blocked_curl_and_prevents_queued_downloads() {
    let already_cancelled = transport::DownloadCancel::default();
    already_cancelled.cancel();
    assert!(
        transport::article("https://example.com/never-requested", &already_cancelled)
            .unwrap_err()
            .contains("cancelled")
    );
    if !curl_available() {
        return;
    }
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (connected, ready) = mpsc::channel();
    let server = std::thread::spawn(move || {
        let (mut connection, _) = listener.accept().unwrap();
        connection
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        let mut request = [0; 4096];
        connection.read(&mut request).unwrap();
        connected.send(()).unwrap();
        // Hold the response open; killing curl must close the socket promptly.
        assert_eq!(connection.read(&mut request).unwrap(), 0);
    });
    let cancel = transport::DownloadCancel::default();
    let running = cancel.clone();
    let (completed, result) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        completed
            .send(transport::article(
                &format!("http://{address}/blocked"),
                &running,
            ))
            .unwrap();
    });
    ready.recv_timeout(Duration::from_secs(3)).unwrap();
    let start = Instant::now();
    cancel.cancel();
    assert!(
        start.elapsed() < Duration::from_secs(1),
        "cancellation must not wait for network I/O"
    );
    assert!(
        result
            .recv_timeout(Duration::from_secs(3))
            .unwrap()
            .unwrap_err()
            .contains("cancelled")
    );
    worker.join().unwrap();
    server.join().unwrap();
}
