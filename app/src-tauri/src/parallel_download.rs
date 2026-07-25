use std::sync::Arc;
use tokio::sync::Mutex;
use grammers_client::types::Media;
use grammers_client::Client;

const PARALLEL_WORKERS: usize = 4;
const DOWNLOAD_CHUNK_SIZE: usize = 512 * 1024; // 512KB

/// Parallel MTProto chunk download manager that fetches disjoined ranges simultaneously
pub async fn download_media_parallel(
    client: &Client,
    media: &Media,
    total_size: u64,
    dest_path: &std::path::Path,
    progress_callback: impl Fn(u64, u64) + Send + Sync + 'static,
) -> Result<(), String> {
    let file = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(dest_path)
        .await
        .map_err(|e| format!("Failed to create destination file: {}", e))?;

    file.set_len(total_size)
        .await
        .map_err(|e| format!("Failed to preallocate file size: {}", e))?;

    let file_writer = Arc::new(Mutex::new(file));
    let downloaded_bytes = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let progress_cb = Arc::new(progress_callback);

    let total_chunks = ((total_size as f64) / (DOWNLOAD_CHUNK_SIZE as f64)).ceil() as usize;
    let chunks_per_worker = ((total_chunks as f64) / (PARALLEL_WORKERS as f64)).ceil() as usize;

    let mut tasks = Vec::new();

    for worker_idx in 0..PARALLEL_WORKERS {
        let start_chunk = worker_idx * chunks_per_worker;
        let end_chunk = std::cmp::min((worker_idx + 1) * chunks_per_worker, total_chunks);

        if start_chunk >= total_chunks {
            break;
        }

        let client_clone = client.clone();
        let media_clone = media.clone();
        let writer_clone = file_writer.clone();
        let downloaded_clone = downloaded_bytes.clone();
        let callback_clone = progress_cb.clone();

        let task = tokio::spawn(async move {
            let offset_bytes = (start_chunk * DOWNLOAD_CHUNK_SIZE) as u64;
            let mut download_iter = client_clone.iter_download(&media_clone);
            download_iter = download_iter.offset(offset_bytes as usize).chunk_size(DOWNLOAD_CHUNK_SIZE);

            let mut current_part = start_chunk;

            while current_part < end_chunk {
                match download_iter.next().await {
                    Ok(Some(chunk)) => {
                        let chunk_len = chunk.len() as u64;
                        let write_pos = (current_part * DOWNLOAD_CHUNK_SIZE) as u64;

                        {
                            let mut writer = writer_clone.lock().await;
                            use tokio::io::AsyncSeekExt;
                            use tokio::io::AsyncWriteExt;
                            writer.seek(std::io::SeekFrom::Start(write_pos)).await
                                .map_err(|e| format!("Seek failed at pos {}: {}", write_pos, e))?;
                            writer.write_all(&chunk).await
                                .map_err(|e| format!("Write failed at pos {}: {}", write_pos, e))?;
                        }

                        let new_total = downloaded_clone.fetch_add(chunk_len, std::sync::atomic::Ordering::Relaxed) + chunk_len;
                        callback_clone(new_total, total_size);

                        current_part += 1;
                        if chunk_len < DOWNLOAD_CHUNK_SIZE as u64 {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => return Err(format!("Parallel download worker {} error: {}", worker_idx, e)),
                }
            }
            Ok(())
        });

        tasks.push(task);
    }

    for task in tasks {
        match task.await {
            Ok(Ok(())) => {}
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(format!("Worker task join error: {}", e)),
        }
    }

    Ok(())
}
