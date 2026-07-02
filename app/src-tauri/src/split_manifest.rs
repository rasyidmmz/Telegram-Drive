use crate::models::{SplitManifest, SPLIT_MANIFEST_VERSION};
use std::collections::HashSet;

pub(crate) fn validate_split_manifest(manifest: &SplitManifest) -> Result<(), String> {
    if manifest.teledrive_split != SPLIT_MANIFEST_VERSION {
        return Err(format!("Unsupported split manifest version {}", manifest.teledrive_split));
    }
    if manifest.filename.trim().is_empty() {
        return Err("Split manifest filename is empty".to_string());
    }
    if manifest.size == 0 {
        return Err("Split manifest size is zero".to_string());
    }
    if manifest.part_size == 0 {
        return Err("Split manifest part_size is zero".to_string());
    }
    if manifest.parts.is_empty() {
        return Err("Split manifest has no parts".to_string());
    }

    let expected_count = expected_part_count(manifest.size, manifest.part_size)?;
    if manifest.parts.len() != expected_count {
        return Err(format!(
            "Split manifest expected {} parts but contains {}",
            expected_count,
            manifest.parts.len()
        ));
    }

    let mut seen = HashSet::new();
    let mut total = 0u64;
    for (index, part) in manifest.parts.iter().enumerate() {
        let part_number = index + 1;
        if part.message_id <= 0 {
            return Err(format!("Split manifest part {} has invalid message_id {}", part_number, part.message_id));
        }
        if !seen.insert(part.message_id) {
            return Err(format!("Split manifest has duplicate message_id {}", part.message_id));
        }
        if part.size == 0 {
            return Err(format!("Split manifest part {} has zero size", part_number));
        }
        let is_last = index == manifest.parts.len() - 1;
        if !is_last && part.size != manifest.part_size {
            return Err(format!(
                "Split manifest part {} size mismatch: expected {}, got {}",
                part_number, manifest.part_size, part.size
            ));
        }
        if is_last && part.size > manifest.part_size {
            return Err(format!(
                "Split manifest last part size {} exceeds part_size {}",
                part.size, manifest.part_size
            ));
        }
        total = total
            .checked_add(part.size)
            .ok_or_else(|| "Split manifest total size overflow".to_string())?;
    }

    if total != manifest.size {
        return Err(format!(
            "Split manifest total size mismatch: expected {}, got {}",
            manifest.size, total
        ));
    }

    Ok(())
}

pub(crate) fn expected_part_count(size: u64, part_size: u64) -> Result<usize, String> {
    if size == 0 {
        return Err("Split manifest size is zero".to_string());
    }
    if part_size == 0 {
        return Err("Split manifest part_size is zero".to_string());
    }
    usize::try_from(((size - 1) / part_size) + 1)
        .map_err(|_| "Split manifest part count exceeds platform limit".to_string())
}

#[cfg(test)]
mod tests {
    use crate::models::{SplitManifest, SplitPart, SPLIT_MANIFEST_VERSION};

    #[test]
    fn accepts_valid_manifest() {
        let manifest = SplitManifest {
            teledrive_split: SPLIT_MANIFEST_VERSION,
            filename: "movie.mkv".to_string(),
            size: 5,
            mime_type: "video/x-matroska".to_string(),
            file_ext: Some("mkv".to_string()),
            part_size: 2,
            parts: vec![
                SplitPart { message_id: 10, size: 2 },
                SplitPart { message_id: 11, size: 2 },
                SplitPart { message_id: 12, size: 1 },
            ],
        };

        assert!(super::validate_split_manifest(&manifest).is_ok());
    }

    #[test]
    fn rejects_wrong_part_count() {
        let manifest = SplitManifest {
            teledrive_split: SPLIT_MANIFEST_VERSION,
            filename: "movie.mp4".to_string(),
            size: 5,
            mime_type: "video/mp4".to_string(),
            file_ext: Some("mp4".to_string()),
            part_size: 2,
            parts: vec![
                SplitPart { message_id: 10, size: 2 },
                SplitPart { message_id: 11, size: 3 },
            ],
        };

        assert!(super::validate_split_manifest(&manifest)
            .unwrap_err()
            .contains("expected 3 parts"));
    }

    #[test]
    fn rejects_duplicate_message_ids() {
        let manifest = SplitManifest {
            teledrive_split: SPLIT_MANIFEST_VERSION,
            filename: "movie.mp4".to_string(),
            size: 4,
            mime_type: "video/mp4".to_string(),
            file_ext: Some("mp4".to_string()),
            part_size: 2,
            parts: vec![
                SplitPart { message_id: 10, size: 2 },
                SplitPart { message_id: 10, size: 2 },
            ],
        };

        assert!(super::validate_split_manifest(&manifest)
            .unwrap_err()
            .contains("duplicate"));
    }
}
