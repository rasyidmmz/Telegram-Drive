# TeleStash Core Engine Upgrades Todo

- [ ] Task 1.1: Create SQLite table `upload_checkpoints` and helper functions in `upload_checkpoint.rs`
- [ ] Task 1.2: Integrate upload resumption into `fs.rs` (`cmd_upload_file_inner` and `upload_large_file_split`)
- [ ] Task 1.3: Add adaptive micro-delay rate limiter
- [ ] Task 2.1: Build `RingBuffer` struct with 16MB capacity and forward pre-fetch logic in `streaming_buffer.rs`
- [ ] Task 2.2: Connect `RingBuffer` to Actix Web streaming handler in `api_routes.rs`
- [ ] Task 3.1: Implement 4-worker Tokio async download pool in `parallel_download.rs`
- [ ] Task 3.2: Connect parallel download engine to file download commands in `fs.rs`
- [ ] Task 4.1: Implement 60s ping loop and automatic peer cache healing in `session_health.rs`
