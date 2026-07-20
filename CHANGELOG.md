# Changelog

All notable changes to Neko are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] — 2026-07-20

### Added

- Manual download dialog when a new version is found but auto-update cannot complete (opens GitHub Releases latest)
- Settings footer version from `app.getVersion()` instead of a hardcoded string

### Notes

- macOS auto-install still requires Apple Developer ID signing; this dialog is the fallback

## [0.1.1] — 2026-07-20

### Changed

- Default working hours cover the full day (`00:00`–`23:59`); narrow ranges remain configurable in Settings
- Working-hours time inputs use hour / minute selects instead of Ant Design `TimePicker`
- Upgrade Electron to 43
- GitHub Releases publish publicly when the release tag is pushed

### Fixed

- macOS Dock icon and visibility stay in sync while the settings window is shown, minimized, or hidden

### Docs

- README: Gatekeeper install steps placed directly under the download links

## [0.1.0] — 2026-07-19

### Added

- Cross-platform Electron desktop app (macOS / Windows / Linux)
- Break scheduler with configurable frequency and length
- Full-screen message-card and video break styles, plus system notifications
- Working hours (per weekday ranges) and Smart Breaks (idle / lock reset)
- Settings UI: Breaks, Hours, Appearance, System — with today’s status panel
- Break preview from Appearance without affecting stats
- Built-in calm break video and custom video import
- Sounds (optional; default muted), appearance colors / overlay opacity
- System tray with cup template icon; optional macOS menu-bar timer
- Locales: Chinese, English, Japanese (system default)
- Auto-update via `electron-updater` + GitHub Releases
- CI quality gates and multi-platform release workflow

### Packaging

- Separate macOS `arm64` / `x64` installers (not universal)
- Smaller asar by excluding renderer-only dependencies already bundled by Vite
- App icon from branded kitten artwork (transparent PNG → icns / ico)

### License

- PolyForm Noncommercial License 1.0.0  
  Required Notice: Copyright (c) 2026 MultCat Authors

[0.1.2]: https://github.com/rokiai/neko/releases/tag/v0.1.2
[0.1.1]: https://github.com/rokiai/neko/releases/tag/v0.1.1
[0.1.0]: https://github.com/rokiai/neko/releases/tag/v0.1.0
