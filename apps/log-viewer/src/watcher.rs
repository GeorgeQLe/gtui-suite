use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::time::Duration;

/// Watches a file for changes
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    rx: Receiver<notify::Result<notify::Event>>,
}

impl FileWatcher {
    pub fn new(path: &str) -> notify::Result<Self> {
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default().with_poll_interval(Duration::from_millis(500)),
        )?;

        watcher.watch(Path::new(path), RecursiveMode::NonRecursive)?;

        Ok(Self {
            _watcher: watcher,
            rx,
        })
    }

    /// Check if file has been modified
    pub fn check(&mut self) -> bool {
        loop {
            match self.rx.try_recv() {
                Ok(Ok(event)) => {
                    if event.kind.is_modify() {
                        return true;
                    }
                }
                Ok(Err(_)) => return false,
                Err(TryRecvError::Empty) => return false,
                Err(TryRecvError::Disconnected) => return false,
            }
        }
    }
}
