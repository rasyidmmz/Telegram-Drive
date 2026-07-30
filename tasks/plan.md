# Implementation Plan: TeleStash Core Engine Upgrades

## Overview
Comprehensive upgrade across Upload, Streaming (MPV), Download, and Session Health to deliver resumable uploads, zero-stutter MPV pre-fetching, 3x-5x faster parallel downloads, and automatic session healing.

## Subsystem Phases

### Phase 1: Resumable Upload Checkpoints (`upload_checkpoint.rs`)
- [ ] **Task 1.1:** Create SQLite table `upload_checkpoints` and CRUD helper functions in `upload_checkpoint.rs`.
- [ ] **Task 1.2:** Integrate checkpoint lookup and resumption into `cmd_upload_file_inner` and `upload_large_file_split` in `fs.rs`.
- [ ] **Task 1.3:** Add adaptive micro-delay rate limiter to prevent `FLOOD_WAIT`.

### Phase 2: MPV Streaming Pre-Fetching & Ring Buffer (`streaming_buffer.rs`)
- [ ] **Task 2.1:** Build `RingBuffer` struct with 16MB capacity and forward pre-fetch logic in `streaming_buffer.rs`.
- [ ] **Task 2.2:** Connect `RingBuffer` into Actix Web video streaming handler in `api_routes.rs`.

### Phase 3: Multi-Worker Parallel Download Pool (`parallel_download.rs`)
- [ ] **Task 3.1:** Implement 4-worker Tokio async download pool in `parallel_download.rs`.
- [ ] **Task 3.2:** Connect parallel download engine to single and split file download functions in `fs.rs`.

### Phase 4: Session Health Manager (`session_health.rs`)
- [ ] **Task 4.1:** Implement 60s ping loop and automatic peer cache refresh upon sleep-wake events in `session_health.rs`.

## Verification Gates
- [ ] `npx tsc --noEmit --pretty false`
- [ ] `git diff --check`
- [ ] Automated unit tests for resumption & ring-buffer
