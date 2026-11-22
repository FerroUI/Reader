use crate::{background, persistence};
use ferroui_base::{
    platform::storage::{FilePickerFileType, FilePickerOpenOptions, FilePickerSaveOptions},
    Ref,
};
use ferroui_controls::Window;
use reader_state::AppViewModel;
use std::{
    io::{Read, Write},
    path::PathBuf,
    rc::Rc,
};

pub async fn export(
    window: &Ref<Window>,
    filename: String,
    contents: String,
) -> Result<(), String> {
    let mut options = FilePickerSaveOptions::new();
    options.set_title(Some("Export Reader library".into()));
    options.set_default_extension(filename.rsplit_once('.').map(|(_, e)| e.to_owned()));
    options.set_suggested_file_name(Some(filename));
    let Some(file) = window
        .storage_provider()
        .save_file_picker_async(options)
        .await
        .map_err(|e| e.to_string())?
    else {
        return Ok(());
    };
    let path = file.path().as_url().and_then(|url| url.to_file_path().ok());
    if let Some(path) = path {
        background::run(move || persistence::atomic_write(&path, contents.as_bytes())).await?;
    } else {
        let mut stream = file.open_write_async().await.map_err(|e| e.to_string())?;
        stream
            .write_all(contents.as_bytes())
            .map_err(|e| e.to_string())?;
        stream.flush().map_err(|e| e.to_string())?;
    }
    file.dispose();
    Ok(())
}

pub async fn import(
    vm: &Rc<AppViewModel>,
    window: &Ref<Window>,
    format: String,
) -> Result<(), String> {
    let mut options = FilePickerOpenOptions::new();
    options.set_title(Some("Import into Reader".into()));
    let patterns: &[&str] = if format == "opml" {
        &["*.opml", "*.xml"]
    } else {
        &["*.json"]
    };
    options.set_file_type_filter(Some(vec![FilePickerFileType::new(Some(
        if format == "opml" {
            "Feed subscriptions"
        } else {
            "Reader backup"
        },
    ))
    .with_patterns(patterns)]));
    let files = window
        .storage_provider()
        .open_file_picker_async(options)
        .await
        .map_err(|e| e.to_string())?;
    let Some(file) = files.first() else {
        return Ok(());
    };
    let path = file.path().as_url().and_then(|url| url.to_file_path().ok());
    let contents = if let Some(path) = path {
        background::run(move || persistence::read_document(&path)).await?
    } else {
        let stream = file.open_read_async().await.map_err(|e| e.to_string())?;
        let mut contents = String::new();
        stream
            .take(64 * 1024 * 1024 + 1)
            .read_to_string(&mut contents)
            .map_err(|e| e.to_string())?;
        if contents.len() > 64 * 1024 * 1024 {
            return Err("The import exceeds the 64 MB limit.".into());
        }
        contents
    };
    file.dispose();
    if format == "opml" {
        vm.import_opml(&contents)?;
    } else {
        vm.import_json(&contents)?;
    }
    Ok(())
}

pub async fn print(
    window: &Ref<Window>,
    directory: PathBuf,
    title: String,
    html: String,
) -> Result<(), String> {
    let path = directory.join("print-article.html");
    let target = path.clone();
    background::run(move || {
        let text = escape(&reader_domain::plain_text(&html));
        let document = format!("<!doctype html><html><head><meta charset=\"utf-8\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; style-src 'unsafe-inline'\"><title>{}</title><style>body{{font:16px/1.6 Arial,sans-serif;max-width:760px;margin:40px auto;padding:24px}}h1{{font-size:26px}}@media print{{.hint{{display:none}}body{{margin:0}}}}</style></head><body><h1>{}</h1><p>{text}</p><p class=\"hint\">Use your browser's Print command to print this article.</p></body></html>", escape(&title), escape(&title));
        persistence::atomic_write(&target, document.as_bytes())
    }).await?;
    if window.launcher().launch_file_info_async(&path).await {
        Ok(())
    } else {
        Err("Could not open the print preview.".into())
    }
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
