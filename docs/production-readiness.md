# ToastDesk Production Readiness

## Current Direction

- Product identity is `ToastDesk`.
- Target platform is Windows 11.
- Core value is persistent, visible desktop toast cards backed by Windows notification capture.

## Hardening Slices

- Stabilize app identity: executable name, AppUserModelID, shortcut, startup registration, settings path, and installer metadata.
- Improve lifecycle behavior: startup, minimized tray launch, graceful exit, duplicate process handling, and clean shutdown while capture is active.
- Improve notification behavior: capture filtering, deduplication, do-not-disturb behavior, overlay queue limits, and action handling.
- Improve settings: durable migration, clear defaults, restore defaults, notification sound presets/custom audio, and direct links to Windows notification permissions.
- Prepare packaging: self-contained `.exe`, installer path, icon, versioning, and uninstall cleanup.
- Add diagnostics: local logs for notification permission, shortcut registration, startup registration, and capture polling failures.

## Current Public Repo Assets

- `README.md`, `LICENSE`, `SECURITY.md`, and `CONTRIBUTING.md`
- GitHub Actions CI and release workflows
- `scripts/publish-release.ps1` for Windows x64 self-contained release ZIPs
- `installer/ToastDesk.iss` for an optional Inno Setup installer
- `assets/icons/ToastDesk.png` and `assets/icons/ToastDesk.ico`
