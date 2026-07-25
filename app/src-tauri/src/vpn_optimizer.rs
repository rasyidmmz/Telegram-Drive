//! Fixed transfer policy for direct Telegram connections.

pub struct NetworkConfig;

impl NetworkConfig {
    pub fn new() -> Self {
        Self
    }

    pub fn connect_timeout_secs(&self) -> u64 {
        5
    }

    pub fn rw_timeout_secs(&self) -> u64 {
        10
    }

    pub fn retry_attempts(&self) -> u32 {
        0
    }

    pub fn retry_base_backoff_ms(&self) -> u64 {
        1000
    }

    pub fn retry_max_backoff_ms(&self) -> u64 {
        30000
    }

    pub fn should_respect_flood_wait(&self) -> bool {
        true
    }

    pub fn peer_cache_size(&self) -> usize {
        500
    }

    pub fn upload_limit_bytes_per_sec(&self) -> u64 {
        0
    }

    pub fn download_limit_bytes_per_sec(&self) -> u64 {
        0
    }

    pub fn chunk_size_bytes(&self) -> usize {
        512 * 1024
    }

    pub fn archive_max_bytes(&self) -> u64 {
        256 * 1024 * 1024
    }
}

pub fn backoff_ms(attempt: u32, base_ms: u64, max_ms: u64) -> u64 {
    let capped = base_ms.saturating_mul(1u64 << attempt.min(10)).min(max_ms);
    capped + (capped as f64 * 0.25 * rand::random::<f64>()) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_transfer_policy_is_unthrottled() {
        let config = NetworkConfig::new();

        assert_eq!(config.upload_limit_bytes_per_sec(), 0);
        assert_eq!(config.download_limit_bytes_per_sec(), 0);
        assert!(config.should_respect_flood_wait());
    }
}
