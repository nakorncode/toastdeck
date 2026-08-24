# ToastDesk

Persistent Windows notifications you cannot miss.

This repository is in a **v2 rewrite**. The shipping WPF app lives on branch [`v1`](https://github.com/nakorncode/toastdeck/tree/v1) and tag [`v1-final`](https://github.com/nakorncode/toastdeck/releases/tag/v1-final). GitHub **Latest** is still v1 (`v0.1.4`). v2 downloadable builds are **pre-releases** that keep a separate Windows identity.

## Status

| Line | What it is |
| --- | --- |
| **v1** | Unpackaged .NET WPF app. Captures Windows Notification Center toasts and keeps them on screen. Last release: `v0.1.4`. |
| **v2 (this branch)** | Tauri 2 + SolidJS + `solid-sonner` tray overlay. Unpackaged daily driver: Notification Center capture, test toast, sounds, 9-point position, duration, debug, startup. Needs Windows notification access. Pre-releases use identity `ToastDesk.v2` and can run beside v1. |

Until a **non-prerelease** GitHub Release is published from this `main`, installed v1 can keep running. **That first stable Release takes over the old ToastDesk Windows identity.** Uninstall v1 before it. Pre-releases do not.

## v2 shape

- Almost no UI: always-on-top Sonner overlay + a tray icon.
- Right-click tray: **Launch on startup**, **Show toast on launch**, **Sound**, **Position**, **Duration**, **Capture Windows notifications**, **Retry notification access**, **Debug overlay**, **Show test toast**, **Exit**.
- Capture needs Windows notification access. Existing Action Center toasts are not replayed on launch. Click a card to open the source app; × dismisses locally (Notification Center is unchanged).
- Left-click does nothing. Overlay shows only when a toast exists or debug is on.
- No settings window. No user-facing main window.
- Position: 9-point grid on the primary monitor. Default **top-right**.
- Duration: 10s / 30s / 1 min / infinite (default infinite).
- Unpackaged Tauri for local `pnpm tauri dev`. GitHub pre-releases ship NSIS (Setup.exe) and MSI. MSIX is still out of scope.
- First-run startup is **on** (Run value `ToastDesk.v2`). Disable v1 startup if both would launch.

## Download (v2 pre-release)

Pre-releases: [GitHub Releases](https://github.com/nakorncode/toastdeck/releases) tagged `v0.2.0-pre.*` (not Latest). Prefer **Setup.exe** (NSIS, per-user) or **MSI**. A portable zip is still attached. Requires [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/). Grant Windows notification access from the tray (**Retry notification access**) if capture is empty.

Push a `v*.*.*` tag (or run **Release** → workflow_dispatch with that tag) to build installers. Tags with a hyphen (for example `v0.2.0-pre.1`) stay pre-release and do not replace GitHub Latest.

## Development

Rust must be on `PATH` (`%USERPROFILE%\.cargo\bin`). Cursor terminals that were opened before rustup was installed will not see it; `pnpm tauri` prepends that directory so `pnpm tauri dev` still works.

```powershell
pnpm install
pnpm tauri dev
```

Checks:

```powershell
pnpm check
cargo check --manifest-path .\src-tauri\Cargo.toml
```

The no-bundle executable is written to `src-tauri\target\release\ToastDesk.exe` after `pnpm tauri build --no-bundle`. Installers:

```powershell
pnpm tauri build --bundles nsis,msi --ci --no-sign
./scripts/package-release.ps1 -ProductVersion v0.2.0-pre.1
```

## Run v1 from this clone

```powershell
git switch v1
```

## Local path

```text
G:\NakornCode\git\toastdeck
```

Related pattern source (read-only; do not copy the suite into this repo):

```text
G:\NakornCode\git\traybits
```

## License

MIT. See [LICENSE](LICENSE).
