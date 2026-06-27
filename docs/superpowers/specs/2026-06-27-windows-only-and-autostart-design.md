# Design Specification: Windows-Only Optimization and Autostart Feature

This document specifies the design for cleaning up the codebase to target Windows 64-bit exclusively, removing unused stubs for other platforms (Android/Linux), and replacing the defunct Linux rendering setting with a native Windows "Launch on Startup" (autostart) configuration.

## 1. Problem Identification & Context
* **Cross-Platform Stubs:** The codebase currently carries legacy stubs for Android JNI caching, foreground services, and Linux rendering fixes. Since the target is strictly Windows 64-bit, this dead code can be deleted to simplify maintenance.
* **Windows Autostart Feature:** Users running the app on Windows expect an option to start the application automatically when the system boots. We can implement this natively using Windows registry commands (`reg.exe`) via `std::process::Command` without adding any third-party crate dependencies.

---

## 2. Proposed Changes

### 2.1 Backend Cleanup & Autostart (Rust)
#### [DELETE] [jni_cache.rs](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src-tauri/src/jni_cache.rs)
* Delete this file entirely as JNI caching is only used for Android FFI.

#### [DELETE] [upload_service.rs](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src-tauri/src/upload_service.rs)
* Delete this file entirely as foreground services are only used for Android persistent notifications.

#### [MODIFY] [main.rs](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src-tauri/src/main.rs)
* Delete all `#[cfg(target_os = "linux")]` sections.
* Simplify `main` to directly invoke `app_lib::run()`.

#### [MODIFY] [lib.rs](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src-tauri/src/lib.rs)
* Remove `pub mod jni_cache;` and `pub mod upload_service;`.
* Remove the Android JNI initialization logic block (`#[cfg(target_os = "android")]` block inside `setup`).
* Remove `upload_service::cmd_start_foreground_service` and `upload_service::cmd_stop_foreground_service` from the Tauri `generate_handler!` registration.
* Register `commands::settings::cmd_set_autostart` under `generate_handler!`.

#### [MODIFY] [settings.rs](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src-tauri/src/commands/settings.rs)
* Implement a new Tauri command `cmd_set_autostart(enabled: bool) -> Result<String, String>`.
* In `cmd_set_autostart` (gated with `#[cfg(target_os = "windows")]`), spawn `reg.exe` commands to add/remove the application path from the Current User Run registry: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.

---

### 2.2 Frontend UI & Localization (React / TypeScript)
#### [MODIFY] [SettingsContext.tsx](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src/context/SettingsContext.tsx)
* Replace the `linuxRenderingFix: boolean` setting field with `windowsAutostart: boolean` (default: `false`).

#### [MODIFY] [SettingsModal.tsx](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src/components/desktop/dashboard/SettingsModal.tsx)
* Replace the "Linux Rendering Fix" toggle section with "Launch on Windows Startup".
* When toggled, update `windowsAutostart` and call `invoke('cmd_set_autostart', { enabled: newValue })`.

#### [MODIFY] [en.json](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src/i18n/locales/en.json) & [id.json](file:///c:/repo-rasyidmmz/Telegram-Drive/app/src/i18n/locales/id.json)
* Replace the `linux_rendering_fix` and `linux_rendering_desc` translation keys with:
  * English:
    * `"windows_autostart": "Launch on Windows Startup"`
    * `"windows_autostart_desc": "Start Telegram Drive automatically when Windows boots"`
  * Indonesian:
    * `"windows_autostart": "Jalankan saat Windows Dimulai"`
    * `"windows_autostart_desc": "Mulai Telegram Drive secara otomatis saat Windows boot"`

---

## 3. Verification Plan
### Manual Verification
1. Open Settings, toggle "Launch on Windows Startup" to enabled, and verify that the registry key `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\TelegramDrive` points to the executable path.
2. Toggle "Launch on Windows Startup" to disabled, and verify that the registry key is deleted.
3. Verify that settings persistence still works (restarting the app preserves the state of the toggle).
