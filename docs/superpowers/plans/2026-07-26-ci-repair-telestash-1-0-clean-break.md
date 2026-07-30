# CI Repair And TeleStash 1.0 Clean Break Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Produce a Windows 11 x64-only TeleStash v1.0.0 whose source, installer, updater, UI, and release workflow have no legacy-product dependency or known release blocker.

**Architecture:** The work is ordered as release gates rather than one large rebrand commit. First make the current source compile deterministically and make GitHub Actions reject an invalid tag before a release draft exists. Then correct the high-impact desktop UX defects, remove deprecated network-control code, and complete the clean-break identity/provenance work. The public repository reset and v1.0 release happen only after the local gates are green and the owner completes the provenance gate.

**Tech Stack:** React 19, TypeScript 5.8, Tailwind 4, Tauri 2, Rust, Grammers, GitHub Actions, NSIS.

## Local Implementation Status

Completed in the working tree: source cleanup, version reset, direct-transfer
policy cleanup, UI fixes, release-workflow gates, and public documentation.
Pending by design: local/runtime validation, updater-key replacement, provenance
review, commit, push, tag, remote release, and installer verification.

## Global Constraints

- Support Windows 11 x64 only; do not add non-Windows targets or packaging.
- Use direct Telegram connections only. Do not retain proxy, VPN, application bandwidth-throttle, or network-optimizer controls.
- Preserve split upload for every file type above `2_000_000_000` bytes; a clean break means old app manifests and local state are intentionally unsupported.
- Do not commit, push, tag, trigger a workflow, alter secrets, delete the remote repository, or create a replacement remote repository without explicit user approval in that task.
- Do not overwrite the untracked `PLAN.md`; it is a separate English CC plan.
- A release tag is valid only when `app/package.json`, `app/package-lock.json`, `app/src-tauri/Cargo.toml`, `app/src-tauri/Cargo.lock`, `app/src-tauri/tauri.conf.json`, and the new changelog entry all state the same `1.0.0` version.

## Audit Status Before Implementation

| Gate | Status | Evidence | Required outcome |
| --- | --- | --- | --- |
| Frontend type check | Pass | `npx tsc --noEmit --pretty false` passed on 2026-07-26 | Keep green after every frontend task. |
| Updater key unit tests | Pass | `npm run test:updater-signing-key` passed: 4 tests | Keep green; do not reuse legacy updater identity. |
| Local Rust check | Blocked by environment | GNU toolchain stops at missing `dlltool.exe` before application code compiles | Use GitHub Windows MSVC validation as the source-compilation gate. |
| CI source compilation | Fail | `tauri::tray` is used without the `tray-icon` feature; two unused parallel modules call unavailable Grammers APIs | Fix before any release tag. |
| Release ordering | Fail | `create-release` runs before frontend/Rust validation, so failed builds leave a draft release | Add a required validation job before draft creation. |
| Reproducible dependencies | Fail | workflow uses `npm install` | Use `npm ci` and keep the lockfile aligned. |
| UI critical paths | Fail | overlapping queues; keyboard-inaccessible file/watch cards; missing dialog focus handling; misleading empty search state | Complete Tasks 3-5. |
| Direct-transfer policy | Fail | frontend/back end still contain VPN, proxy, and throttle remnants | Complete Task 6. |
| Clean-break provenance | Pending owner audit | current repository history and documents contain legacy material | Do not publish v1.0 until this is reviewed outside the future public repository. |

---

### Task 1: Restore a Compilable Tauri Baseline

**Files:**
- Modify: `app/src-tauri/Cargo.toml`
- Modify: `app/src-tauri/src/lib.rs`
- Delete: `app/src-tauri/src/parallel_upload.rs`
- Delete: `app/src-tauri/src/parallel_download.rs`

**Produces:** a Tauri configuration whose tray code is feature-enabled and whose compiled module tree contains no unused code using invalid Grammers APIs.

- [x] Add Tauri's `tray-icon` feature to the existing `tauri` dependency; do not add a second tray dependency.
- [x] Remove only the `parallel_upload` and `parallel_download` module declarations from `lib.rs`.
- [x] Delete both module files after confirming their only references are those declarations.
- [x] Preserve the existing Grammers `upload_stream` path, which already owns its protocol-level multipart behavior.
- [ ] Run `cargo check --locked` on Windows MSVC in CI. Locally, record a missing `dlltool.exe` as an environment limitation rather than a passing Rust result.

### Task 2: Make Validation Precede Every Draft Release

**Files:**
- Modify: `.github/workflows/release.yml`
- Modify: `app/package.json` only if a release-validation script is added
- Create: `app/scripts/verify-release-metadata.js` only if workflow shell validation cannot keep version checks readable

**Produces:** no draft, asset upload, or published release when source validation, version alignment, changelog validation, or signing prerequisites fail.

- [x] Add a `validate-release` job on `windows-latest` before build and publication.
- [x] In validation, use Node and stable MSVC Rust, run `npm ci`, TypeScript validation, updater-key tests, and `cargo check --locked`.
- [x] Validate the tag version against all five version files and require exactly one nonempty `## [1.0.0]` release entry when a v1.0 tag is being prepared.
- [x] Build only after validation, then create and publish the release only after verified assets exist.
- [x] Replace `npm install` in `build-tauri` with `npm ci`.
- [x] Require the updater signing secret for release builds. A missing key fails the release.
- [x] Make asset upload fail when the installer or required updater metadata is absent; replace duplicate assets deliberately on rerun.
- [x] Publish only after successful validation, build, asset collection, and upload.

### Task 3: Fix Transfer Queue Visibility and Error Recovery

**Files:**
- Modify: `app/src/components/desktop/DesktopDashboard.tsx`
- Modify: `app/src/components/desktop/dashboard/UploadQueue.tsx`
- Modify: `app/src/components/desktop/dashboard/DownloadQueue.tsx`
- Modify: `app/src/components/desktop/dashboard/FileExplorer.tsx`

**Produces:** simultaneous uploads and downloads remain visible; a file-list failure tells the user what happened and offers recovery.

- [x] Add a narrow `bottomOffsetClass` prop to `UploadQueue`; when `downloadQueue.length > 0`, Dashboard places uploads above the download queue.
- [x] Keep the two queue components independent because their item models differ.
- [x] Pass the current search term, `onClearSearch`, and `onRetry` from Dashboard to FileExplorer.
- [x] Implement `onRetry` with `queryClient.invalidateQueries({ queryKey: ['files', activeFolderId] })`.
- [x] Display the received error message and a Retry control in the file explorer error state; retain detailed diagnostics in Logs.
- [ ] Test manually with one upload and one download, then with a failed file fetch followed by Retry.

### Task 4: Correct Search and Empty-State Copy for Windows

**Files:**
- Modify: `app/src/components/desktop/dashboard/FileExplorer.tsx`
- Modify: `app/src/components/desktop/dashboard/EmptyState.tsx`
- Modify: `app/src/components/desktop/DesktopDashboard.tsx`
- Modify: locale files under `app/src/i18n/locales/` when replacing visible copy

**Produces:** an empty folder, an empty search result, and a loading search each have distinct and truthful states.

- [x] Add an EmptyState variant for `search-empty` that displays the query and only a Clear search action.
- [x] Keep the folder-empty variant limited to manual upload; remove all promises of external drag-and-drop.
- [x] Replace `Cmd + F` with `Ctrl + F` in Windows-only copy and keep the existing shortcut focused on search.
- [x] Move the new empty and error-state strings into i18n keys.
- [ ] Verify: a populated folder searched with an unknown term never displays "This folder is empty" or an Upload Files call to action.

### Task 5: Make Desktop Interaction Keyboard and Modal Safe

**Files:**
- Create: `app/src/hooks/useModalDialog.ts`
- Modify: `app/src/components/desktop/dashboard/FileCard.tsx`
- Modify: `app/src/components/desktop/dashboard/FileListItem.tsx`
- Modify: `app/src/components/desktop/dashboard/RecentWatchBar.tsx`
- Modify: `app/src/components/desktop/dashboard/SettingsModal.tsx`
- Modify: `app/src/components/desktop/dashboard/LogsModal.tsx`

**Produces:** all audited actions are keyboard reachable and Settings/Logs behave as modal dialogs.

- [x] Give each file card, list item, and recent-watch item a focusable primary action with Enter/Space behavior; keep destructive, download, share, and remove controls as separate labeled buttons.
- [x] Ensure hover-only quick actions become visible on keyboard focus via `focus-visible` styles.
- [x] Add accessible names to every icon-only control touched by this task.
- [x] Implement `useModalDialog(isOpen, onClose, initialFocusRef)` with Escape close, focus on open, Tab/Shift+Tab cycling inside the dialog, and focus restoration on close.
- [x] Apply `role="dialog"`, `aria-modal="true"`, and an accessible heading relationship to SettingsModal and LogsModal first.
- [ ] Verify with keyboard only: open logs/settings, tab through controls, reverse-tab from the first control, press Escape, and confirm focus returns to the launching toolbar button.

### Task 6: Finish the Direct-Transfer Cleanup

**Files:**
- Modify: `app/src/context/SettingsContext.tsx`
- Modify: `app/src/components/desktop/dashboard/SettingsModal.tsx`
- Delete or modify only after tracing callers: `app/src-tauri/src/vpn_optimizer.rs`, `app/src-tauri/src/api_routes.rs`, `app/src-tauri/src/commands/fs.rs`, `app/src-tauri/src/commands/archive.rs`, `app/src-tauri/src/lib.rs`
- Modify: `app/src-tauri/Cargo.toml`
- Modify or delete unused `app/src/components/shared/QualitySelector.tsx` and related unused throttle constants after confirming zero UI callers

**Produces:** no user-facing or runtime proxy/VPN/network-optimizer/application-throttle path remains. Telegram FLOOD_WAIT protocol backoff and transfer retries remain intact.

- [x] Remove the unreachable network settings type, state, panel, and persisted keys.
- [x] Trace the transfer policy and backoff through local, API, archive, and URL upload paths; retain one shared direct-transfer policy.
- [x] Remove Grammers proxy features, Reqwest SOCKS support, and runtime proxy code after the caller trace reaches zero.
- [x] Remove the unused QualitySelector throttle implementation; media-quality selection remains only for real media variants.
- [x] Preserve FLOOD_WAIT handling as mandatory server-directed backoff, not a user-configurable throttle.

### Task 7: Apply the TeleStash v1.0 Identity Break

**Files:**
- Modify: `app/package.json`, `app/package-lock.json`, `app/src-tauri/Cargo.toml`, `app/src-tauri/Cargo.lock`, `app/src-tauri/tauri.conf.json`
- Replace: `CHANGELOG.md`, `README.md`, `docs/AI_HANDOFF.md`, `docs/RELEASE_RUNBOOK.md`
- Modify: source, locale, capability, autostart, update, stream-header, split-manifest, and diagnostics identifiers found by a repository-wide legacy-name scan
- Remove from the future public repository: legacy plans, copied upstream analysis, obsolete API documents, legacy screenshots, and old product references that are not required legal notices

**Produces:** a new `TeleStash` identity with version `1.0.0`, a fresh updater identity, and no legacy product names or compatibility behavior.

- [ ] Generate a new TeleStash updater signing key and configure its private key and password in the replacement repository only; never commit either secret.
- [x] Change product-facing and persistent identifiers together: log storage key, autostart name, MPV watch-later directory, streaming header, split manifest filename/field/caption prefix, and Telegram folder marker.
- [x] Do not implement migration readers for old identifiers. New TeleStash recognizes only new state; old local data remains untouched unless the owner removes it manually.
- [x] Rebuild CHANGELOG with one clean `## [1.0.0]` entry; do not retain historical release notes in the future public repository.
- [x] Rebuild README and agent handoff docs around actual shipped Windows-only behavior.
- [x] Run a case-insensitive scan for retired product names; generic Telegram protocol references are allowed only when technically required.

### Task 8: Provenance Gate and New Public Repository

**Owner-required actions:**
- Private provenance audit outside the future public repository.
- Explicit approval to delete the current GitHub repository and create the replacement `rasyidmmz/Telestash` repository.
- Configure replacement repository secrets and verify access control.

**Produces:** a public repository with a fresh root commit, no legacy commit/tag/release/action history, and no unreviewed borrowed material.

- [ ] Record the audit evidence privately. Do not publish documents that name or reproduce legacy source material.
- [ ] Obtain a legal/ownership conclusion from the owner or qualified counsel before publication. Removing Git history alone does not create copyright permission.
- [ ] After the owner authorizes the destructive operation in that task, create a new repository with a fresh root commit from the audited TeleStash tree.
- [ ] Set the remote, add the new updater signing secrets, and confirm the default branch is `main`.

### Task 9: Release Candidate Validation and Publication

**Files:**
- Modify: `CHANGELOG.md` only if the reviewed release notes need correction
- Modify: `README.md` only if validation reveals documentation differs from shipped behavior

**Produces:** an installer and updater asset set that is verified before v1.0.0 is described as released.

- [ ] Run frontend type check, updater unit tests, workflow validation job, and GitHub Windows MSVC build.
- [ ] Perform Windows smoke tests: first install, overwrite an existing TeleStash install, in-app update, startup from system tray, upload/download concurrently, search-empty state, keyboard-only Logs/Settings, MPV resume, ordinary file upload, and MP4/MKV split upload above `2_000_000_000` bytes.
- [ ] Verify the release has an NSIS installer, updater metadata, signatures created by the same TeleStash key, and a published non-draft GitHub release.
- [ ] Do not call the release successful until the asset download, installation, and in-app update path have been observed to succeed.

## Self-Review

- This plan covers every P1/P2 frontend finding from the 2026-07-26 audit.
- It prevents draft releases before build validation and preserves the existing tag-driven release model.
- It keeps the split-upload contract while deliberately rejecting legacy app state.
- It separates local implementation from provenance review and destructive remote operations.
- There are no TODO placeholders; every remaining owner action is an explicit gate rather than an implementation omission.
