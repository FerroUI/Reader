use reader_domain::ParsedFeed;
mod cancel;
pub use cancel::DownloadCancel;
use std::{
    io::Read,
    process::{Command, Stdio},
};

const FEED_LIMIT: usize = 8 * 1024 * 1024;
const IMAGE_LIMIT: usize = 8 * 1024 * 1024;
const EFFECTIVE_URL: &[u8] = b"\nREADER_EFFECTIVE_URL:";
pub struct FeedResponse {
    /// Subscription identity. Redirects change the parse base, not this address.
    pub url: String,
    pub feed: ParsedFeed,
}
struct Response {
    bytes: Vec<u8>,
    url: String,
}

/// Native requests do not need a CORS proxy. The operating system's curl supplies
/// HTTPS certificates, compression, redirects and a hard per-request timeout.
fn download(url: &str, limit: usize) -> Result<Response, String> {
    download_with_cancel(url, limit, &DownloadCancel::default())
}

fn download_with_cancel(
    url: &str,
    limit: usize,
    cancel: &DownloadCancel,
) -> Result<Response, String> {
    cancel.check()?;
    let url =
        reader_domain::safe_url(url, None).ok_or("Only HTTP and HTTPS URLs are supported.")?;
    let mut command = Command::new("curl");
    command
        .args([
            "--disable",
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--max-redirs",
            "5",
            "--proto",
            "=http,https",
            "--proto-redir",
            "=http,https",
            "--connect-timeout",
            "10",
            "--max-time",
            "25",
            "--compressed",
            "--user-agent",
            "FerroUI-Reader/0.1 (+https://github.com/FerroUI/Reader)",
            "--max-filesize",
            &limit.to_string(),
            "--write-out",
            "\nREADER_EFFECTIVE_URL:%{url_effective}",
            "--url",
            &url,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    let child = command.spawn().map_err(|error| {
        format!("Could not run curl: {error}. Install curl to retrieve content in the desktop app.")
    })?;
    let mut bytes = Vec::new();
    let read = cancel
        .attach(child)?
        .take((limit + 8193) as u64)
        .read_to_end(&mut bytes);
    if read.is_err() || bytes.len() > limit + 8192 {
        cancel.cancel();
        let _ = cancel.finish();
        return Err(read.err().map_or_else(
            || {
                format!(
                    "The response exceeds the {} MB limit.",
                    limit / (1024 * 1024)
                )
            },
            |e| e.to_string(),
        ));
    }
    let result = cancel.finish()?;
    cancel.check()?;
    if !result.status.success() {
        let error = String::from_utf8_lossy(&result.stderr).trim().to_owned();
        return Err(if error.is_empty() {
            format!("The request failed: {}", result.status)
        } else {
            error
        });
    }
    let separator = bytes
        .windows(EFFECTIVE_URL.len())
        .rposition(|s| s == EFFECTIVE_URL)
        .ok_or("The download did not report its final URL.")?;
    let effective = String::from_utf8_lossy(&bytes[separator + EFFECTIVE_URL.len()..]);
    let url = reader_domain::safe_url(&effective, None)
        .ok_or("The download redirected to an unsupported URL.")?;
    bytes.truncate(separator);
    if bytes.len() > limit {
        return Err(format!(
            "The response exceeds the {} MB limit.",
            limit / (1024 * 1024)
        ));
    }
    Ok(Response { bytes, url })
}

pub fn feed(url: String, now: i64, discover: bool) -> Result<FeedResponse, String> {
    let response = download(&url, FEED_LIMIT)?;
    let source =
        String::from_utf8(response.bytes).map_err(|_| "The feed must use UTF-8 text encoding.")?;
    // Resolve publisher links against curl's final response URL before crossing
    // back to the UI thread. MergeFeed retains `url` as the subscription identity.
    match reader_domain::parse_feed(&source, &response.url, now) {
        Ok(feed) => Ok(FeedResponse { url, feed }),
        Err(original_error) if discover => {
            let Some(discovered) = crate::discovery::feed_link(&source, &response.url) else {
                return Err(original_error);
            };
            if discovered == url {
                return Err(original_error);
            }
            feed(discovered, now, false)
        }
        Err(error) => Err(error),
    }
}

pub fn image(url: &str) -> Result<Vec<u8>, String> {
    download(url, IMAGE_LIMIT).map(|r| r.bytes)
}

pub fn article(
    url: &str,
    cancel: &DownloadCancel,
) -> Result<reader_content::ExtractedArticle, String> {
    let response = download_with_cancel(url, reader_content::MAX_DOCUMENT_BYTES, cancel)?;
    cancel.check()?;
    let source = String::from_utf8_lossy(&response.bytes);
    let content = reader_content::extract(&source, &response.url)?;
    cancel.check()?;
    Ok(content)
}
