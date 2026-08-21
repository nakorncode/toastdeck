# ToastDesk v2 plan

Locked 2026-08-21. Source of truth for product and git decisions on `main`. Change this file when the lock changes.

## Why v2 exists

v1 (WPF) is a working notification-capture app. v2 is a rewrite: Tauri 2 + SolidJS + `solid-sonner`, almost no chrome, settings on the tray. v1 is frozen, not deleted.

## Git

| Ref | Role |
| --- | --- |
| `v1` branch | Frozen WPF tree (`fcc820b` and ancestors). |
| `v1-final` tag | Named freeze of that tip. Older tags `v0.1.0`–`v0.1.4` still point at v1 history. |
| `main` | Orphan v2 root. No parent in v1 history. |
| `develop` | Reset to the same orphan root as `main`. |

`main` was force-pushed once to plant this root. Further force-pushes to `main` are out of scope unless explicitly requested.

## Milestones

1. **Docs-only root (this commit).** No `src`, no `src-tauri`, no `package.json`.
2. **Scaffold.** Unpackaged Tauri 2 + SolidJS app. Tray + transparent `"toast"` overlay. No capture. Identity strings from `AGENTS.md`.
3. **Overlay MVP.** `solid-sonner` cards, test toast from tray, sound presets, 9-point position, debug bounds, launch on startup.
4. **Capture.** Port behavior from branch `v1` (UserNotificationListener, permission, dedupe, open/dismiss) into Rust. Decide MSIX then. Do not start from a TrayBits file dump.
5. **Identity cutover.** First GitHub **Release** on this `main` takes the old ToastDesk identity. Uninstall v1 first. Overlay-only may be that release if a Release is published before capture exists.

## TrayBits

Path: `G:\NakornCode\git\traybits`.

Use it as a pattern: transparent toast window, always-on-top, click-through, Sonner overlay, tray lifecycle, startup Run key, overlay debug flags.

Do not import the PowerToys shell, Caps Lock hook, eye-rest, language indicator, or the monolithic `lib.rs`. Rewrite a slim ToastDesk surface in this repo.

v1 WPF remains the capture spec (event + polling hybrid, unpackaged listener, overlay stacking, open-by-AUMID). TrayBits also has a Rust listener; treat both as references, neither as a copy source.

## Overlay and tray

- Windows: overlay labeled `"toast"`; no user-facing main window.
- Default placement: primary monitor, top-right.
- Positions: top-left, top-center, top-right, middle-left, center, middle-right, bottom-left, bottom-center, bottom-right.
- Sound: on/off plus a **short** preset list (about 5–8), selected from a tray submenu.
- Debug: overlay window bounds visible and a sample toast on screen.
- Test toast: tray item that pushes a demo card (and plays sound if enabled).

## Out of scope until asked

Activity list window, Do Not Disturb, per-monitor picker, TrayBits 50-sound catalog, MSIX before capture, stealing v1 identity before a GitHub Release.
