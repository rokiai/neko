# Neko

<p align="center">
  <img src="./resources/icon.png" alt="Neko" width="160" />
</p>

<p align="center">
  <strong>Elegant break reminders for desktop</strong><br />
  macOS · Windows · Linux
</p>

<p align="center">
  <a href="./README.zh-CN.md">中文</a> ·
  <a href="https://github.com/rokiai/neko/releases/latest">Download</a> ·
  <a href="https://github.com/rokiai/neko/releases">Releases</a> ·
  <a href="./CHANGELOG.md">Changelog</a>
</p>

<!-- DOWNLOAD_LINKS:START -->

## Download

**Current release: [v0.1.2](https://github.com/rokiai/neko/releases/latest)**

| Platform                  | Installer                                                                                            |
| ------------------------- | ---------------------------------------------------------------------------------------------------- |
| **macOS (Apple Silicon)** | [Neko-0.1.2-arm64.dmg](https://github.com/rokiai/neko/releases/download/v0.1.2/Neko-0.1.2-arm64.dmg) |
| **macOS (Intel)**         | [Neko-0.1.2-x64.dmg](https://github.com/rokiai/neko/releases/download/v0.1.2/Neko-0.1.2-x64.dmg)     |
| **Windows**               | [Neko-0.1.2-setup.exe](https://github.com/rokiai/neko/releases/download/v0.1.2/Neko-0.1.2-setup.exe) |
| **Linux (AppImage)**      | [Neko-0.1.2.AppImage](https://github.com/rokiai/neko/releases/download/v0.1.2/Neko-0.1.2.AppImage)   |
| **Linux (deb)**           | [neko_0.1.2_amd64.deb](https://github.com/rokiai/neko/releases/download/v0.1.2/neko_0.1.2_amd64.deb) |

All releases: [https://github.com/rokiai/neko/releases/latest](https://github.com/rokiai/neko/releases/latest)

> Links are generated from `package.json` version via `pnpm sync:readme`. Re-run after bumping the version (also runs on `pnpm version`).

<!-- DOWNLOAD_LINKS:END -->

## Install guide

Neko is **not notarized with an Apple Developer ID** yet. That is normal for this release; follow the steps below.

### macOS

1. Download the matching DMG (`arm64` for Apple Silicon, `x64` for Intel).
2. Open the DMG and drag **Neko** into **Applications**.
3. Open **Neko** from Applications.

#### If macOS says the app is damaged (“已损坏，无法打开”)

This is **Gatekeeper quarantine**, not a corrupt download. System Settings will **not** show **Open Anyway** for this message.

In Terminal:

```bash
xattr -cr /Applications/Neko.app
open /Applications/Neko.app
```

Then look for the cup icon in the **menu bar** (Neko is a menu-bar app and often has **no Dock icon**).

#### If macOS says the developer cannot be verified (“无法验证开发者”)

You may either:

- Right-click the app → **Open** → **Open**, or
- Dismiss the dialog, then within about an hour: **System Settings → Privacy & Security → Security → Open Anyway**.

The Terminal `xattr` command above also works for this case.

### Windows

1. Download `Neko-*-setup.exe` and run the installer.
2. If **SmartScreen** shows “Windows protected your PC” (unsigned installer):
   - Click **More info** → **Run anyway**.
3. Launch Neko from the Start menu / desktop shortcut; it appears in the **system tray**.

Windows does **not** use the macOS-style “app is damaged” quarantine message. SmartScreen is the usual prompt for unsigned builds.

### Linux

1. **AppImage**: `chmod +x Neko-*.AppImage && ./Neko-*.AppImage`
2. **deb**: `sudo dpkg -i neko_*_amd64.deb` (fix deps with `sudo apt -f install` if needed)

Linux generally does **not** block unsigned desktop apps the way macOS Gatekeeper does. You mainly need execute permission (AppImage) or package install rights (deb).

### After install (all platforms)

Closing the settings window does **not** quit Neko — use the tray / menu-bar icon. Choose **Quit** in the tray menu to exit fully.

Elegant cross-platform desktop break reminders, built with [electron-vite](https://electron-vite.org/) + React + TypeScript + Ant Design.

Repo: [rokiai/neko](https://github.com/rokiai/neko)

## Screenshots

<p align="center">
  <img src="./docs/screenshot/1.png" alt="Neko settings — breaks" width="720" />
</p>

<p align="center">
  <em>Break schedule, snooze options, and today’s status</em>
</p>

<p align="center">
  <img src="./docs/screenshot/2.png" alt="Neko settings — appearance with break preview" width="720" />
</p>

<p align="center">
  <em>Appearance settings with live break popup preview</em>
</p>

## Features

- Configurable break schedule (frequency / length)
- Message-card or video popup, plus system notifications
- Working hours and smart idle / lock reset
- System tray (macOS menu-bar icon; optional menu-bar timer)
- Sounds, appearance, launch at login, update checks
- Locales: Chinese / English / Japanese (follows system by default)

## Build installers for three platforms with GitHub Actions

[`.github/workflows/release.yml`](.github/workflows/release.yml) already builds **macOS, Windows, and Linux** desktop packages (not Android).

**Option A — tag release**

```bash
# bump version (also refreshes README download links)
pnpm version patch   # or minor / major / 0.2.0
git push origin main --follow-tags
```

Pushing a `v*` tag builds all platforms and publishes a GitHub Release. If you tag manually, run `pnpm sync:readme` after changing `package.json` version.

**Option B — manual run**

1. GitHub → **Actions** → **Release** → **Run workflow**
2. `dry_run=true`: build artifacts only  
   `dry_run=false` on a tag: also publish a Release

Artifacts include platform installers plus `latest*.yml` files used by auto-update.

## Development

```bash
pnpm install
pnpm dev          # use the Electron window, not a bare browser URL
```

```bash
pnpm lint && pnpm typecheck && pnpm test
pnpm build:mac    # or build:win / build:linux
```

## Auto-update

Powered by `electron-updater` against GitHub Releases:

- Checks run only in **packaged** apps (not `pnpm dev`)
- Users get a system notification when an update is available / downloaded
- Release assets must include electron-builder’s `latest*.yml` (uploaded by the Action)

Unsigned / unnotarized macOS builds may limit seamless auto-update; manual install from Releases still works.

## Structure

```
src/
  main/       # scheduler, tray, windows, persistence
  preload/    # typed IPC
  renderer/   # settings / break / sounds UI
  shared/     # shared types, i18n, pure logic
```

## License

[PolyForm Noncommercial License 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0)

Required Notice: Copyright (c) 2026 MultCat Authors

See [`LICENSE`](./LICENSE) in the repository root.
