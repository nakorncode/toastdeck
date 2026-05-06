# ToastDesk

Persistent Windows notifications you cannot miss.

ToastDesk is a Windows 11 desktop app that mirrors Windows notifications into persistent, always-on-top toast cards. It keeps important alerts visible until you open or dismiss them.

![ToastDesk icon](assets/icons/ToastDesk.png)

## Download

Download the latest Windows build from:

[ToastDesk Releases](https://github.com/nakorncode/toastdeck/releases/latest)

Use `ToastDesk-win-x64.zip`, extract it, then run `ToastDesk.exe`.

ToastDesk is currently distributed as a self-contained Windows x64 ZIP package. No separate .NET runtime install is required.

## Status

ToastDesk is early public-preview software. Core notification capture, persistent overlay cards, tray behavior, settings, startup registration, notification actions, and notification sounds are implemented. Installer/signing work is still in progress.

## Features

- Persistent always-on-top toast cards
- Sonner-style stacked notification layout
- Windows Notification Center capture
- Activity list inside the app
- Tray/background app behavior
- Start with Windows
- Start minimized to tray
- Do Not Disturb mode
- Notification sound presets and custom sound files
- Production-safe bundled sound assets

## Release Flow

Maintainers can publish a GitHub Release by pushing a version tag:

```powershell
git tag v0.1.0
git push origin v0.1.0
```

The GitHub Actions release workflow will build `ToastDesk-win-x64.zip` and attach it to the release.

You can also create the package locally:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\publish-release.ps1
```

The local package is written to `artifacts/release/ToastDesk-win-x64.zip`.

## Requirements

- Windows 11
- Windows notification access permission
- No .NET runtime required when using the self-contained release package

## Run From Source

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\run-dev.ps1
```

## Build

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\run-build.ps1
```

## Publish Release Package Locally

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\publish-release.ps1
```

The release package is written to `artifacts/release/ToastDesk-win-x64.zip`.

## Installer

An Inno Setup script is provided at `installer/ToastDesk.iss`. It expects the published app under `artifacts/publish/win-x64`.

MSI packaging is not finalized yet. The current public-ready package target is a signed or unsigned Windows x64 ZIP/EXE package.

## Notification Sounds

ToastDesk bundles OpenCode notifier sounds from `@mohak34/opencode-notifier` and additional subtle notification sounds from `akx/Notifications`.

See [assets/sounds/README.md](assets/sounds/README.md) for source and license notes.

## Security

ToastDesk reads notification metadata through Windows notification listener APIs after the user grants permission. It stores settings locally under `%LOCALAPPDATA%\ToastDesk`.

Do not publish logs or screenshots that may contain private notification contents.

## License

MIT. See [LICENSE](LICENSE).
