# ToastDesk agent notes (v3)

This branch is **v3**: a native Win32 + Direct2D app in `native/`. v2 (Tauri + SolidJS) stays on `main`
and tag `v0.2.0-pre.5`. Do not reintroduce a WebView or Tauri here.

## Layout

- `native/src/main.rs`: window procedure, app state, command handling, startup.
- `native/src/overlay.rs`: layered window, Direct2D/DirectWrite drawing, layout, hit testing (has unit tests).
- `native/src/tray.rs`: `Shell_NotifyIcon` and the right-click menu.
- `native/src/capture.rs`: `UserNotificationListener` polling, ported from v2 (same rules).
- `native/src/model.rs`: toast model and the queue from background threads to the UI thread.
- `native/src/settings.rs`, `sound.rs`, `platform.rs`: settings file, AOSP sounds, Run key, priority, AUMID.
- `packaging/windows/`: NSIS and WiX (MSI) scripts. `scripts/build-installers.ps1` builds both.
- `docs/v3-parity-checklist.md`: what must match v2, and what to check by hand.

## Identity

| Key | Value |
| --- | --- |
| Product name | `ToastDesk v3` |
| AUMID | `NakornCode.ToastDesk.v3` |
| Settings | `%LOCALAPPDATA%\ToastDesk.v3\settings.json` (first run copies v2 once; v2 is never changed) |
| Run key value | `ToastDesk.v3` |
| Executable | `ToastDeskV3.exe` |
| Install folder | `%LOCALAPPDATA%\ToastDesk.v3` (per user) |

Pre-releases use `v3.0.0-pre.N`. Non-pre-releases are not planned until v3 has parity.

## Locked product shape

Same as v2 (see `docs/v3-parity-checklist.md`). Tray: Launch on startup · High priority on startup ·
Show toast on launch · Hide Windows toast banners · Sound · Position · Duration · Capture Windows notifications · Retry notification access ·
Debug overlay · Show test toast · Exit. Overlay appears only when a toast exists or debug is on.
The overlay also has a "Dismiss all" row, which clears captured cards only.

## Verification

```powershell
cargo test --manifest-path .\native\Cargo.toml
cargo build --release --manifest-path .\native\Cargo.toml
powershell -ExecutionPolicy Bypass -File .\scripts\build-installers.ps1 -ProductVersion 3.0.0-pre.1
```
