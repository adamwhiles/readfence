use crate::messages::Message;
use iced::Subscription;
use iced::futures::SinkExt;
use std::path::PathBuf;
use std::time::Duration;

pub async fn load_files() -> Vec<(PathBuf, String, Option<std::time::SystemTime>)> {
    let handles = rfd::AsyncFileDialog::new()
        .add_filter("Markdown", &["md", "markdown", "txt"])
        .set_title("Open Markdown Files")
        .pick_files()
        .await
        .unwrap_or_default();

    load_paths(
        handles
            .into_iter()
            .map(|handle| handle.path().to_path_buf())
            .collect(),
    )
    .await
}

pub async fn load_paths(
    paths: Vec<PathBuf>,
) -> Vec<(PathBuf, String, Option<std::time::SystemTime>)> {
    let mut result = Vec::new();
    for path in paths {
        if !is_supported_markdown_path(&path) {
            continue;
        }
        if let Ok(content) = tokio::fs::read_to_string(&path).await {
            let mtime = tokio::fs::metadata(&path)
                .await
                .ok()
                .and_then(|m| m.modified().ok());
            result.push((path, content, mtime));
        }
    }
    result
}

fn is_supported_markdown_path(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "md" | "markdown" | "txt"
            )
        })
        .unwrap_or(false)
}

pub fn file_watcher(path_hash: u64) -> Subscription<Message> {
    Subscription::run_with(path_hash, |_hash| {
        iced::stream::channel::<Message>(1, async move |mut output| {
            let mut interval = tokio::time::interval(Duration::from_millis(500));
            loop {
                interval.tick().await;
                let _ = output.send(Message::WatchTick).await;
            }
        })
    })
}
