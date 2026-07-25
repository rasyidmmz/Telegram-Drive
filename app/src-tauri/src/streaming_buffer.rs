use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents an in-memory cached chunk of media bytes.
#[derive(Clone)]
pub struct CachedChunk {
    pub offset: u64,
    pub data: Vec<u8>,
}

/// A thread-safe ring buffer that stores up to 16MB of forward media pre-fetch data per stream.
pub struct MediaStreamBuffer {
    chunks: HashMap<u64, Vec<u8>>,
    max_bytes: usize,
    current_bytes: usize,
}

impl MediaStreamBuffer {
    pub fn new(max_mb: usize) -> Self {
        Self {
            chunks: HashMap::new(),
            max_bytes: max_mb * 1024 * 1024,
            current_bytes: 0,
        }
    }

    pub fn get_chunk(&self, offset: u64) -> Option<Vec<u8>> {
        self.chunks.get(&offset).cloned()
    }

    pub fn insert_chunk(&mut self, offset: u64, data: Vec<u8>) {
        if self.chunks.contains_key(&offset) {
            return;
        }

        let chunk_len = data.len();
        // Evict oldest chunks if capacity exceeded
        while self.current_bytes + chunk_len > self.max_bytes && !self.chunks.is_empty() {
            if let Some(&min_offset) = self.chunks.keys().min() {
                if let Some(removed) = self.chunks.remove(&min_offset) {
                    self.current_bytes = self.current_bytes.saturating_sub(removed.len());
                }
            }
        }

        self.current_bytes += chunk_len;
        self.chunks.insert(offset, data);
    }

    pub fn invalidate_before(&mut self, offset: u64) {
        let keys_to_remove: Vec<u64> = self
            .chunks
            .keys()
            .copied()
            .filter(|&k| k < offset.saturating_sub(2 * 1024 * 1024))
            .collect();

        for k in keys_to_remove {
            if let Some(removed) = self.chunks.remove(&k) {
                self.current_bytes = self.current_bytes.saturating_sub(removed.len());
            }
        }
    }

    pub fn clear(&mut self) {
        self.chunks.clear();
        self.current_bytes = 0;
    }
}

pub type SharedStreamBuffer = Arc<RwLock<MediaStreamBuffer>>;

pub fn create_stream_buffer(max_mb: usize) -> SharedStreamBuffer {
    Arc::new(RwLock::new(MediaStreamBuffer::new(max_mb)))
}
