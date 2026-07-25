# Architecture Design Spec: Teledrive Core Engine Upgrades

**Date:** 2026-07-26  
**Target Platform:** Windows 11 64-bit (Tauri + React + Rust)  
**Scope:** Backend Upload Engine, MPV Streaming Engine, Parallel Download Engine, Session Health  

---

## 1. Executive Summary

This specification outlines four major architecture upgrades to the Teledrive core engine:
1. **Resumable Upload Checkpoints & Adaptive Limiter:** Enables resuming interrupted uploads from the exact chunk index without re-uploading from 0%, while incorporating adaptive micro-delay pacing to eliminate `FLOOD_WAIT`.
2. **MPV Streaming Ring Buffer & Pre-Fetching:** Implements an in-memory ring buffer with pro-active chunk pre-fetching for instant, zero-stutter MPV video playback and fast seeking.
3. **Multi-Worker Parallel Download Engine:** Parallelizes single and split file downloads across 4 concurrent MTProto worker streams, boosting download speeds 3x–5x.
4. **Session Keep-Alive & Health Manager:** Auto-detects network reconnects / sleep-wake events to keep MTProto sessions active without user re-login.

---

## 2. Subsystem Designs

### 2.1 Resumable Upload Checkpoints (`upload_checkpoint.rs`)

#### Database Schema (SQLite)
```sql
CREATE TABLE IF NOT EXISTS upload_checkpoints (
    id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    modified_time INTEGER NOT NULL,
    telegram_file_id INTEGER NOT NULL,
    last_part_index INTEGER NOT NULL,
    total_parts INTEGER NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

#### Upload Resumption Lifecycle
- **Start / Retry:** `cmd_upload_file_inner` queries SQLite for an existing valid checkpoint matching `(file_path, size, mtime)`.
- **Match Found:** Resumes uploading from `last_part_index + 1` instead of chunk 0.
- **Progress Saving:** Checkpoint is updated in SQLite every 20 parts.
- **Completion:** Upon successful `send_message`, the checkpoint is removed from SQLite.

---

### 2.2 MPV Streaming Pre-Fetching & Ring Buffer (`streaming_buffer.rs`)

#### Architecture
- **Buffer Size:** 16 MB per active stream session.
- **Trigger:** When MPV requests byte offset $O$, the streaming handler reads from the local memory buffer if cached, and simultaneously spawns a background worker to pre-fetch offsets $O + 512\text{KB}$ through $O + 4\text{MB}$.
- **Seeking:** On large seek jumps, the buffer is invalidated and re-populated immediately from the target offset.

---

### 2.3 Multi-Worker Parallel Download Pool (`parallel_download.rs`)

#### Architecture
- **Concurrency:** 4 parallel `tokio::spawn` worker tasks per file download.
- **Chunk Allocation:**
  - File is logically divided into $N$ segments.
  - Worker $k$ fetches segment $k$ via MTProto stream.
  - Writes directly to positional offsets using `tokio::fs::File::seek` and `write_all`.
- **Speed Gain:** 3x–5x improvement over single-threaded download by maximizing multi-connection throughput to Telegram Data Centers.

---

### 2.4 Session Keep-Alive & Health Manager (`session_health.rs`)

#### Architecture
- Periodic background ping (`ping` / `get_state`) every 60 seconds.
- Detects network disconnects or system sleep-wake events on Windows 11.
- Automatically re-establishes Grammers client connections and clears stale peer cache entries without requiring user intervention or app restart.

---

## 3. Implementation Plan & Verification

- **Task Breakdown:** Tasks will be executed in 4 logical phases.
- **Verification:** Automated unit tests + `npx tsc` + `git diff --check`.
- **Git & Release Boundary:** No git commits or tag pushes will be made without explicit user permission.
