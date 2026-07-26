<div align="center">

![TeleStash for Windows 11](docs/assets/telestash_hero_banner.jpg)

# TeleStash

**A Windows 11 media vault and MPV player backed by a personal Telegram account.**

[Download the latest Windows release](https://github.com/rasyidmmz/Telestash/releases/latest)

</div>

## What is TeleStash?

TeleStash is a personal Windows 11 media application that organizes files in
Telegram-backed folders and opens video through its bundled MPV integration.
It keeps a familiar media-library workflow without requiring a separate
streaming-server account, browser dashboard, proxy, or VPN setting.

The application does not impose a library-size quota of its own. Available
storage and file limits remain governed by the connected Telegram account and
Telegram's service rules.

## Why TeleStash?

| Need | TeleStash approach |
| --- | --- |
| Keep a personal media library | Use Saved Messages or private channels as Telegram-backed folders. |
| Play common personal-media formats | Hand off MP4 and MKV, including HEVC/H.265 libraries, to MPV. |
| Keep large uploads intact | Split files above `2_000_000_000` bytes and validate their manifest before use. |
| Recover from an interrupted transfer | Preserve upload state, retry decisions, and clear transfer diagnostics. |
| Keep the desktop workflow simple | Use Windows tray controls, recent-watch history, signed updates, and optional autostart. |

## System Architecture

The data path is intentionally short: the Windows application talks directly
to Telegram, MPV receives a local authenticated stream, and large files retain
their validation metadata beside their Telegram parts.

```mermaid
flowchart TB
    User[Windows 11 user] --> App[TeleStash desktop app]

    subgraph Desktop[TeleStash on Windows]
        App --> Core[Rust and Tauri core]
        Core --> Transfer[Transfer integrity\nresume state and diagnostics]
        Core --> Stream[Local authenticated stream]
        Stream --> MPV[MPV playback\nMP4 MKV HEVC/H.265]
        Core --> Captions[Whisper English CC\nlocal SRT cache]
    end

    Transfer --> Telegram[Direct Telegram connection]
    Telegram --> Library[Saved Messages and private channels]
    Transfer --> Split[Files over 2_000_000_000 bytes\nvalidated parts and manifest]
    Split --> Library
    Captions --> Library
```

## Media Library and Playback

- Browse files and folders stored in Saved Messages or private channels.
- Open compatible MP4 and MKV media in MPV through the local streaming server.
- Preserve MPV watch-later state and show a recent-watch list in the desktop
  application.
- Generate local English CC subtitles with Whisper, reuse cached SRT files,
  and upload generated subtitles to the active Telegram folder.
- Review storage analytics, transfer diagnostics, and recent playback activity
  from the desktop interface.

## Transfer Integrity

- Upload local files or URL sources through direct Telegram connections.
- Split every file larger than `2_000_000_000` bytes into 512 MiB parts,
  regardless of extension, then store a validated manifest for reconstruction.
- Validate split parts and total size before streaming or downloading a large
  file, so a missing or malformed part is reported rather than played as a
  damaged video.
- Keep resumable upload state and transfer diagnostics, including attempts,
  part indexes, retry decisions, and Telegram error classes.
- Respect Telegram `FLOOD_WAIT` values as protocol backoff. `FILE_PARTS_INVALID`
  is treated as a permanent error for the current request.

## Windows Integration

- Run from the Windows notification area with show and exit controls.
- Optionally start with Windows for a personal desktop library that stays ready.
- Check for signed updates, download and install them, then restart TeleStash.

## Install

1. Download the current `TeleStash_*_x64-setup.exe` from the
   [latest release](https://github.com/rasyidmmz/Telestash/releases/latest).
2. Run the installer on Windows 11 64-bit.
3. Sign in with the Telegram account that owns the media library.

Updates are delivered from the same release channel through a signed updater
manifest. The installer uses a per-user Windows installation, so a newer
release replaces the existing application without requiring an uninstall.

## Supported Environment

TeleStash supports **Windows 11 64-bit only**. It uses Telegram directly; it
does not provide proxy, VPN, network-optimizer, or manual bandwidth-throttle
settings. Use of Telegram storage and media content remains subject to
Telegram's Terms of Service and applicable law.

## Development

The React and TypeScript frontend is in `app/src`; the Rust and Tauri backend
is in `app/src-tauri`.

```powershell
cd app
npm ci
npm run dev
```

For a production Windows installer, use the release procedure in
[docs/RELEASE_RUNBOOK.md](docs/RELEASE_RUNBOOK.md). Agent boundaries and
release safeguards are defined in [AGENTS.md](AGENTS.md).

## Project Identity

TeleStash is an independent project and is not affiliated with, endorsed by,
or sponsored by Telegram. This repository does not currently publish a license
file; do not assume reuse rights beyond the terms explicitly provided here.
