use serde::Serialize;
use std::sync::{Mutex, OnceLock};

const MAX_TRANSFER_LOGS: usize = 100;

#[derive(Debug, Clone, Serialize)]
pub struct TransferLogEntry {
    pub time: String,
    pub source: String,
    pub message: String,
    pub details: Option<String>,
}

static TRANSFER_LOGS: OnceLock<Mutex<Vec<TransferLogEntry>>> = OnceLock::new();

pub(crate) fn record_transfer_log(
    source: impl Into<String>,
    message: impl Into<String>,
    details: Option<String>,
) {
    let entry = TransferLogEntry {
        time: chrono::Utc::now().to_rfc3339(),
        source: source.into(),
        message: message.into(),
        details,
    };
    let mut logs = logs().lock().unwrap();
    logs.insert(0, entry);
    logs.truncate(MAX_TRANSFER_LOGS);
}

pub(crate) fn transfer_logs() -> Vec<TransferLogEntry> {
    logs().lock().unwrap().clone()
}

pub(crate) fn clear_transfer_logs() {
    logs().lock().unwrap().clear();
}

#[tauri::command]
pub fn cmd_get_transfer_logs() -> Vec<TransferLogEntry> {
    transfer_logs()
}

#[tauri::command]
pub fn cmd_clear_transfer_logs() {
    clear_transfer_logs();
}

fn logs() -> &'static Mutex<Vec<TransferLogEntry>> {
    TRANSFER_LOGS.get_or_init(|| Mutex::new(Vec::new()))
}

#[cfg(test)]
mod tests {
    #[test]
    fn keeps_newest_entries_first_and_caps_old_entries() {
        super::clear_transfer_logs();

        for i in 0..105 {
            super::record_transfer_log("upload", format!("entry {i}"), None);
        }

        let logs = super::transfer_logs();
        assert_eq!(logs.len(), 100);
        assert_eq!(logs[0].message, "entry 104");
        assert_eq!(logs[99].message, "entry 5");
    }
}
