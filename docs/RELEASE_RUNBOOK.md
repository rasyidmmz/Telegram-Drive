# Windows Release Runbook

## Preconditions

1. Obtain explicit approval to commit, push, tag, and release.
2. Confirm `main` contains the intended changes and inspect `git status` for
   unrelated files. Stage only approved files.
3. Choose the next semantic version. Keep it identical in:
   - `app/package.json`
   - `app/package-lock.json`
   - `app/src-tauri/Cargo.toml`
   - `app/src-tauri/Cargo.lock`
   - `app/src-tauri/tauri.conf.json`
4. Add a nonempty `## [x.y.z]` section to `CHANGELOG.md`.
5. Update `README.md` if the release contains major feature changes, architectural overhauls, or significant user capability impacts.
6. Run `npx tsc --noEmit --pretty false` from `app/` and `git diff --check`.
   Run focused Rust tests/checks where the local toolchain permits. Report a
   missing GNU `dlltool.exe` as a local environment limitation.

## Publish

1. Commit the approved source, documentation, version, and changelog changes.
2. Push `main` to `origin`.
3. Create and push the matching annotated tag: `vX.Y.Z`.
4. The release workflow creates a draft GitHub release, builds the Windows NSIS
   installer, uploads its signed updater artifacts, then publishes the release.

## GitHub Actions Requirements

The workflow is tag-driven by `v*`. Its changelog extraction requires a header
matching the tag version exactly. It builds on `windows-latest` with the MSVC
Rust toolchain, downloads the MPV Windows sidecar, prepares Whisper resources,
and verifies updater signing before publishing.

Required repository secrets include the Tauri signing private key and its
password. Do not expose, print, copy into source, or ask an autonomous agent to
persist these values.

## Verification

Confirm the `create-release`, `build-tauri`, and `publish-release` jobs succeed
and that the release contains the expected Windows installer and updater
artifacts. If the user explicitly asks not to monitor the workflow after it is
started, report only that the tag was accepted by GitHub Actions; do not infer
publication.
