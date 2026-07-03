use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tauri::Manager;

const RESUME_VERSION: u8 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SplitUploadResumePart {
    pub index: usize,
    pub message_id: i32,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SplitUploadResumeState {
    pub version: u8,
    pub folder_id: Option<i64>,
    pub file_name: String,
    pub file_size: u64,
    pub part_size: u64,
    pub total_parts: usize,
    pub parts: Vec<SplitUploadResumePart>,
}

impl SplitUploadResumeState {
    pub(crate) fn new(
        folder_id: Option<i64>,
        file_name: String,
        file_size: u64,
        part_size: u64,
        total_parts: usize,
        parts: Vec<SplitUploadResumePart>,
    ) -> Self {
        Self {
            version: RESUME_VERSION,
            folder_id,
            file_name,
            file_size,
            part_size,
            total_parts,
            parts,
        }
    }

    pub(crate) fn matches_upload(
        &self,
        folder_id: Option<i64>,
        file_name: &str,
        file_size: u64,
        part_size: u64,
        total_parts: usize,
    ) -> bool {
        self.version == RESUME_VERSION
            && self.folder_id == folder_id
            && self.file_name == file_name
            && self.file_size == file_size
            && self.part_size == part_size
            && self.total_parts == total_parts
    }
}

pub(crate) fn expected_split_part_size(
    file_size: u64,
    part_size: u64,
    index: usize,
    total_parts: usize,
) -> Option<u64> {
    if part_size == 0 || index >= total_parts || total_parts == 0 {
        return None;
    }
    if index + 1 == total_parts {
        let used_before_last = part_size.checked_mul(index as u64)?;
        file_size.checked_sub(used_before_last)
    } else {
        Some(part_size)
    }
}

pub(crate) fn split_upload_state_path(
    app_handle: &tauri::AppHandle,
    folder_id: Option<i64>,
    file_name: &str,
    file_size: u64,
    part_size: u64,
) -> Result<PathBuf, String> {
    let dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Could not resolve app data directory: {}", e))?
        .join("split-upload-state");
    Ok(dir.join(format!(
        "{}.json",
        split_upload_state_key(folder_id, file_name, file_size, part_size)
    )))
}

pub(crate) async fn load_split_upload_state(
    path: &PathBuf,
) -> Option<SplitUploadResumeState> {
    let bytes = tokio::fs::read(path).await.ok()?;
    serde_json::from_slice(&bytes).ok()
}

pub(crate) async fn save_split_upload_state(
    path: &PathBuf,
    state: &SplitUploadResumeState,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("Failed to create split upload state directory: {}", e))?;
    }
    let bytes = serde_json::to_vec(state)
        .map_err(|e| format!("Failed to encode split upload state: {}", e))?;
    tokio::fs::write(path, bytes)
        .await
        .map_err(|e| format!("Failed to write split upload state: {}", e))
}

pub(crate) async fn clear_split_upload_state(path: &PathBuf) {
    let _ = tokio::fs::remove_file(path).await;
}

fn split_upload_state_key(
    folder_id: Option<i64>,
    file_name: &str,
    file_size: u64,
    part_size: u64,
) -> String {
    let raw = format!(
        "{}\0{}\0{}\0{}",
        folder_id.map(|v| v.to_string()).unwrap_or_else(|| "home".to_string()),
        file_name,
        file_size,
        part_size
    );
    let digest = Sha256::digest(raw.as_bytes());
    digest[..16].iter().map(|byte| format!("{:02x}", byte)).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn computes_last_split_part_size() {
        assert_eq!(super::expected_split_part_size(10, 4, 0, 3), Some(4));
        assert_eq!(super::expected_split_part_size(10, 4, 1, 3), Some(4));
        assert_eq!(super::expected_split_part_size(10, 4, 2, 3), Some(2));
        assert_eq!(super::expected_split_part_size(10, 4, 3, 3), None);
    }

    #[test]
    fn resume_state_matches_exact_upload_identity() {
        let state = super::SplitUploadResumeState::new(
            Some(42),
            "movie.mkv".to_string(),
            10,
            4,
            3,
            Vec::new(),
        );

        assert!(state.matches_upload(Some(42), "movie.mkv", 10, 4, 3));
        assert!(!state.matches_upload(Some(42), "other.mkv", 10, 4, 3));
        assert!(!state.matches_upload(None, "movie.mkv", 10, 4, 3));
    }
}
