# AI Agent Handoff

## Product Boundary

Teledrive is a personal Telegram drive for Windows 11 64-bit. It is a fork
with its own Teledrive identity, Windows installer, updater, logs, and MPV
streaming integration. It is not a cross-platform product.

The frontend is React/TypeScript in `app/src`. The desktop backend is Rust and
Tauri in `app/src-tauri/src`. GitHub Actions builds the Windows NSIS installer.

## Transfer Behavior

Normal transfers use direct Telegram connections without an application-side
rate cap. Telegram protocol limits still apply: `FLOOD_WAIT` must be honoured,
and retry policy belongs in the shared retry/classifier layer.

The safe single-file Telegram cutoff used by Teledrive is `2_000_000_000`
bytes. A larger local or remote URL upload is stored as parts with a manifest.
This applies to MP4, MKV, and all other file types. `FILE_PARTS_INVALID` is not
a transient network error and must not enter the normal retry loop.

Split-file changes affect listing, manifest validation, streaming, downloads,
move/delete, API routes, and retry cleanup. Preserve manifest integrity before
claiming that a split file is playable in MPV.

## Current Operating Decisions

- Windows-only: no mobile or non-Windows platform paths.
- Direct network connections: no proxy, VPN, local SOCKS bridge, or optimizer
  settings.
- No application-side upload/download bandwidth throttle. This does not mean
  ignoring Telegram flood waits or transient connection failures.
- User-facing errors are recorded in the in-app logs. Preserve diagnostics such
  as transfer id, attempt, error kind, retry decision, and part index.
- Keep the fork name `Teledrive`, including installer and autostart identity.

## Working Method

Inspect the current branch, status, source, versions, and workflow before
assuming prior work is deployed. Preserve uncommitted work that you did not
create. Fix the shared root cause rather than adding per-screen workarounds.

Use `AGENTS.md` for mandatory boundaries and `RELEASE_RUNBOOK.md` for release
operations. A status note, prior plan, or test artifact is not proof that a
feature was committed, pushed, released, or validated at runtime.
