use std::sync::Arc;
use tokio::sync::Mutex;
use grammers_client::Client;

/// Session Health Manager that runs background ping health checks every 60 seconds
pub struct SessionHealthManager {
    ping_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl SessionHealthManager {
    pub fn new() -> Self {
        Self {
            ping_handle: Mutex::new(None),
        }
    }

    pub async fn start_monitoring(
        &self,
        client: Client,
        peer_cache: Arc<tokio::sync::RwLock<std::collections::HashMap<i64, grammers_client::types::Peer>>>,
    ) {
        let mut handle_guard = self.ping_handle.lock().await;
        if let Some(old_task) = handle_guard.take() {
            old_task.abort();
        }

        let new_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                interval.tick().await;
                match client.is_authorized().await {
                    Ok(true) => {
                        log::debug!("MTProto session health check: OK");
                    }
                    Ok(false) => {
                        log::warn!("MTProto session health check: Session unauthorized. Clearing peer cache.");
                        peer_cache.write().await.clear();
                    }
                    Err(e) => {
                        log::warn!("MTProto session health check ping failed: {}. Retrying peer cache refresh.", e);
                        peer_cache.write().await.clear();
                    }
                }
            }
        });

        *handle_guard = Some(new_task);
    }

    pub async fn stop_monitoring(&self) {
        let mut handle_guard = self.ping_handle.lock().await;
        if let Some(task) = handle_guard.take() {
            task.abort();
        }
    }
}
