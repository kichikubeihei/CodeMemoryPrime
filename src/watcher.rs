use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::error::Error;
use std::path::Path;
use tokio::sync::mpsc;

pub fn start_watcher(
    workspace_path: String,
) -> Result<mpsc::Receiver<String>, Box<dyn Error>> {
    let (tx, rx) = mpsc::channel::<String>(100);

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if let Ok(event) = res {
                if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
                    for path in event.paths {
                        if let Some(path_str) = path.to_str() {
                            let _ = tx.blocking_send(path_str.to_string());
                        }
                    }
                }
            }
        },
        Config::default(),
    )?;

    watcher.watch(Path::new(&workspace_path), RecursiveMode::Recursive)?;

    std::mem::forget(watcher);

    Ok(rx)
}
