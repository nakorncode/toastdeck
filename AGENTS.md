# ToastDesk agent notes

These instructions apply to `G:\NakornCode\git\toastdeck` on **v2** (`main` after merge, or `cursor/toastdesk-v2-tauri-scaffold`).

## What this tree is

Unpackaged Tauri 2 + SolidJS + `solid-sonner` tray overlay. Tray settings, Windows notification capture, test toast, sounds, 9-point placement, duration, debug bounds. Identity stays `ToastDesk.v2` until a GitHub Release.

WPF v1 is **read-only** on branch `v1` / tag `v1-final`. TrayBits is a pattern, not a file dump.

## Identity

Until the first GitHub **Release** on `main`, v2 uses a separate identity:

| Key | Value |
| --- | --- |
| Product name | `ToastDesk` |
| AUMID | `NakornCode.ToastDesk.v2` |
| Settings | `%LOCALAPPDATA%\ToastDesk.v2\settings.json` |
| Run key value | `ToastDesk.v2` (`tauri-plugin-autostart` `app_name`) |
| Tauri identifier | `com.nakorncode.toastdesk.v2` |
| Executable | `ToastDesk.exe` |

The first GitHub Release on `main` **steals** the old ToastDesk identity. Uninstall v1 before that release.

## Locked product shape

- No user-facing main window. Overlay `"toast"` only.
- Left-click tray does nothing. Right-click the tray for settings.
- Overlay is visible only when a toast exists or debug is on.
- Tray: Launch on startup · Sound (on/off + 8 AOSP presets, default Argon) · Position (9-point, primary) · Duration (10s / 30s / 1 min / infinite) · Capture Windows notifications · Retry notification access · Debug overlay · Show test toast · Exit.
- First run: sound on, startup on, top-right, debug off, duration infinite, capture on.
- Test toast is obviously fake (`Test`). Debug uses bounds fill + a sample toast, not a decorated window.

## Working style

- PowerShell. `pnpm` for JS; Cargo for `src-tauri`.
- `pnpm tauri` runs `scripts/with-cargo.mjs` so Cursor terminals get Cargo **and** keep `pnpm` on `Path`.
- Read [docs/v2-plan.md](docs/v2-plan.md) before capture, packaging, or identity cutover.

## Verification

```powershell
pnpm typecheck
pnpm lint
pnpm build
cargo check --manifest-path .\src-tauri\Cargo.toml
```
