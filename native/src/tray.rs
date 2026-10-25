//! Notification-area icon and its right-click menu, built with Shell_NotifyIcon and popup menus.

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, LoadIconW, LoadImageW, SetForegroundWindow,
    TrackPopupMenu, HICON, HMENU, IDI_APPLICATION, IMAGE_ICON, LR_DEFAULTSIZE, LR_LOADFROMFILE, MF_CHECKED,
    MF_POPUP, MF_SEPARATOR, MF_STRING,
    MF_UNCHECKED, TPM_RETURNCMD, TPM_RIGHTBUTTON,
};

use crate::settings::{AppSettings, CardDuration, OverlayPlacement};
use crate::sound::SOUND_PRESETS;

/// Callback message the shell sends for tray icon mouse events.
pub const WM_APP_TRAY: u32 = 0x8002;
const TRAY_ID: u32 = 1;

pub const ID_STARTUP: u32 = 100;
pub const ID_HIGH_PRIORITY: u32 = 101;
pub const ID_LAUNCH_TOAST: u32 = 102;
pub const ID_SUPPRESS_WINDOWS_TOASTS: u32 = 103;
pub const ID_SOUND_ENABLED: u32 = 110;
pub const ID_SOUND_PRESET_BASE: u32 = 111;
pub const ID_POSITION_BASE: u32 = 130;
pub const ID_DURATION_BASE: u32 = 150;
pub const ID_CAPTURE: u32 = 160;
pub const ID_CAPTURE_RETRY: u32 = 161;
pub const ID_DEBUG: u32 = 162;
pub const ID_TEST_TOAST: u32 = 163;
pub const ID_EXIT: u32 = 164;

/// What the user picked from the tray menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Startup,
    HighPriority,
    LaunchToast,
    SuppressWindowsToasts,
    SoundEnabled,
    SoundPreset(usize),
    Position(OverlayPlacement),
    Duration(CardDuration),
    Capture,
    CaptureRetry,
    Debug,
    TestToast,
    Exit,
}

pub fn command_for(id: u32) -> Option<Command> {
    let position = |index: u32| OverlayPlacement::ALL.get(index as usize).copied();
    let duration = |index: u32| CardDuration::ALL.get(index as usize).copied();
    match id {
        ID_STARTUP => Some(Command::Startup),
        ID_HIGH_PRIORITY => Some(Command::HighPriority),
        ID_LAUNCH_TOAST => Some(Command::LaunchToast),
        ID_SUPPRESS_WINDOWS_TOASTS => Some(Command::SuppressWindowsToasts),
        ID_SOUND_ENABLED => Some(Command::SoundEnabled),
        ID_CAPTURE => Some(Command::Capture),
        ID_CAPTURE_RETRY => Some(Command::CaptureRetry),
        ID_DEBUG => Some(Command::Debug),
        ID_TEST_TOAST => Some(Command::TestToast),
        ID_EXIT => Some(Command::Exit),
        id if (ID_SOUND_PRESET_BASE..ID_SOUND_PRESET_BASE + SOUND_PRESETS.len() as u32).contains(&id) => {
            Some(Command::SoundPreset((id - ID_SOUND_PRESET_BASE) as usize))
        }
        id if (ID_POSITION_BASE..ID_POSITION_BASE + 9).contains(&id) => {
            position(id - ID_POSITION_BASE).map(Command::Position)
        }
        id if (ID_DURATION_BASE..ID_DURATION_BASE + 4).contains(&id) => {
            duration(id - ID_DURATION_BASE).map(Command::Duration)
        }
        _ => None,
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn flags(base: u32, checked: bool) -> windows::Win32::UI::WindowsAndMessaging::MENU_ITEM_FLAGS {
    let check = if checked { MF_CHECKED } else { MF_UNCHECKED };
    windows::Win32::UI::WindowsAndMessaging::MENU_ITEM_FLAGS(base | check.0)
}

unsafe fn append_item(menu: HMENU, id: u32, label: &str, checked: bool) {
    let text = wide(label);
    let _ = AppendMenuW(menu, flags(MF_STRING.0, checked), id as usize, PCWSTR(text.as_ptr()));
}

unsafe fn append_submenu(menu: HMENU, submenu: HMENU, label: &str) {
    let text = wide(label);
    let _ = AppendMenuW(menu, MF_POPUP, submenu.0 as usize, PCWSTR(text.as_ptr()));
}

unsafe fn build_menu(settings: &AppSettings) -> windows::core::Result<HMENU> {
    let menu = CreatePopupMenu()?;

    append_item(menu, ID_STARTUP, "Launch on startup", settings.launch_on_startup);
    append_item(menu, ID_HIGH_PRIORITY, "High priority on startup", settings.high_priority);
    append_item(menu, ID_LAUNCH_TOAST, "Show toast on launch", settings.show_launch_toast);
    append_item(menu, ID_SUPPRESS_WINDOWS_TOASTS, "Hide Windows toast banners", settings.suppress_windows_toasts);

    let sound = CreatePopupMenu()?;
    append_item(sound, ID_SOUND_ENABLED, "Enabled", settings.sound_enabled);
    let _ = AppendMenuW(sound, MF_SEPARATOR, 0, PCWSTR::null());
    for (index, preset) in SOUND_PRESETS.iter().enumerate() {
        let checked = settings.sound_preset == preset.id;
        append_item(sound, ID_SOUND_PRESET_BASE + index as u32, preset.label, checked);
    }
    append_submenu(menu, sound, "Sound");

    let position = CreatePopupMenu()?;
    for (index, placement) in OverlayPlacement::ALL.iter().enumerate() {
        let checked = settings.overlay_placement == *placement;
        append_item(position, ID_POSITION_BASE + index as u32, placement.label(), checked);
    }
    append_submenu(menu, position, "Position");

    let duration = CreatePopupMenu()?;
    for (index, value) in CardDuration::ALL.iter().enumerate() {
        let checked = settings.card_duration == *value;
        append_item(duration, ID_DURATION_BASE + index as u32, value.label(), checked);
    }
    append_submenu(menu, duration, "Duration");

    append_item(menu, ID_CAPTURE, "Capture Windows notifications", settings.windows_capture);
    append_item(menu, ID_CAPTURE_RETRY, "Retry notification access", false);
    append_item(menu, ID_DEBUG, "Debug overlay", settings.debug_overlay);
    append_item(menu, ID_TEST_TOAST, "Show test toast", false);
    append_item(menu, ID_EXIT, "Exit", false);
    Ok(menu)
}

/// Adds the icon to the notification area. Messages from it go to `hwnd` as `WM_APP_TRAY`.
pub fn add(hwnd: HWND, tooltip: &str) -> windows::core::Result<()> {
    let mut data = icon_data(hwnd, tooltip)?;
    unsafe { Shell_NotifyIconW(NIM_ADD, &mut data).ok() }
}

pub fn set_tooltip(hwnd: HWND, tooltip: &str) {
    if let Ok(mut data) = icon_data(hwnd, tooltip) {
        let _ = unsafe { Shell_NotifyIconW(NIM_MODIFY, &mut data) };
    }
}

pub fn remove(hwnd: HWND) {
    let mut data = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ID,
        ..Default::default()
    };
    let _ = unsafe { Shell_NotifyIconW(NIM_DELETE, &mut data) };
}

fn icon_data(hwnd: HWND, tooltip: &str) -> windows::core::Result<NOTIFYICONDATAW> {
    let mut data = NOTIFYICONDATAW {
        cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ID,
        uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
        uCallbackMessage: WM_APP_TRAY,
        hIcon: load_tray_icon(),
        ..Default::default()
    };
    for (slot, unit) in data.szTip.iter_mut().zip(tooltip.encode_utf16().take(127)) {
        *slot = unit;
    }
    Ok(data)
}

/// Shows the menu at the cursor and returns the chosen command, if any.
pub fn show_menu(hwnd: HWND, settings: &AppSettings) -> Option<Command> {
    unsafe {
        let menu = build_menu(settings).ok()?;
        let mut point = POINT::default();
        let _ = GetCursorPos(&mut point);
        // Required so the menu dismisses when the user clicks elsewhere.
        let _ = SetForegroundWindow(hwnd);
        let chosen = TrackPopupMenu(menu, TPM_RETURNCMD | TPM_RIGHTBUTTON, point.x, point.y, Some(0), hwnd, None);
        let _ = DestroyMenu(menu);
        if chosen.0 == 0 {
            None
        } else {
            command_for(chosen.0 as u32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sound_preset_ids_map_back_to_index() {
        assert_eq!(command_for(ID_SOUND_PRESET_BASE + 2), Some(Command::SoundPreset(2)));
    }

    #[test]
    fn position_and_duration_ids_map_to_values() {
        assert_eq!(command_for(ID_POSITION_BASE + 2), Some(Command::Position(OverlayPlacement::TopRight)));
        assert_eq!(command_for(ID_DURATION_BASE + 3), Some(Command::Duration(CardDuration::Infinite)));
    }

    #[test]
    fn unknown_id_is_ignored() {
        assert_eq!(command_for(999), None);
    }
}

/// The ToastDesk icon from `icon.ico` next to the executable, or the system default when missing.
fn load_tray_icon() -> HICON {
    use std::os::windows::ffi::OsStrExt;

    let icon_path = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("icon.ico")))
        .filter(|path| path.is_file());
    if let Some(path) = icon_path {
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
        let loaded = unsafe {
            LoadImageW(None, PCWSTR(wide.as_ptr()), IMAGE_ICON, 0, 0, LR_LOADFROMFILE | LR_DEFAULTSIZE)
        };
        if let Ok(handle) = loaded {
            return HICON(handle.0);
        }
    }
    unsafe { LoadIconW(None, IDI_APPLICATION).unwrap_or_default() }
}
