# ToastDesk Production Readiness

## Current Direction

- Product identity is `ToastDesk`.
- Target platform is Windows 11.
- Core value is persistent, visible desktop toast cards backed by Windows notification capture.

## Hardening Slices

- Stabilize app identity: executable name, AppUserModelID, shortcut, startup registration, settings path, and installer metadata.
- Improve lifecycle behavior: startup, minimized tray launch, graceful exit, duplicate process handling, and clean shutdown while capture is active.
- Improve notification behavior: capture filtering, deduplication, do-not-disturb behavior, overlay queue limits, and action handling.
- Toast cards now use explicit `Open` and `Dismiss` actions. `Open` attempts to launch the source app by AppUserModelID when Windows exposes one, with ToastDesk as fallback.
- Notification capture prefers Windows change events when available. If WinRT event subscription fails in the unpackaged desktop app, ToastDesk falls back to guarded 500ms polling to keep latency low without overlapping capture calls.
- Improve settings: durable migration, clear defaults, restore defaults, notification sound presets/custom audio, and direct links to Windows notification permissions.
- Prepare packaging: self-contained `.exe`, installer path, icon, versioning, and uninstall cleanup.
- Add diagnostics: local logs for notification permission, shortcut registration, startup registration, and capture polling failures.

## Current Public Repo Assets

- `README.md`, `LICENSE`, `SECURITY.md`, and `CONTRIBUTING.md`
- GitHub Actions CI and release workflows
- `scripts/publish-release.ps1` for Windows x64 self-contained release ZIPs
- `installer/ToastDesk.iss` for an optional Inno Setup installer
- `assets/icons/ToastDesk.png` and `assets/icons/ToastDesk.ico`
