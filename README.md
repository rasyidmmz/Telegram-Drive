# Teledrive

**Teledrive** is a Windows 11 64-bit fork of Telegram Drive focused on using
Telegram as a personal media and file drive, with better video playback,
upload reliability, and a Windows-only release pipeline.

This fork is intentionally renamed from the upstream **Telegram Drive** app so
it can be installed on the same machine without colliding with the original
application.

## Install Identity

Teledrive uses its own Windows/Tauri identity:

- Product name: `Teledrive`
- Tauri identifier: `com.rasyidmmz.teledrive`
- Windows autostart registry value: `Teledrive`
- GitHub updater feed: `rasyidmmz/Telegram-Drive`
- Release target: Windows NSIS installer only

When autostart is toggled, Teledrive also removes the old `TelegramDrive`
startup entry so older fork builds do not start alongside the renamed app.

## What Changed From Upstream

This fork keeps the core Telegram storage model from the original project:
Saved Messages and Telegram channels are used as folders, while files are
managed from a desktop file explorer UI.

Main differences:

- Windows 11 x64 only. Android, iOS, macOS, and Linux release paths were removed
  from the active build.
- App branding and installer identity were changed to `Teledrive`.
- The release workflow builds only the Windows NSIS installer.
- MPV is bundled as a sidecar so HEVC/H.265 video playback does not depend on
  the browser video codec stack.
- Video files open through MPV using the local stream endpoint.
- `.mp4` and `.mkv` metadata probing is available for upload diagnostics and
  desktop metadata badges.
- Upload reliability was improved with queue cooldown, retry/backoff handling,
  `FLOOD_WAIT` sleep behavior, and bandwidth throttling.
- Telegram's single-file upload limit is handled with a clear error message.
  Automatic split-file upload is not in `main` yet.
- Windows startup support was added through
  `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
- Android-only JNI and foreground-service code was removed.
- Mobile-only Tauri capability configuration was removed.

## Current Video Scope

Teledrive is optimized for personal `.mp4` and `.mkv` movie files, including
HEVC/H.265 content.

Current behavior:

- Normal-size MP4/MKV files can be uploaded through the current upload flow.
- Local metadata probing logs MP4/MKV duration and resolution when available.
- Desktop file cards can show MP4/MKV duration and resolution badges.
- Playback prefers MPV through the local stream server.

Large files above Telegram's single-file limit are still blocked in `main`.
The planned split-file design lives separately in the
`codex/large-file-parts-upload` branch and is not part of the stable Windows
release yet.

## Features

- Telegram-backed file storage using Saved Messages and private channels.
- Folder creation and management.
- Drag-and-drop uploads.
- Upload and download queues.
- Retry/backoff controls for unstable connections.
- Automatic `FLOOD_WAIT` handling.
- Upload/download bandwidth throttles.
- MPV-based video/audio playback.
- MP4/MKV metadata badges.
- PDF preview.
- Shareable links with optional protection.
- Local REST API for automation and tool integration.
- SOCKS5 proxy support.
- VPN-oriented timeout, polling, keep-alive, and peer-cache settings.
- Windows autostart toggle.
- Windows-only updater artifacts.

## Download

Use the latest Windows installer from this fork's releases:

https://github.com/rasyidmmz/Telegram-Drive/releases/latest

The expected release assets are:

- `Teledrive_<version>_x64-setup.exe`
- `Teledrive_<version>_x64-setup.exe.sig`
- `latest.json`

## Build From Source

### Prerequisites

- Windows 11 64-bit
- Node.js 18+
- Rust stable
- Visual Studio Build Tools with **Desktop development with C++**
- Microsoft Edge WebView2 Runtime
- Telegram API ID and API Hash from https://my.telegram.org

### Commands

```powershell
git clone https://github.com/rasyidmmz/Telegram-Drive.git
cd Telegram-Drive\app
npm install
npm run tauri dev
```

Build the installer:

```powershell
npm run tauri build
```

## Repository Notes

- `main` is the stable Windows-only branch.
- `implementation_plan.md` tracks the current split between stable work and
  the future large-file split/upload branch.
- `codex/large-file-parts-upload` is reserved for the separate large-file parts
  feature.

## License

This fork keeps the upstream MIT license.

Teledrive is not affiliated with Telegram FZ-LLC. Use it responsibly and in
accordance with Telegram's Terms of Service.
