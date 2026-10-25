# v3 parity checklist

Compare with v2 by running `G:\NakornCode\git\toastdeck-v2-ref` (tag `v0.2.0-pre.5`) or the installed v2
side by side. Both can run at once: v2 uses `ToastDesk.v2`, v3 uses `ToastDesk.v3`.

Status: **unit** = covered by `cargo test` in `native/`; **manual** = check by eye or by hand on a PC.

| Area | v2 behaviour | v3 | Check |
| --- | --- | --- | --- |
| Overlay position, 9 points | `overlayPlacement` | `overlay::overlay_position` | unit + manual |
| Card stack, newest first (bottom placement reversed) | Sonner order | `overlay::display_order` | unit + manual |
| Card height follows text, max one title line and two body lines | CSS line-clamp | Measured layout, body capped at `BODY_LINES` with an ellipsis | manual |
| Every captured card is kept; cards that do not fit the screen are not drawn (still queued) | Max four cards | `App::push`, `overlay::stack_layout` | manual |
| Appear and disappear instantly | Sonner animation | No animation (approved, Q2) | manual |
| Click card: open source app and dismiss | `open_toast` | `App::handle_hit(Hit::Card)` | manual |
| Click ×: dismiss one card | Sonner close button | `Hit::Close` | manual |
| Dismiss all: captured cards only, button at the top | (new in v3, Q13/Q16) | `Hit::DismissAll` keeps debug and capture-denied cards | manual |
| Duration 10 s / 30 s / 1 min / infinite | `cardDuration` | `App::expire` | manual |
| Sound on capture and launch, 8 AOSP presets | `sound.rs` | `sound.rs` (PlaySoundW, same files) | manual |
| Capture Windows notifications | `capture.rs` | `capture.rs` ported (same polling and text rules) | manual |
| Capture-denied notice, retry from tray | `capture-denied` | same id, `request_retry` | manual |
| Debug overlay: bounds fill and sample card | red dashed outline | bounds fill (Q12) | manual |
| Test toast | `native.rs` | overlay card (`Command::TestToast`); WinRT notifications from this app are not reliably shown | manual |
| Launch card after capture is ready | `schedule_launch_toast` | `start_launch_timer`, `App::show_launch` (overlay card) | manual |
| Tray menu items and order | `tray.rs` | `tray.rs` (same labels, same order) | manual |
| Left-click tray does nothing | yes | `show_menu` only on right-click | manual |
| Launch on startup (Run key) | `ToastDesk.v2` | `ToastDesk.v3` | manual |
| Hide Windows toast banners while running | not in v2 | **Not working.** Tray option is kept but does not hide banners (see Known issues) | manual |
| High priority with Above Normal fallback | `apply_priority` | `platform::apply_priority` | manual |
| Settings file and first-run copy from v2 | v2 path | `ToastDesk.v3\settings.json`, copied once (Q19) | unit + manual |
| Single instance | Tauri plugin | named mutex | manual |
| Installer: per-user NSIS and MSI | Tauri bundler | `packaging/windows` (Q6) | manual |

## Known differences to review

- Tray icon loads `icon.ico` from the install folder (same icon as v2). If the file is missing it falls back
  to the system default icon. The exe also embeds `icon.ico` at build time (`native/build.rs`).
- Windows 11 may place a new tray icon in the overflow area (^). A fresh machine may need the icon dragged
  onto the taskbar once.
- Card icon: for captured notifications the app's icon is taken from the executable behind its Start menu
  shortcut (matched by AppUserModelID). When no executable is found the bell emoji 🔔 is shown. Store and
  MSIX apps usually have no matching executable, so they show 🔔. Other emoji prefixes: ✅ launch,
  ⚠️ capture notice, 🛠 debug.
- Hover: the card gets a soft glow and the pointer shows a hand over clickable areas (cards, ×, Dismiss all).
  The × is larger and gets a round highlight on hover.
- Capture access is requested on the UI thread, as in v2, through an event and a reply channel.

## Known issues (pre-release)

1. **Windows Defender flags the build.** `Trojan:Win32/Bearfoos.A!ml` is detected on `ToastDeskV3.exe` from
   the current build (likely a heuristic false positive; not yet confirmed or submitted). Defender can
   quarantine the exe during install or build. Workaround: allow the file, or wait for a signed release.
2. **Hide Windows toast banners does not work.** The tray option exists but Windows still shows its own
   banners next to ToastDesk cards. Tested and ruled out on Windows 11 (build 26300): the global
   `Notifications\Settings\NOC_GLOBAL_SETTING_TOASTS_ENABLED`, `PushNotifications\ToastEnabled` (this is the
   Settings "Notifications" switch and turning it off also silences ToastDesk), and a per-app `Enabled=0`
   for ToastDesk's own AUMID (hides its own banners only). The option is currently not effective.
3. **"Show test toast" uses an overlay card**, not a Windows notification. Windows did not display WinRT
   notifications sent by this app during testing.
4. **Launch card is always shown in the overlay** on start (when "Show toast on launch" is on).
5. **Tray Exit and force kill:** a forced kill can leave a saved banner value behind; only relevant while
   the banner feature is in use (currently not effective, see 2).
6. **Not tested on a clean machine:** the MSI uninstall while the app runs, Start menu and Explorer icon
   after a fresh install, and the emoji colour rendering by eye.
