use std::path::PathBuf;
use crate::messages::Message;
use iced::Subscription;
use std::time::Duration;
use iced::futures::SinkExt;

pub async fn load_files() -> Vec<(PathBuf, String, Option<std::time::SystemTime>)> {
    let handles = rfd::AsyncFileDialog::new()
        .add_filter("Markdown", &["md", "markdown", "txt"])
        .set_title("Open Markdown Files")
        .pick_files()
        .await
        .unwrap_or_default();

    let mut result = Vec::new();
    for handle in handles {
        let path = handle.path().to_path_buf();
        if let Ok(content) = tokio::fs::read_to_string(&path).await {
            let mtime = tokio::fs::metadata(&path).await
                .ok()
                .and_then(|m| m.modified().ok());
            result.push((path, content, mtime));
        }
    }
    result
}

pub fn file_watcher(path_hash: u64) -> Subscription<Message> {
    Subscription::run_with(path_hash, |hash| {
        iced::stream::channel::<Message>(1, async move |mut output| {
              let mut interval = tokio::time::interval(Duration::from_millis(500));
              loop {
                  interval.tick().await;
                  let _ = output.send(Message::WatchTick).await;
            }
        })
    })
}
