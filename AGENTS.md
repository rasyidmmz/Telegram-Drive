# Teledrive Agent Guide

This repository is Teledrive, a Windows 11 64-bit Tauri/React application for
using Telegram as a personal file drive. Treat this file as the primary
instruction set for any coding agent. Read it before editing files.

## Scope

- Support Windows 11 64-bit only. Do not add Android, Linux, macOS, or iOS
  logic, build targets, packaging, or documentation.
- The frontend is in `app/`; the Rust/Tauri backend is in `app/src-tauri/`.
- Use the existing local patterns. Keep changes narrow and avoid speculative
  abstractions or new dependencies.
- Teledrive uses direct Telegram connections only. Do not reintroduce proxy,
  VPN, bandwidth-throttling, or network-optimizer settings.

## Transfer Rules

- Files larger than `2_000_000_000` bytes must use split upload, for local and
  URL uploads, regardless of whether the video is MP4 or MKV.
- `FILE_PARTS_INVALID` is permanent for the current request; do not retry it.
- Respect Telegram `FLOOD_WAIT` values. This is protocol backoff, not a user
  bandwidth throttle.
- Keep upload retry classification in the shared retry/classifier modules so
  local, API, and URL upload paths remain consistent.

## Git And Release Boundaries

- Do not commit, push, create a tag, trigger a workflow, publish a release, or
  modify GitHub secrets without explicit user approval in the current task.
- Preserve unrelated working-tree changes. Never reset, revert, or delete them.
- A pushed `v*` tag triggers `.github/workflows/release.yml`. Before creating
  a tag, keep versions aligned in `app/package.json`, `app/package-lock.json`,
  `app/src-tauri/Cargo.toml`, `app/src-tauri/Cargo.lock`, and
  `app/src-tauri/tauri.conf.json`.
- Add a nonempty matching `## [x.y.z]` entry to `CHANGELOG.md`; the workflow
  fails before building if it is missing.
- Never call a release successful merely because the workflow started. Verify
  the build and release assets unless the user explicitly asks to stop after
  the workflow is triggered.

## Validation

- Run `npx tsc --noEmit --pretty false` from `app/` for frontend changes.
- Run `git diff --check` before committing.
- Run the most focused Rust checks available. The local GNU Rust setup may lack
  `dlltool.exe`; report that environment blocker honestly instead of claiming a
  full Rust build passed. GitHub Actions uses the Windows MSVC environment.
- Do not include `target/`, `.codex/`, temporary plans, installer artifacts,
  credentials, tokens, or signing keys in commits.

See `docs/AI_HANDOFF.md` for architecture and `docs/RELEASE_RUNBOOK.md` for
the tag-release procedure.
