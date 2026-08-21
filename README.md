# ToastDesk

Persistent Windows notifications you cannot miss.

This repository is in a **v2 rewrite**. Current `main` may still be docs-only until this branch is merged. The shipping WPF app lives on branch [`v1`](https://github.com/nakorncode/toastdeck/tree/v1) and tag [`v1-final`](https://github.com/nakorncode/toastdeck/releases/tag/v1-final).

## Status

| Line | What it is |
| --- | --- |
| **v1** | Unpackaged .NET WPF app. Captures Windows Notification Center toasts and keeps them on screen. Last release: `v0.1.4`. |
| **v2 (this branch)** | Tauri 2 + SolidJS + `solid-sonner` tray overlay. Test toast, sounds, 9-point position, duration, debug, startup. No Notification Center capture yet. |

Until a GitHub **Release** is published from this new `main`, installed v1 can keep running. **The first GitHub Release on this `main` takes over the old ToastDesk Windows identity.** Uninstall v1 before that release.

## v2 shape

- Almost no UI: always-on-top Sonner overlay + a tray icon.
- Right-click tray: **Launch on startup**, **Sound**, **Position**, **Duration**, **Debug overlay**, **Show test toast**, **Exit**.
- Left-click does nothing. Overlay shows only when a toast exists or debug is on.
- No settings window. No user-facing main window.
- Position: 9-point grid on the primary monitor. Default **top-right**.
- Duration: 10s / 30s / 1 min / infinite (default infinite).
- Unpackaged Tauri until capture work needs MSIX.
- First-run startup is **on** (Run value `ToastDesk.v2`). Disable v1 startup if both would launch.

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

The no-bundle executable is written to `src-tauri\target\release\ToastDesk.exe` after `pnpm tauri build --no-bundle`.

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
