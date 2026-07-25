use std::sync::Arc;
use tokio::sync::Mutex;
use grammers_client::Client;

const UPLOAD_PARALLEL_WORKERS: usize = 4;
const UPLOAD_CHUNK_SIZE: usize = 512 * 1024; // 512 KB

/// Upload a large file concurrently across 4 MTProto worker connections
pub async fn upload_file_parallel(
    client: &Client,
    file_path: &std::path::Path,
    file_size: u64,
    file_id: i64,
    progress_cb: impl Fn(u64, u64) + Send + Sync + 'static,
) -> Result<(), String> {
    let total_parts = ((file_size as f64) / (UPLOAD_CHUNK_SIZE as f64)).ceil() as usize;
    let parts_per_worker = ((total_parts as f64) / (UPLOAD_PARALLEL_WORKERS as f64)).ceil() as usize;

    let uploaded_bytes = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let progress_callback = Arc::new(progress_cb);

    let mut tasks = Vec::new();

    for worker_idx in 0..UPLOAD_PARALLEL_WORKERS {
        let start_part = worker_idx * parts_per_worker;
        let end_part = std::cmp::min((worker_idx + 1) * parts_per_worker, total_parts);

        if start_part >= total_parts {
            break;
        }

        let client_clone = client.clone();
        let path_clone = file_path.to_path_buf();
        let uploaded_clone = uploaded_bytes.clone();
        let callback_clone = progress_callback.clone();

        let task = tokio::spawn(async move {
            let mut file = tokio::fs::File::open(&path_clone)
                .await
                .map_err(|e| format!("Worker {} failed to open file: {}", worker_idx, e))?;

            for part_idx in start_part..end_part {
                let offset = (part_idx * UPLOAD_CHUNK_SIZE) as u64;
                use tokio::io::AsyncSeekExt;
                use tokio::io::AsyncReadExt;

                file.seek(std::io::SeekFrom::Start(offset))
                    .await
                    .map_err(|e| format!("Worker {} seek failed: {}", worker_idx, e))?;

                let read_size = std::cmp::min(UPLOAD_CHUNK_SIZE as u64, file_size.saturating_sub(offset)) as usize;
                let mut buffer = vec![0u8; read_size];
                file.read_exact(&mut buffer)
                    .await
                    .map_err(|e| format!("Worker {} read failed: {}", worker_idx, e))?;

                // Save big file part to Telegram DC
                client_clone
                    .upload_big_file_part(file_id, part_idx as i32, total_parts as i32, buffer)
                    .await
                    .map_err(|e| format!("Worker {} saveBigFilePart error: {}", worker_idx, e))?;

                let current = uploaded_clone.fetch_add(read_size as u64, std::sync::atomic::Ordering::Relaxed) + (read_size as u64);
                callback_clone(current, file_size);

                // Micro-sleep to maintain rate-limit pacing
                tokio::time::sleep(std::time::Duration::from_millis(15)).await;
            }
            Ok::<(), String>(())
        });

        tasks.push(task);
    }

    for task in tasks {
        match task.await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(format!("Upload worker task join error: {}", e)),
        }
    }

    Ok(())
}
