use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum FileWatchEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
}

#[allow(dead_code)]
pub struct ImportFileWatcher {
    watcher: Option<notify::RecommendedWatcher>,
    event_sender: mpsc::Sender<FileWatchEvent>,
    #[cfg(test)]
    test_sender: Option<mpsc::Sender<FileWatchEvent>>,
}

#[allow(dead_code)]
impl ImportFileWatcher {
    pub fn new(event_sender: mpsc::Sender<FileWatchEvent>) -> Self {
        Self {
            watcher: None,
            #[cfg(test)]
            test_sender: Some(event_sender.clone()),
            event_sender,
        }
    }

    pub fn watch_directory(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let sender = self.event_sender.clone();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            if let Ok(event) = res {
                match event.kind {
                    EventKind::Create(_) => {
                        for path in event.paths {
                            let _ = sender.send(FileWatchEvent::Created(path));
                        }
                    }
                    EventKind::Modify(_) => {
                        for path in event.paths {
                            let _ = sender.send(FileWatchEvent::Modified(path));
                        }
                    }
                    EventKind::Remove(_) => {
                        for path in event.paths {
                            let _ = sender.send(FileWatchEvent::Deleted(path));
                        }
                    }
                    _ => {}
                }
            }
        })?;

        watcher.watch(&path, RecursiveMode::Recursive)?;
        self.watcher = Some(watcher);

        Ok(())
    }

    #[cfg(test)]
    pub fn trigger_test_event(&self, event: FileWatchEvent) {
        if let Some(sender) = &self.test_sender {
            let _ = sender.send(event);
        }
    }
}
