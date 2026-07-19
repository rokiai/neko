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
  <a href="https://github.com/rokiai/neko/releases">Releases</a> ·
  <a href="./CHANGELOG.md">Changelog</a>
</p>

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

Download a prebuilt installer from GitHub Releases — no local build required.

1. Open [Releases](https://github.com/rokiai/neko/releases)
2. Under the latest release **Assets**, pick your platform:

| OS          | File                                                                   | How to install                                           |
| ----------- | ---------------------------------------------------------------------- | -------------------------------------------------------- |
| **macOS**   | `Neko-x.y.z-arm64.dmg` (Apple Silicon) or `Neko-x.y.z-x64.dmg` (Intel) | Open the DMG and drag Neko into Applications             |
| **Windows** | `Neko-x.y.z-setup.exe`                                                 | Run the setup wizard                                     |
| **Linux**   | `Neko-x.y.z.AppImage` or `.deb`                                        | AppImage: `chmod +x` then run; deb: `sudo dpkg -i *.deb` |

3. **macOS “unidentified developer”**  
   System Settings → Privacy & Security → Open Anyway; or:

   ```bash
   xattr -cr /Applications/Neko.app
   ```

4. After launch, Neko lives in the menu bar / tray. Closing the settings window does **not** quit the app — click the tray icon to reopen. Use **Quit** in the tray menu to exit fully.

## Build installers for three platforms with GitHub Actions

[`.github/workflows/release.yml`](.github/workflows/release.yml) already builds **macOS, Windows, and Linux** desktop packages (not Android).

**Option A — tag release**

```bash
git tag v0.1.0
git push origin v0.1.0
```

Pushing a `v*` tag builds all platforms and opens a **draft** GitHub Release.

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
