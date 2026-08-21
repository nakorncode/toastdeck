# ToastDesk

Persistent Windows notifications you cannot miss.

This repository is in a **v2 rewrite**. `main` is an empty Tauri-era tree (docs only for now). The shipping WPF app lives on branch [`v1`](https://github.com/nakorncode/toastdeck/tree/v1) and tag [`v1-final`](https://github.com/nakorncode/toastdeck/releases/tag/v1-final).

## Status

| Line | What it is |
| --- | --- |
| **v1** | Unpackaged .NET WPF app. Captures Windows Notification Center toasts and keeps them on screen. Last release: `v0.1.4`. |
| **v2 (this branch)** | Tauri 2 + SolidJS + `solid-sonner`. Tray overlay only at first. Real capture comes later. No application source in the first commit. |

Until a GitHub **Release** is published from this new `main`, installed v1 can keep running. **The first GitHub Release on this `main` takes over the old ToastDesk Windows identity.** Uninstall v1 before that release.

## v2 shape (planned)

- Almost no UI: always-on-top Sonner toast overlay + a tray icon.
- Settings live on the **tray context menu**: launch on startup, sound on/off and preset, overlay position, debug overlay, show test toast, exit.
- No settings window. No user-facing main window.
- Overlay positions: 9-point grid on the primary monitor (corners, edge middles, center). Default **top-right**.
- Unpackaged Tauri until capture work needs MSIX.

## Run v1 from this clone

```powershell
git switch v1
```

v1 docs, installer scripts, and source are on that branch only.

## Local path

```text
G:\NakornCode\git\toastdeck
```

Related pattern source (read-only; do not copy the suite into this repo):

```text
G:\NakornCode\git\traybits
```

## License

MIT. The v1 `LICENSE` file is on branch `v1`. A license file will return on `main` when application source lands.
