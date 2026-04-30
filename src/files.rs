use std::path::PathBuf;

pub async fn load_files() -> Vec<(PathBuf, String)> {
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
            result.push((path, content));
        }
    }
    result
}