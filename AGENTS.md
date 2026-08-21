# ToastDesk agent notes

These instructions apply to `G:\NakornCode\git\toastdeck` on **`main` (v2)**.

## What this tree is

`main` is ToastDesk **v2**, started from an **orphan** commit with no WPF history. The first commit is docs only. Application source is added in later slices.

WPF v1 is **read-only history** on branch `v1` and tag `v1-final`. Inspect it when you need the old capture/behavior spec. Rewrite v2 from that spec and from TrayBits as a *pattern*. Do not copy TrayBits suite files (Caps Lock, settings shell, eye-rest) into this repo.

## Identity

Until the first GitHub **Release** on this `main`, v2 uses a separate identity so installed v1 can keep running:

| Key | Value |
| --- | --- |
| Product name | `ToastDesk` |
| AUMID | `NakornCode.ToastDesk.v2` |
| Mutex | `Local\NakornCode.ToastDesk.v2` |
| Settings | `%LOCALAPPDATA%\ToastDesk.v2` |
| Run key value | `ToastDesk.v2` |
| Start Menu | `ToastDesk v2.lnk` |
| Tauri identifier | `com.nakorncode.toastdesk.v2` |
| Executable | `ToastDesk.exe` |

The first GitHub Release on this `main` **steals** `NakornCode.ToastDesk`, mutex `Local\NakornCode.ToastDesk`, `%LOCALAPPDATA%\ToastDesk`, Run value `ToastDesk`, and `ToastDesk.lnk`. Uninstall v1 before that release. Pushing commits is not that cutover; publishing a Release is.

## Locked product shape

Tray-only overlay app. Hidden Tauri window only if the runtime requires one.

Tray: Launch on startup · Sound (enabled + short preset list) · Position · Debug overlay · Show test toast · Exit.

Position: 9-point grid, **primary monitor**, default **top-right**.

Debug overlay: show overlay bounds and a sample toast.

Stack when source lands: Tauri 2, SolidJS, `solid-sonner`, unpackaged. MSIX / `userNotificationListener` capability only when capture is in scope. Capture is not the first source slice.

## Working style

- PowerShell on Windows.
- Keep this `main` free of v1 WPF files.
- Treat tray, startup, overlay click-through, and identity strings as user-impacting surfaces.
- Read [docs/v2-plan.md](docs/v2-plan.md) before implementing overlay, tray, sound, position, debug, packaging, or capture.
- Read branch `v1` only when you need the old Windows capture or installer behavior as a spec.

## Verification

This commit has no application source. After a future scaffold, use the commands in `package.json` / `src-tauri` rather than restating them here.
