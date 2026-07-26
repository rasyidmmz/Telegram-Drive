# TeleStash

TeleStash is a Windows 11 64-bit desktop media vault that uses a personal
Telegram account for file storage and MPV for playback.

## What It Does

- Upload files and organize them in Telegram-backed folders.
- Play MP4 and MKV media in MPV with resume position and recent-watch history.
- Split every local or URL upload larger than `2_000_000_000` bytes into
  verified parts, regardless of file extension.
- Validate split manifests before playback, download, move, or deletion.
- Preserve transfer diagnostics with attempt, part, retry, and Telegram error
  details in the in-app log.
- Run in the Windows notification area and optionally start with Windows.
- Check, download, install, and restart into signed application updates.

## Supported Environment

TeleStash supports Windows 11 64-bit only. It connects directly to Telegram;
there are no proxy, VPN, network-optimizer, or application bandwidth settings.
Telegram `FLOOD_WAIT` responses remain mandatory protocol backoff.

## Development

The React/TypeScript frontend is in `app/src`. The Rust/Tauri backend is in
`app/src-tauri`. Build and release instructions are in
[`docs/RELEASE_RUNBOOK.md`](docs/RELEASE_RUNBOOK.md), and agent boundaries are
in [`AGENTS.md`](AGENTS.md).

## License

TeleStash is licensed under the MIT License. Use it in compliance with
Telegram's Terms of Service.
