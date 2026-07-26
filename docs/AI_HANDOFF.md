# TeleStash AI Handoff

## Product Boundary

TeleStash is a Windows 11 64-bit Tauri/React desktop application. The frontend
is in `app/src`; the backend is in `app/src-tauri/src`. Do not add non-Windows
targets, packages, documentation, or CI jobs.

## Transfer Contract

- Use direct Telegram connections only. Do not add proxy, VPN, SOCKS, network
  optimizer, or application bandwidth-throttle features.
- Files over `2_000_000_000` bytes use split upload for every file type and
  source, including local files and URLs.
- Preserve split manifest validation across listing, streaming, download,
  move, delete, and retry paths.
- `FILE_PARTS_INVALID` is permanent for the current request. Respect Telegram
  `FLOOD_WAIT` values through shared retry and classifier modules.
- Transfer logs must retain transfer id, attempt, part index, error class, and
  retry decision.

## Working Method

Read `AGENTS.md`, inspect the branch and working tree, then change the shared
root cause. Preserve unrelated work. Do not claim a build or release succeeded
without the relevant evidence.

Commit, push, tag creation, workflow triggering, release publication, and
secret changes each require direct user approval in the current task.
