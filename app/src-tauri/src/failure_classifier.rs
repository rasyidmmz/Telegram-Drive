pub(crate) fn classify_failure(message: &str, details: Option<&str>) -> &'static str {
    let text = format!("{}\n{}", message, details.unwrap_or_default()).to_ascii_lowercase();

    if text.contains("split")
        || text.contains("manifest")
        || text.contains("part missing")
        || text.contains("size mismatch")
    {
        "manifest/split"
    } else if text.contains("failed to open")
        || text.contains("access is denied")
        || text.contains("no such file")
        || text.contains("cannot find the file")
    {
        "local file"
    } else if text.contains("flood_wait")
        || text.contains("rpc error")
        || text.contains("telegram")
        || text.contains("upload.save")
    {
        "Telegram/server"
    } else if text.contains("read 0 bytes")
        || text.contains("reached eof before")
        || text.contains("unexpected eof")
        || text.contains("connection reset")
        || text.contains("connection aborted")
        || text.contains("forcibly closed")
        || text.contains("os error 10054")
        || text.contains("broken pipe")
        || text.contains("timed out")
        || text.contains("timeout")
    {
        "network/proxy"
    } else {
        "unknown"
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn classifies_common_upload_failures() {
        assert_eq!(
            super::classify_failure("Upload failed: request error: read 0 bytes", None),
            "network/proxy"
        );
        assert_eq!(
            super::classify_failure(
                "Upload failed",
                Some("request error: An existing connection was forcibly closed by the remote host. (os error 10054)")
            ),
            "network/proxy"
        );
        assert_eq!(
            super::classify_failure("FLOOD_WAIT_30", Some("Telegram limited this request")),
            "Telegram/server"
        );
        assert_eq!(
            super::classify_failure("Split part size mismatch", None),
            "manifest/split"
        );
        assert_eq!(
            super::classify_failure("Failed to open large file for splitting", None),
            "local file"
        );
    }
}
