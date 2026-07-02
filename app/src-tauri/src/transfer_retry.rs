const TRANSIENT_UPLOAD_RETRY_ATTEMPTS: u32 = 2;

pub(crate) fn upload_stream_retry_attempts(configured_attempts: u32) -> u32 {
    configured_attempts.max(TRANSIENT_UPLOAD_RETRY_ATTEMPTS)
}

pub(crate) fn should_retry_upload_error(
    err: &str,
    attempt: u32,
    configured_attempts: u32,
) -> bool {
    if attempt < configured_attempts {
        return true;
    }

    is_transient_upload_error(err) && attempt < upload_stream_retry_attempts(configured_attempts)
}

pub(crate) fn upload_error_kind(err: &str) -> &'static str {
    if err.starts_with("FLOOD_WAIT_") {
        "telegram flood wait"
    } else if is_transient_upload_error(err) {
        "transient network/Telegram read error"
    } else {
        "non-retryable upload error"
    }
}

fn is_transient_upload_error(err: &str) -> bool {
    let err = err.to_ascii_lowercase();
    [
        "read 0 bytes",
        "reached eof before",
        "eof before reaching",
        "unexpected eof",
        "early eof",
        "connection reset",
        "connection aborted",
        "connection closed",
        "connection lost",
        "broken pipe",
        "timed out",
        "timeout",
        "temporarily unavailable",
    ]
    .iter()
    .any(|needle| err.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transient_read_errors_get_two_internal_retries_when_user_retry_is_disabled() {
        let err = "request error: read 0 bytes";

        assert!(should_retry_upload_error(err, 0, 0));
        assert!(should_retry_upload_error(err, 1, 0));
        assert!(!should_retry_upload_error(err, 2, 0));
    }

    #[test]
    fn permanent_upload_errors_do_not_get_internal_retries() {
        let err = "rpc error 400: FILE_PARTS_INVALID caused by upload.saveBigFilePart";

        assert!(!should_retry_upload_error(err, 0, 0));
    }

    #[test]
    fn user_retry_budget_still_applies_before_transient_only_retry() {
        let err = "rpc error 400: FILE_PARTS_INVALID caused by upload.saveBigFilePart";

        assert!(should_retry_upload_error(err, 0, 1));
        assert!(!should_retry_upload_error(err, 1, 1));
    }
}
