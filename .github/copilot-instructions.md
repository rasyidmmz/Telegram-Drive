# Teledrive Instructions

Follow the repository root `AGENTS.md` as the source of truth. In particular:

- Work only on Windows 11 64-bit support.
- Keep changes in the existing React/Tauri structure under `app/` and
  `app/src-tauri/`.
- Do not reintroduce proxy, VPN, or application-level transfer throttling.
- Route files above `2_000_000_000` bytes through split upload in every upload
  entrypoint; respect Telegram `FLOOD_WAIT`; do not retry `FILE_PARTS_INVALID`.
- Do not push to `main`, create tags, publish releases, or access secrets unless
  explicitly instructed. Prefer a branch and pull request for autonomous work.
- Match versions and `CHANGELOG.md` before any `v*` release tag.
- Validate with TypeScript and focused Rust checks, and state any Windows GNU
  toolchain blocker such as a missing `dlltool.exe`.
