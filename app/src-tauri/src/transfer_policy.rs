//! Fixed policy for direct Telegram transfers.

pub struct TransferPolicy;

impl TransferPolicy {
    pub fn new() -> Self {
        Self
    }

    pub fn retry_attempts(&self) -> u32 {
        0
    }

    pub fn retry_base_backoff_ms(&self) -> u64 {
        1_000
    }

    pub fn retry_max_backoff_ms(&self) -> u64 {
        30_000
    }

    pub fn should_respect_flood_wait(&self) -> bool {
        true
    }

    pub fn upload_limit_bytes_per_sec(&self) -> u64 {
        0
    }

    pub fn download_limit_bytes_per_sec(&self) -> u64 {
        0
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
    fn direct_transfers_are_unthrottled_and_respect_flood_waits() {
        let policy = TransferPolicy::new();

        assert_eq!(policy.upload_limit_bytes_per_sec(), 0);
        assert_eq!(policy.download_limit_bytes_per_sec(), 0);
        assert!(policy.should_respect_flood_wait());
    }
}
