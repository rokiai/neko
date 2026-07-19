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

**Current release: [v0.1.0](https://github.com/rokiai/neko/releases/latest)**

| Platform                  | Installer                                                                                            |
| ------------------------- | ---------------------------------------------------------------------------------------------------- |
| **macOS (Apple Silicon)** | [Neko-0.1.0-arm64.dmg](https://github.com/rokiai/neko/releases/download/v0.1.0/Neko-0.1.0-arm64.dmg) |
| **macOS (Intel)**         | [Neko-0.1.0-x64.dmg](https://github.com/rokiai/neko/releases/download/v0.1.0/Neko-0.1.0-x64.dmg)     |
| **Windows**               | [Neko-0.1.0-setup.exe](https://github.com/rokiai/neko/releases/download/v0.1.0/Neko-0.1.0-setup.exe) |
| **Linux (AppImage)**      | [Neko-0.1.0.AppImage](https://github.com/rokiai/neko/releases/download/v0.1.0/Neko-0.1.0.AppImage)   |
| **Linux (deb)**           | [neko_0.1.0_amd64.deb](https://github.com/rokiai/neko/releases/download/v0.1.0/neko_0.1.0_amd64.deb) |

All releases: [https://github.com/rokiai/neko/releases/latest](https://github.com/rokiai/neko/releases/latest)

> Links are generated from `package.json` version via `pnpm sync:readme`. Re-run after bumping the version (also runs on `pnpm version`).

<!-- DOWNLOAD_LINKS:END -->

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

## Install guide (recommended)

Use the [Download](#download) links above for the current installers (no local build required).

1. Pick your platform file and install:
   - **macOS**: open the DMG and drag Neko into Applications
   - **Windows**: run the setup wizard
   - **Linux AppImage**: `chmod +x` then run; **deb**: `sudo dpkg -i *.deb`
2. **macOS “unidentified developer”** — System Settings → Privacy & Security → Open Anyway; or:

   ```bash
   xattr -cr /Applications/Neko.app
   ```

3. After launch, Neko lives in the menu bar / tray. Closing the settings window does **not** quit the app — click the tray icon to reopen. Use **Quit** in the tray menu to exit fully.

## Build installers for three platforms with GitHub Actions

[`.github/workflows/release.yml`](.github/workflows/release.yml) already builds **macOS, Windows, and Linux** desktop packages (not Android).

**Option A — tag release**

```bash
# bump version (also refreshes README download links)
pnpm version patch   # or minor / major / 0.2.0
git push origin main --follow-tags
```

Pushing a `v*` tag builds all platforms and opens a **draft** GitHub Release. If you tag manually, run `pnpm sync:readme` after changing `package.json` version.

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
