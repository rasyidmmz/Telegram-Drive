use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};
use serde::{Serialize, Deserialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EnglishCcPhase {
    Idle,
    Extracting,
    Transcribing,
    Ready,
    Error,
    Cancelled,
}

#[derive(Serialize, Clone)]
pub struct EnglishCcStatus {
    pub file_key: String,
    pub phase: EnglishCcPhase,
    pub progress: Option<f32>,
    pub cached: bool,
    pub error: Option<String>,
}

pub struct ActiveJob {
    pub message_id: i64,
    pub folder_id: Option<i64>,
    pub phase: EnglishCcPhase,
    pub progress: Option<f32>,
    pub error: Option<String>,
    pub cancel_tx: Option<oneshot::Sender<()>>,
}

pub struct CcManagerState {
    pub active_job: Option<ActiveJob>,
}

pub struct EnglishCcManager {
    pub state: Arc<Mutex<CcManagerState>>,
}

impl EnglishCcManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(CcManagerState { active_job: None })),
        }
    }
}

#[tauri::command]
pub async fn cmd_generate_english_cc(
    message_id: i64,
    folder_id: Option<i64>,
    force: bool,
    manager: State<'_, EnglishCcManager>,
    app_handle: tauri::AppHandle,
) -> Result<EnglishCcStatus, String> {
    let state = manager.state.lock().await;
    let file_key = format!("{}_{}", folder_id.unwrap_or(0), message_id);
    Ok(EnglishCcStatus {
        file_key,
        phase: EnglishCcPhase::Idle,
        progress: None,
        cached: false,
        error: None,
    })
}

#[tauri::command]
pub async fn cmd_get_english_cc_status(
    message_id: i64,
    folder_id: Option<i64>,
    manager: State<'_, EnglishCcManager>,
) -> Result<EnglishCcStatus, String> {
    let state = manager.state.lock().await;
    let file_key = format!("{}_{}", folder_id.unwrap_or(0), message_id);
    Ok(EnglishCcStatus {
        file_key,
        phase: EnglishCcPhase::Idle,
        progress: None,
        cached: false,
        error: None,
    })
}

#[tauri::command]
pub async fn cmd_cancel_english_cc(
    message_id: i64,
    folder_id: Option<i64>,
    manager: State<'_, EnglishCcManager>,
) -> Result<(), String> {
    Ok(())
}
