# v3 parity checklist

Compare with v2 by running `G:\NakornCode\git\toastdeck-v2-ref` (tag `v0.2.0-pre.5`) or the installed v2
side by side. Both can run at once: v2 uses `ToastDesk.v2`, v3 uses `ToastDesk.v3`.

Status: **unit** = covered by `cargo test` in `native/`; **manual** = check by eye or by hand on a PC.

| Area | v2 behaviour | v3 | Check |
| --- | --- | --- | --- |
| Overlay position, 9 points | `overlayPlacement` | `overlay::overlay_position` | unit + manual |
| Card stack, newest first (bottom placement reversed) | Sonner order | `overlay::display_order` | unit + manual |
| Card height follows text, max one title line and two body lines | CSS line-clamp | Measured layout, clipped by `BODY_LINES` | manual |
| Max four cards, oldest non-debug card dropped | `MAX_TOASTS` | `App::push` | manual |
| Appear and disappear instantly | Sonner animation | No animation (approved, Q2) | manual |
| Click card: open source app and dismiss | `open_toast` | `App::handle_hit(Hit::Card)` | manual |
| Click ×: dismiss one card | Sonner close button | `Hit::Close` | manual |
| Dismiss all: captured cards only | (new in v3, Q13/Q16) | `Hit::DismissAll` keeps debug and capture-denied cards | manual |
| Duration 10 s / 30 s / 1 min / infinite | `cardDuration` | `App::expire` | manual |
| Sound on capture and launch, 8 AOSP presets | `sound.rs` | `sound.rs` (PlaySoundW, same files) | manual |
| Capture Windows notifications | `capture.rs` | `capture.rs` ported (same polling and text rules) | manual |
| Capture-denied notice, retry from tray | `capture-denied` | same id, `request_retry` | manual |
| Debug overlay: bounds fill and sample card | red dashed outline | bounds fill (Q12) | manual |
| Test toast (WinRT) | `native.rs` | `platform::show_native_toast` | manual |
| Launch toast after capture is ready | `schedule_launch_toast` | `start_launch_timer`, `App::show_launch` | manual |
| Tray menu items and order | `tray.rs` | `tray.rs` (same labels, same order) | manual |
| Left-click tray does nothing | yes | `show_menu` only on right-click | manual |
| Launch on startup (Run key) | `ToastDesk.v2` | `ToastDesk.v3` | manual |
| High priority with Above Normal fallback | `apply_priority` | `platform::apply_priority` | manual |
| Settings file and first-run copy from v2 | v2 path | `ToastDesk.v3\settings.json`, copied once (Q19) | unit + manual |
| Single instance | Tauri plugin | named mutex | manual |
| Installer: per-user NSIS and MSI | Tauri bundler | `packaging/windows` (Q6) | manual |

## Known differences to review

- Tray icon uses the system default application icon, not the ToastDesk icon from v2. Embedding the
  icon needs a resource file (not done yet).
- Text is clipped rather than ellipsised when it passes the line limit.
- Capture access is requested on the UI thread, as in v2, through an event and a reply channel.
