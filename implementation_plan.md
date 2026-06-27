# Implementation Plan - Telegram Drive Next Update

This plan separates the next work into two lanes:

1. `main`: keep the Windows 11 x64 app stable and apply only small upload reliability improvements.
2. `codex/large-file-parts-upload`: build the large-file split/upload/streaming feature separately.

## User Review Required

> [!IMPORTANT]
> * **Main branch:** no split-file architecture here. Keep the current upload queue, cooldown, flood-wait handling, metadata probing, and clear error messages.
> * **Large file branch:** files above Telegram's single-file limit will be split into ordered byte parts, uploaded separately, and streamed back to MPV through a local virtual file endpoint.
> * **Do not inject Telegram video attributes in `main` yet:** the previous low-level media payload approach was risky with the current `grammers` upload path. Keep metadata extraction for display/streaming support unless the split branch proves a safe payload path.

---

## Main Branch Scope

### 1. Already Applied Platform Cleanups & Windows Autostart

#### [DELETE] [jni_cache.rs](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src-tauri/src/jni_cache.rs)
* Remove the file completely since JNI caching is Android-only.

#### [DELETE] [upload_service.rs](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src-tauri/src/upload_service.rs)
* Remove the file completely since foreground service is Android-only.

#### [MODIFY] [main.rs](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src-tauri/src/main.rs)
* Remove all `#[cfg(target_os = "linux")]` directives and the block that sets `WEBKIT_DISABLE_DMABUF_RENDERER`.
* Simplify `main` to directly run `app_lib::run()`.

#### [MODIFY] [lib.rs](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src-tauri/src/lib.rs)
* Remove `pub mod jni_cache;` and `pub mod upload_service;` declarations.
* Remove Android setup JNI wrapper inside `.setup(...)`.
* Remove `cmd_start_foreground_service` and `cmd_stop_foreground_service` from the command handler registration.
* Register `commands::settings::cmd_set_autostart` under the command handler.

#### [MODIFY] [settings.rs](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src-tauri/src/commands/settings.rs)
* Implement `cmd_set_autostart(enabled: bool) -> Result<String, String>`:
  * On Windows, run `reg.exe` to add or remove `TelegramDrive` in `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
  * Use the running executable's path via `std::env::current_exe()`.

#### [MODIFY] [tauri.conf.json](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src-tauri/tauri.conf.json)
* Remove `"mobile-only"` from security capabilities since we do not target mobile.

---

### 2. Video Upload & Streaming Optimizations

#### [MODIFY] [video_metadata.rs](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src-tauri/src/commands/video_metadata.rs)
* Keep `ParsedMetadata` and `parse_mp4_metadata` available for local metadata reads.
* Keep `.mkv` metadata parsing through the Matroska parser in `mp4_utils`.

#### [MODIFY] [mp4_utils.rs](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src-tauri/src/mp4_utils.rs)
* Keep Matroska EBML parser functions `parse_mkv_metadata`, `read_vint`, and `read_ebml_id`.

#### [MODIFY] [fs.rs](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src-tauri/src/commands/fs.rs)
* Keep upload queue retry and `FLOOD_WAIT` sleep behavior.
* Keep local `.mp4`/`.mkv` metadata probing before upload for diagnostics and later streaming decisions.
* Keep upload throttling as a flood-risk reduction, not as a fix for Telegram's hard file-size limit.
* Do not add split upload or virtual reassembly in `main`.

---

### 3. Frontend Settings UI & Translations

#### [MODIFY] [SettingsContext.tsx](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src/context/SettingsContext.tsx)
* Replace `linuxRenderingFix` with `windowsAutostart: boolean` (default: `false`).

#### [MODIFY] [SettingsModal.tsx](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src/components/desktop/dashboard/SettingsModal.tsx)
* Replace the "Linux Rendering Fix" toggle with a "Launch on Windows Startup" toggle.
* Call `invoke('cmd_set_autostart', { enabled: newValue })` when toggled.

#### [MODIFY] [en.json](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src/i18n/locales/en.json) & [id.json](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src/i18n/locales/id.json)
* Replace `linux_rendering_fix` and `linux_rendering_desc` with `windows_autostart` and `windows_autostart_desc` translations.

---

## Separate Branch Scope: `codex/large-file-parts-upload`

### Goal

Support `.mp4` and `.mkv` HEVC/H.265 files larger than Telegram's single-file upload limit without changing the user's original file.

### Design

* Split large files byte-perfect into ordered parts below the Telegram limit.
* Upload each part as a normal Telegram document.
* Store a manifest containing original filename, total size, part size, ordered message IDs, and optional checksum.
* For MPV playback, expose one local endpoint that behaves like the original complete file.
* Implement HTTP Range support in that endpoint so MPV can seek across part boundaries.
* Do not ask MPV to open raw part files directly.

### Not In Scope For `main`

* No chunk manifest table in `main`.
* No virtual reassembly stream endpoint in `main`.
* No automatic large-file splitting in `main`.

---

## Verification Plan

### Automated Verification
* Build the frontend with `npm run build` to verify React/TypeScript changes compile cleanly.

### Manual Verification
1. Enable "Launch on Windows Startup" and verify the registry entry in Registry Editor.
2. Disable it and verify the registry entry is deleted.
3. Upload normal-size MP4 and MKV HEVC/H.265 files and verify metadata display/playback.
4. On `codex/large-file-parts-upload`, upload a file above the Telegram single-file limit and verify MPV playback through the virtual stream endpoint.
