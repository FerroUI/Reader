use reader_domain::AppState;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

const DOCUMENT_LIMIT: u64 = 64 * 1024 * 1024;
static NEXT_TEMPORARY: AtomicU64 = AtomicU64::new(1);
pub struct Loaded {
    pub state: AppState,
    pub warning: Option<String>,
    pub writable: bool,
    pub changed: bool,
}

pub fn data_directory() -> PathBuf {
    if let Some(path) = std::env::var_os("READER_DATA_DIR") {
        return path.into();
    }
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base =
        std::env::var_os("HOME").map(|p| PathBuf::from(p).join("Library/Application Support"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")));
    base.unwrap_or_else(std::env::temp_dir)
        .join("FerroUI/Reader")
}

pub fn read_document(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut text = String::new();
    file.take(DOCUMENT_LIMIT + 1)
        .read_to_string(&mut text)
        .map_err(|e| e.to_string())?;
    if text.len() as u64 > DOCUMENT_LIMIT {
        return Err("The file exceeds the 64 MB limit.".into());
    }
    Ok(text)
}

pub fn load(directory: &Path, _now: i64) -> Loaded {
    let path = directory.join("reader.json");
    if !path.exists() {
        return Loaded {
            state: AppState::with_default_feeds(),
            warning: None,
            writable: true,
            changed: true,
        };
    }
    match read_document(&path).and_then(|text| AppState::from_json(&text)) {
        Ok(mut state) => {
            let changed = state.migrate_legacy_samples();
            Loaded {
                state,
                warning: None,
                writable: true,
                changed,
            }
        }
        Err(error) => {
            let recovery = directory.join(format!(
                "reader-recovery-{}-{}.json",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            ));
            let copied = std::fs::copy(&path, &recovery).is_ok();
            let explanation = if copied {
                format!("The original is preserved in {}.", recovery.display())
            } else {
                "Saving is disabled to preserve the original. Copy reader.json elsewhere before restarting.".into()
            };
            Loaded {
                state: AppState::new(),
                warning: Some(format!(
                    "Could not load your saved library: {error}. {explanation}"
                )),
                writable: copied,
                changed: false,
            }
        }
    }
}

/// Replace only after the complete new file has reached disk. A failed write leaves
/// the previous document intact; temporary files always reside on the same volume.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or("The save path has no parent directory.")?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let name = path
        .file_name()
        .ok_or("The save path has no file name.")?
        .to_string_lossy();
    let temporary = parent.join(format!(
        ".{name}.{}.{}.tmp",
        std::process::id(),
        NEXT_TEMPORARY.fetch_add(1, Ordering::Relaxed)
    ));
    let write = || -> std::io::Result<()> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(&temporary, path)?;
        #[cfg(unix)]
        File::open(parent)?.sync_all()?;
        Ok(())
    };
    let result = write().map_err(|e| format!("{}: {e}", path.display()));
    if result.is_err() {
        let _ = std::fs::remove_file(temporary);
    }
    result
}
