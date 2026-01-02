//! Task coordination and background task management.

use crate::error::{ShellError, ShellResult};
use crate::AppId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

/// Unique task identifier.
pub type TaskId = u64;

/// Task status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is pending execution.
    Pending,
    /// Task is currently running.
    Running,
    /// Task completed successfully.
    Completed,
    /// Task failed.
    Failed,
    /// Task was cancelled.
    Cancelled,
}

impl TaskStatus {
    /// Check if task is terminal (won't change).
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    /// Check if task is active.
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Pending | Self::Running)
    }
}

/// Information about a task.
#[derive(Debug, Clone)]
pub struct TaskInfo {
    /// Task ID.
    pub id: TaskId,
    /// Task name.
    pub name: String,
    /// Owning app (if any).
    pub app_id: Option<AppId>,
    /// Current status.
    pub status: TaskStatus,
    /// Progress (0.0 to 1.0, if deterministic).
    pub progress: Option<f32>,
    /// Status message.
    pub message: Option<String>,
    /// When task started.
    pub started_at: Option<std::time::Instant>,
    /// When task completed.
    pub completed_at: Option<std::time::Instant>,
}

impl TaskInfo {
    /// Create new pending task.
    pub fn new(id: TaskId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            app_id: None,
            status: TaskStatus::Pending,
            progress: None,
            message: None,
            started_at: None,
            completed_at: None,
        }
    }

    /// Set owning app.
    pub fn with_app(mut self, app_id: AppId) -> Self {
        self.app_id = Some(app_id);
        self
    }

    /// Get elapsed time if running.
    pub fn elapsed(&self) -> Option<std::time::Duration> {
        self.started_at.map(|start| {
            self.completed_at
                .unwrap_or_else(std::time::Instant::now)
                .duration_since(start)
        })
    }
}

/// Task update event.
#[derive(Debug, Clone)]
pub enum TaskEvent {
    /// Task started.
    Started(TaskId),
    /// Task progress updated.
    Progress { id: TaskId, progress: f32, message: Option<String> },
    /// Task completed.
    Completed(TaskId),
    /// Task failed.
    Failed { id: TaskId, error: String },
    /// Task cancelled.
    Cancelled(TaskId),
}

/// Handle to update a running task.
pub struct TaskHandle {
    id: TaskId,
    update_tx: mpsc::Sender<TaskUpdate>,
}

impl TaskHandle {
    /// Report progress.
    pub async fn progress(&self, progress: f32, message: Option<String>) -> ShellResult<()> {
        self.update_tx
            .send(TaskUpdate::Progress {
                id: self.id,
                progress,
                message,
            })
            .await
            .map_err(|_| ShellError::Task("Failed to send progress update".to_string()))
    }

    /// Mark as completed.
    pub async fn complete(self) -> ShellResult<()> {
        self.update_tx
            .send(TaskUpdate::Complete(self.id))
            .await
            .map_err(|_| ShellError::Task("Failed to send completion".to_string()))
    }

    /// Mark as failed.
    pub async fn fail(self, error: impl Into<String>) -> ShellResult<()> {
        self.update_tx
            .send(TaskUpdate::Fail {
                id: self.id,
                error: error.into(),
            })
            .await
            .map_err(|_| ShellError::Task("Failed to send failure".to_string()))
    }

    /// Get task ID.
    pub fn id(&self) -> TaskId {
        self.id
    }
}

/// Internal task update message.
#[derive(Debug)]
enum TaskUpdate {
    Progress { id: TaskId, progress: f32, message: Option<String> },
    Complete(TaskId),
    Fail { id: TaskId, error: String },
    Cancel(TaskId),
}

/// Coordinates background tasks.
pub struct TaskCoordinator {
    /// All tasks.
    tasks: Arc<RwLock<HashMap<TaskId, TaskInfo>>>,
    /// Next task ID.
    next_id: TaskId,
    /// Update sender.
    update_tx: mpsc::Sender<TaskUpdate>,
    /// Update receiver.
    update_rx: mpsc::Receiver<TaskUpdate>,
    /// Event broadcaster.
    event_tx: broadcast::Sender<TaskEvent>,
    /// Maximum concurrent tasks.
    max_concurrent: usize,
}

impl TaskCoordinator {
    /// Create new coordinator.
    pub fn new(max_concurrent: usize) -> Self {
        let (update_tx, update_rx) = mpsc::channel(256);
        let (event_tx, _) = broadcast::channel(256);

        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            next_id: 1,
            update_tx,
            update_rx,
            event_tx,
            max_concurrent,
        }
    }

    /// Register a new task.
    pub async fn register(&mut self, name: impl Into<String>, app_id: Option<AppId>) -> TaskHandle {
        let id = self.next_id;
        self.next_id += 1;

        let mut info = TaskInfo::new(id, name);
        info.app_id = app_id;

        self.tasks.write().await.insert(id, info);

        TaskHandle {
            id,
            update_tx: self.update_tx.clone(),
        }
    }

    /// Start a task.
    pub async fn start(&mut self, id: TaskId) -> ShellResult<()> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(&id)
            .ok_or_else(|| ShellError::Task(format!("Task {} not found", id)))?;

        task.status = TaskStatus::Running;
        task.started_at = Some(std::time::Instant::now());

        let _ = self.event_tx.send(TaskEvent::Started(id));
        Ok(())
    }

    /// Cancel a task.
    pub async fn cancel(&mut self, id: TaskId) -> ShellResult<()> {
        self.update_tx
            .send(TaskUpdate::Cancel(id))
            .await
            .map_err(|_| ShellError::Task("Failed to send cancellation".to_string()))
    }

    /// Process pending updates.
    pub async fn process_updates(&mut self) {
        while let Ok(update) = self.update_rx.try_recv() {
            match update {
                TaskUpdate::Progress { id, progress, message } => {
                    if let Some(task) = self.tasks.write().await.get_mut(&id) {
                        task.progress = Some(progress);
                        task.message = message.clone();
                        let _ = self.event_tx.send(TaskEvent::Progress { id, progress, message });
                    }
                }
                TaskUpdate::Complete(id) => {
                    if let Some(task) = self.tasks.write().await.get_mut(&id) {
                        task.status = TaskStatus::Completed;
                        task.completed_at = Some(std::time::Instant::now());
                        task.progress = Some(1.0);
                        let _ = self.event_tx.send(TaskEvent::Completed(id));
                    }
                }
                TaskUpdate::Fail { id, error } => {
                    if let Some(task) = self.tasks.write().await.get_mut(&id) {
                        task.status = TaskStatus::Failed;
                        task.completed_at = Some(std::time::Instant::now());
                        task.message = Some(error.clone());
                        let _ = self.event_tx.send(TaskEvent::Failed { id, error });
                    }
                }
                TaskUpdate::Cancel(id) => {
                    if let Some(task) = self.tasks.write().await.get_mut(&id) {
                        task.status = TaskStatus::Cancelled;
                        task.completed_at = Some(std::time::Instant::now());
                        let _ = self.event_tx.send(TaskEvent::Cancelled(id));
                    }
                }
            }
        }
    }

    /// Get task info.
    pub async fn get(&self, id: TaskId) -> Option<TaskInfo> {
        self.tasks.read().await.get(&id).cloned()
    }

    /// Get all tasks.
    pub async fn list(&self) -> Vec<TaskInfo> {
        self.tasks.read().await.values().cloned().collect()
    }

    /// Get active tasks.
    pub async fn active(&self) -> Vec<TaskInfo> {
        self.tasks
            .read()
            .await
            .values()
            .filter(|t| t.status.is_active())
            .cloned()
            .collect()
    }

    /// Get tasks for an app.
    pub async fn for_app(&self, app_id: AppId) -> Vec<TaskInfo> {
        self.tasks
            .read()
            .await
            .values()
            .filter(|t| t.app_id == Some(app_id))
            .cloned()
            .collect()
    }

    /// Subscribe to task events.
    pub fn subscribe(&self) -> broadcast::Receiver<TaskEvent> {
        self.event_tx.subscribe()
    }

    /// Clean up completed tasks older than duration.
    pub async fn cleanup(&mut self, older_than: std::time::Duration) {
        let now = std::time::Instant::now();
        self.tasks.write().await.retain(|_, task| {
            if let Some(completed_at) = task.completed_at {
                now.duration_since(completed_at) < older_than
            } else {
                true
            }
        });
    }

    /// Check if at capacity.
    pub async fn at_capacity(&self) -> bool {
        self.active().await.len() >= self.max_concurrent
    }

    /// Get active task count.
    pub async fn active_count(&self) -> usize {
        self.active().await.len()
    }
}

impl Default for TaskCoordinator {
    fn default() -> Self {
        Self::new(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_lifecycle() {
        let mut coord = TaskCoordinator::new(5);

        let handle = coord.register("test-task", None).await;
        let id = handle.id();

        coord.start(id).await.unwrap();

        let task = coord.get(id).await.unwrap();
        assert_eq!(task.status, TaskStatus::Running);

        handle.complete().await.unwrap();
        coord.process_updates().await;

        let task = coord.get(id).await.unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
    }

    #[tokio::test]
    async fn test_task_progress() {
        let mut coord = TaskCoordinator::new(5);

        let handle = coord.register("progress-task", None).await;
        let id = handle.id();

        coord.start(id).await.unwrap();
        handle.progress(0.5, Some("Halfway".to_string())).await.unwrap();
        coord.process_updates().await;

        let task = coord.get(id).await.unwrap();
        assert_eq!(task.progress, Some(0.5));
        assert_eq!(task.message.as_deref(), Some("Halfway"));
    }

    #[test]
    fn test_task_status() {
        assert!(TaskStatus::Pending.is_active());
        assert!(TaskStatus::Running.is_active());
        assert!(!TaskStatus::Completed.is_active());

        assert!(TaskStatus::Completed.is_terminal());
        assert!(TaskStatus::Failed.is_terminal());
        assert!(!TaskStatus::Running.is_terminal());
    }
}
