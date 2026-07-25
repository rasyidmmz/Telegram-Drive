use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BatchCcTask {
    pub message_id: i32,
    pub folder_id: Option<i64>,
    pub file_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchCcQueueStatus {
    pub total_queued: usize,
    pub completed_count: usize,
    pub current_file: Option<String>,
    pub is_running: bool,
}

pub struct BatchCcQueueManager {
    queue: Arc<Mutex<VecDeque<BatchCcTask>>>,
    is_running: Arc<Mutex<bool>>,
    current_file: Arc<Mutex<Option<String>>>,
    completed_count: Arc<Mutex<usize>>,
}

impl BatchCcQueueManager {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            is_running: Arc::new(Mutex::new(false)),
            current_file: Arc::new(Mutex::new(None)),
            completed_count: Arc::new(Mutex::new(0)),
        }
    }

    pub async fn enqueue_tasks(&self, tasks: Vec<BatchCcTask>) {
        let mut q = self.queue.lock().await;
        for t in tasks {
            q.push_back(t);
        }
    }

    pub async fn get_status(&self) -> BatchCcQueueStatus {
        let q = self.queue.lock().await;
        let running = *self.is_running.lock().await;
        let current = self.current_file.lock().await.clone();
        let completed = *self.completed_count.lock().await;

        BatchCcQueueStatus {
            total_queued: q.len(),
            completed_count: completed,
            current_file: current,
            is_running: running,
        }
    }

    pub async fn clear_queue(&self) {
        let mut q = self.queue.lock().await;
        q.clear();
        *self.current_file.lock().await = None;
        *self.is_running.lock().await = false;
    }
}
