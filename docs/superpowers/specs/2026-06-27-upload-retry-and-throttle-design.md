# Design Specification: Upload Retry, Auto-Throttling, Queue Cooldown, and Settings Persistence Fix

This document specifies the design for addressing upload failures due to Telegram rate limits (`FLOOD_PREMIUM_WAIT`), implementing automatic throttling for large files, and fixing the settings persistence issue where custom settings (like concurrent upload limit) are not saved after restarting the application.

## 1. Problem Identification & Context
* **FLOOD_PREMIUM_WAIT Error:** When uploading bulk files or large files, Telegram rate-limits part uploads (`upload.saveBigFilePart`). Currently, `client.upload_stream` does not have retry logic, causing the upload to fail immediately. Furthermore, `map_error` in `commands/utils.rs` only checks for the literal `"FLOOD_WAIT"`, ignoring `"FLOOD_PREMIUM_WAIT"`.
* **Auto-Throttling Requirement:** Uploading files larger than 2 GB frequently triggers flood limits. We need to auto-throttle uploads to 5 MB/s if the file is greater than 2 GB, while keeping it at full speed (no limit) for files smaller than 2 GB (unless the user has configured a custom lower limit).
* **Settings Not Persisting:** In `SettingsContext.tsx`, `persistSettings` is called inside the `setSettings` state updater. This is a React anti-pattern that causes the asynchronous disk write (`store.save()`) to be cancelled or bypassed, leading to settings being lost on application restart.

---

## 2. Proposed Changes

### 2.1 Backend: Error Parsing & Throttling
#### [MODIFY] [utils.rs](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src-tauri/src/commands/utils.rs)
* Update `map_error` to catch both `"FLOOD_WAIT"` and `"FLOOD_PREMIUM_WAIT"`.
* Parse the wait time `(value: X)` and format it as `"FLOOD_WAIT_X"` so it is handled uniformly by the sleep retry logic.

#### [MODIFY] [fs.rs](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src-tauri/src/commands/fs.rs)
* **Modify `ProgressReader`**:
  * Make `ProgressReader` and its `new` function public (`pub(crate)`) so `api_routes.rs` can reuse it.
  * Add speed limiting logic directly inside its `AsyncRead::poll_read` implementation using an async timer (`tokio::time::sleep` registration in a waker).
  * Accept a `limit` (bytes/sec) parameter in `ProgressReader::new`.
* **Retry Loop in `cmd_upload_file_inner` & `cmd_upload_from_url_inner`**:
  * Re-create the `ProgressReader` on retry so it starts reading from byte 0.
  * Calculate speed limit: if `file_size > 2GB`, set `limit = 5 MB/s` (or minimum of 5 MB/s and user's configured limit if set). Otherwise, use the user's configured limit (defaulting to unlimited/full-speed).
  * Wrap `client_clone.upload_stream` in a retry loop. If it fails with an error starting with `"FLOOD_WAIT_"`, extract the duration, sleep, and retry.

#### [MODIFY] [api_routes.rs](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src-tauri/src/api_routes.rs)
* Import `ProgressReader` from `crate::commands::fs`.
* Wrap the web upload stream in `ProgressReader` to apply the same auto-throttling logic and retry loop.

---

### 2.2 Frontend: Queue Cooldown & Settings Persistence
#### [MODIFY] [useFileUpload.ts](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src/hooks/useFileUpload.ts)
* In `processItem`, introduce a 2-second cooldown delay in the `finally` block before decrementing `activeCountRef.current` and triggering the next queue item.

#### [MODIFY] [SettingsContext.tsx](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src/context/SettingsContext.tsx)
* Remove the `persistSettings` call from the `setSettings` state updater in `updateSetting`.
* Introduce a `useEffect` that monitors changes to the `settings` state and triggers `persistSettings` safely outside the rendering cycle once `isLoaded` is true.

---

## 3. Verification Plan
### Manual Verification
1. Set concurrent upload max to `1` in settings, restart the app, and verify it remains `1`.
2. Upload multiple small files to verify the 2-second queue cooldown is respected.
3. Upload a file $> 2$ GB (if a Premium account is available) and verify the speed is capped at 5 MB/s.
4. Upload a file $< 2$ GB and verify it uploads at full speed.
5. Simulate/trigger a `FLOOD_PREMIUM_WAIT` error and verify that the backend sleeps for the required time and successfully retries.
