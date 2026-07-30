# Changelog

## [1.1.0]

### Fixed

- Legacy split part file leakage in folder views ([teledrive-part], [telegram-drive-part], etc.).
- Inject media filename into MPV sidecar titles instead of numeric message IDs.
- Natural ascending sort order for playlists (Ep 01 -> Ep 02 -> Ep 10) and directional arrow key alignment.
- Safe Option map handling for API sparse fieldsets.
- Official v1.1.0 version badge display in the top-left sidebar header.

## [1.0.0]

### Added

- Windows notification-area controls, storage analytics, recent-watch history,
  MPV resume support, and detailed transfer diagnostics.
- Split upload for files over `2_000_000_000` bytes with manifest validation
  across streaming, download, move, and delete operations.

### Changed

- Established the TeleStash 1.0 Windows-only application identity.
- Removed unused parallel transfer modules and proxy, VPN, SOCKS, optimizer,
  and application bandwidth-throttle paths.
- Made the low-overhead rendering policy the application default while keeping
  progress indicators and interaction feedback visible.
- Improved dashboard empty, search, queue, dialog, and keyboard interaction
  states.

### Fixed

- The release workflow now validates version alignment and changelog metadata,
  builds signed assets before creating a release, rejects missing updater
  artifacts, replaces duplicate assets on reruns, and awaits publication.
