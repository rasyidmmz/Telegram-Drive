# Windows Release Runbook

## Before A Release

1. Obtain explicit approval to commit, push, tag, and publish.
2. Keep the version identical in `app/package.json`, `app/package-lock.json`,
   `app/src-tauri/Cargo.toml`, `app/src-tauri/Cargo.lock`, and
   `app/src-tauri/tauri.conf.json`.
3. Add a nonempty exact `## [x.y.z]` entry to `CHANGELOG.md`.
4. Update `README.md` for material user-facing changes.
5. Run frontend checks, focused Rust checks where the local toolchain permits,
   and `git diff --check`.

## Publishing

1. Commit and push only the approved files.
2. Create and push the matching annotated tag `vX.Y.Z`.
3. The GitHub workflow validates the tag, versions, and changelog before the
   build. It uses the Windows MSVC Rust toolchain, creates the NSIS installer,
   signs the updater bundle, and checks every required asset.
4. Only after the verified assets exist does the workflow create or update the
   GitHub release, replace duplicate assets on a rerun, and publish it.

## Required Secret

`TAURI_SIGNING_PRIVATE_KEY` and its password must be configured in GitHub.
The workflow fails before a release if the signing key is absent or does not
match the configured updater public key. Never add key material to source or
logs.
