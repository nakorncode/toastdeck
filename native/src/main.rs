//! ToastDesk v3: a tray app that shows Windows notifications as an overlay.
//! Native Win32 + Direct2D; no WebView. Behaviour follows v2 (see AGENTS.md).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod icon;
mod model;
mod overlay;
mod platform;
mod settings;
mod sound;
mod tray;

use std::cell::RefCell;
use std::mem::size_of;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::UI::Input::KeyboardAndMouse::{TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT};

/// Sent when the pointer leaves a window tracked with TrackMouseEvent (WM_MOUSELEAVE = 0x02A3).
const WM_MOUSELEAVE: u32 = 0x02A3;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetCursorPos, GetMessageW, LoadCursorW,
    PostQuitMessage, RegisterClassExW, SetCursor, SetTimer, TranslateMessage, HTCLIENT, IDC_ARROW, IDC_HAND, MSG,
    WM_DESTROY, WM_LBUTTONUP, WM_MOUSEMOVE, WM_RBUTTONUP, WM_SETCURSOR, WM_TIMER, WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
    WS_POPUP,
};

use capture::read_or_request_access;
use model::{CaptureAccess, Event, Toast, CAPTURE_ENABLED, CAPTURE_READY, DEBUG_TOAST_ID, MAIN_HWND, WM_APP_EVENTS};
use overlay::{Hit, Overlay};
use settings::{load_settings, save_settings, AppSettings};
use sound::{is_known_preset, SOUND_PRESETS};
use tray::{Command, WM_APP_TRAY};

const TIMER_EXPIRE: usize = 1;
const LAUNCH_WAIT: Duration = Duration::from_secs(10);

/// Everything the UI thread owns. Only touched from the window procedure.
struct App {
    hwnd: HWND,
    settings: AppSettings,
    toasts: Vec<Toast>,
    overlay: Overlay,
    access: CaptureAccess,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

fn with_app<T>(f: impl FnOnce(&mut App) -> T) -> Option<T> {
    APP.with(|cell| cell.try_borrow_mut().ok().and_then(|mut slot| slot.as_mut().map(f)))
}

impl App {
    fn render(&mut self) {
        if let Err(error) = self.overlay.render(&self.toasts, self.settings.debug_overlay, self.settings.overlay_placement) {
            crate::platform::log(&format!("overlay: {error}"));
        }
    }

    /// Newest first, no duplicate ids. Every card is kept; the overlay only draws the ones that fit on screen.
    fn push(&mut self, toast: Toast, play_sound: bool) {
        self.toasts.retain(|item| item.id != toast.id);
        self.toasts.insert(0, toast);
        if play_sound && self.settings.sound_enabled {
            if let Err(error) = sound::play(&self.settings.sound_preset) {
                crate::platform::log(&format!("sound: {error}"));
            }
        }
        self.render();
    }

    fn remove(&mut self, id: &str) {
        self.toasts.retain(|item| item.id != id);
        self.render();
    }

    fn sync_debug_toast(&mut self) {
        self.toasts.retain(|item| item.id != DEBUG_TOAST_ID);
        if self.settings.debug_overlay {
            self.push(Toast::new(DEBUG_TOAST_ID, "Debug", "Overlay bounds are visible."), false);
        } else {
            self.render();
        }
    }

    fn save(&self) {
        if let Err(error) = save_settings(&self.settings) {
            crate::platform::log(&format!("settings: {error}"));
        }
    }

    fn apply_side_effects(&mut self) {
        CAPTURE_ENABLED.store(self.settings.windows_capture, Ordering::SeqCst);
        platform::apply_startup(self.settings.launch_on_startup);
        platform::apply_priority(self.settings.high_priority);
        self.sync_windows_toasts();
    }

    /// Turns Windows banners off while the switch is on, and remembers the original value so it can
    /// be put back. Notification Center is not affected.
    fn sync_windows_toasts(&mut self) {
        if self.settings.suppress_windows_toasts {
            if self.settings.saved_windows_toast_setting.is_none() {
                let original = platform::read_windows_toasts().unwrap_or(-1);
                match platform::write_windows_toasts(0) {
                    Ok(()) => self.settings.saved_windows_toast_setting = Some(original),
                    Err(error) => platform::log(&error),
                }
            }
        } else {
            self.release_windows_toasts();
        }
        self.save();
    }

    /// Puts back the banner value saved before ToastDesk changed it.
    fn release_windows_toasts(&mut self) {
        if let Some(original) = self.settings.saved_windows_toast_setting.take() {
            if let Err(error) = platform::write_windows_toasts(original) {
                platform::log(&error);
            }
            self.save();
        }
    }

    fn update_settings(&mut self, mutate: impl FnOnce(&mut AppSettings)) {
        mutate(&mut self.settings);
        if !is_known_preset(&self.settings.sound_preset) {
            self.settings.sound_preset = settings::DEFAULT_SOUND_PRESET.into();
        }
        self.save();
        self.apply_side_effects();
    }

    fn handle_hit(&mut self, hit: Hit) {
        match hit {
            Hit::Card(index) => {
                let Some(toast) = self.toasts.get(index).cloned() else {
                    return;
                };
                if toast.id == DEBUG_TOAST_ID || toast.id == model::CAPTURE_DENIED_ID {
                    return;
                }
                if let Some(aumid) = toast.source_aumid.as_deref() {
                    if let Err(error) = capture::open_source_app(aumid) {
                        crate::platform::log(&format!("open toast: {error}"));
                    }
                }
                self.remove(&toast.id);
            }
            Hit::Close(index) => {
                if let Some(id) = self.toasts.get(index).map(|toast| toast.id.clone()) {
                    self.remove(&id);
                }
            }
            Hit::DismissAll => {
                self.toasts.retain(|toast| !toast.captured);
                self.render();
            }
        }
    }

    fn handle_command(&mut self, command: Command) {
        match command {
            Command::Startup => self.update_settings(|s| s.launch_on_startup = !s.launch_on_startup),
            Command::HighPriority => self.update_settings(|s| s.high_priority = !s.high_priority),
            Command::LaunchToast => self.update_settings(|s| s.show_launch_toast = !s.show_launch_toast),
            Command::SuppressWindowsToasts => {
                self.update_settings(|s| s.suppress_windows_toasts = !s.suppress_windows_toasts)
            }
            Command::SoundEnabled => self.update_settings(|s| s.sound_enabled = !s.sound_enabled),
            Command::SoundPreset(index) => {
                if let Some(preset) = SOUND_PRESETS.get(index) {
                    let id = preset.id.to_string();
                    self.update_settings(|s| {
                        s.sound_preset = id;
                        s.sound_enabled = true;
                    });
                }
            }
            Command::Position(placement) => self.update_settings(|s| s.overlay_placement = placement),
            Command::Duration(duration) => self.update_settings(|s| s.card_duration = duration),
            Command::Capture => self.update_settings(|s| s.windows_capture = !s.windows_capture),
            Command::CaptureRetry => capture::request_retry(),
            Command::Debug => {
                self.update_settings(|s| s.debug_overlay = !s.debug_overlay);
                self.sync_debug_toast();
            }
            Command::TestToast => {
                // Shows a card in the overlay directly. A WinRT notification from this app is not
                // always displayed by Windows, so it cannot be used to check the overlay.
                self.push(Toast::new("test", "Test", "ToastDesk overlay is working."), false);
            }
            Command::Exit => unsafe {
                let _ = DestroyWindow(self.hwnd);
            },
        }
        self.render();
    }

    fn handle_events(&mut self) {
        for event in model::take_events() {
            match event {
                Event::Push { toast, play_sound } => self.push(toast, play_sound),
                Event::Remove(id) => self.remove(&id),
                Event::Access(access) => {
                    self.access = access;
                    tray::set_tooltip(self.hwnd, access.tooltip());
                }
                Event::AccessRequest { force, reply } => {
                    let _ = reply.send(read_or_request_access(force));
                }
                Event::Launch => self.show_launch(),
            }
        }
    }

    /// Shows the "started" card in the overlay. The WinRT notification is not used here: Windows does not
    /// reliably display notifications sent by this app, so the card must always come from the overlay.
    fn show_launch(&mut self) {
        if !self.settings.show_launch_toast {
            return;
        }
        const TITLE: &str = "ToastDesk";
        const BODY: &str = "ToastDesk has started.";
        self.push(Toast::new("launch", TITLE, BODY), true);
    }

    /// Removes cards whose duration has passed. Infinite cards are never removed.
    fn expire(&mut self) {
        let Some(limit) = self.settings.card_duration.millis() else {
            return;
        };
        let limit = Duration::from_millis(limit);
        let before = self.toasts.len();
        self.toasts.retain(|toast| toast.created.elapsed() < limit);
        if self.toasts.len() != before {
            self.render();
        }
    }
}

unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_APP_EVENTS => {
            with_app(|app| app.handle_events());
            LRESULT(0)
        }
        WM_APP_TRAY => {
            let mouse = (lparam.0 & 0xFFFF) as u32;
            if mouse == WM_RBUTTONUP {
                let settings = with_app(|app| app.settings.clone());
                if let Some(settings) = settings {
                    if let Some(command) = tray::show_menu(hwnd, &settings) {
                        if command == Command::Exit {
                            let _ = DestroyWindow(hwnd);
                        } else {
                            with_app(|app| app.handle_command(command));
                        }
                    }
                }
            }
            LRESULT(0)
        }
        WM_MOUSEMOVE => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
            let mut tracking = TRACKMOUSEEVENT {
                cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: hwnd,
                dwHoverTime: 0,
            };
            let _ = TrackMouseEvent(&mut tracking);
            with_app(|app| {
                let hover = app.overlay.hit_test(x, y);
                if app.overlay.set_hover(hover) {
                    app.render();
                }
            });
            LRESULT(0)
        }
        WM_MOUSELEAVE => {
            with_app(|app| {
                if app.overlay.set_hover(None) {
                    app.render();
                }
            });
            LRESULT(0)
        }
        WM_SETCURSOR if (lparam.0 & 0xFFFF) as u32 == HTCLIENT => {
            let mut point = POINT::default();
            let _ = GetCursorPos(&mut point);
            let _ = ScreenToClient(hwnd, &mut point);
            let clickable = with_app(|app| app.overlay.hit_test(point.x, point.y).is_some()).unwrap_or(false);
            let name = if clickable { IDC_HAND } else { IDC_ARROW };
            if let Ok(cursor) = LoadCursorW(None, name) {
                SetCursor(Some(cursor));
            }
            LRESULT(1)
        }
        WM_LBUTTONUP => {
            let x = (lparam.0 & 0xFFFF) as i16 as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
            with_app(|app| {
                if let Some(hit) = app.overlay.hit_test(x, y) {
                    app.handle_hit(hit);
                }
            });
            LRESULT(0)
        }
        WM_TIMER if wparam.0 == TIMER_EXPIRE => {
            with_app(|app| app.expire());
            LRESULT(0)
        }
        WM_DESTROY => {
            with_app(|app| app.release_windows_toasts());
            tray::remove(hwnd);
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn start_launch_timer() {
    std::thread::spawn(|| {
        let deadline = Instant::now() + LAUNCH_WAIT;
        while Instant::now() < deadline && !CAPTURE_READY.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(50));
        }
        std::thread::sleep(Duration::from_millis(150));
        model::post_event(Event::Launch);
    });
}

unsafe fn create_window() -> windows::core::Result<HWND> {
    let instance = GetModuleHandleW(None)?;
    let class_name = w!("ToastDeskOverlay");
    let class = WNDCLASSEXW {
        cbSize: size_of::<WNDCLASSEXW>() as u32,
        lpfnWndProc: Some(window_proc),
        hInstance: instance.into(),
        lpszClassName: class_name,
        ..Default::default()
    };
    RegisterClassExW(&class);
    CreateWindowExW(
        WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
        class_name,
        w!("ToastDesk"),
        WS_POPUP,
        0,
        0,
        1,
        1,
        None,
        None,
        Some(instance.into()),
        None,
    )
}

fn run() -> windows::core::Result<()> {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
    platform::set_process_identity();
    if let Err(error) = platform::register_aumid() {
        platform::log(&error);
    }
    // Called by the installer and uninstaller before they stop the app.
    if std::env::args().any(|arg| arg == "--restore-windows-toasts") {
        restore_windows_toasts_only();
        return Ok(());
    }
    if !platform::acquire_single_instance() {
        return Ok(());
    }

    let mut settings = load_settings();
    // A value still saved here means the last run ended without restoring it (for example a crash).
    if let Some(original) = settings.saved_windows_toast_setting.take() {
        if let Err(error) = platform::write_windows_toasts(original) {
            platform::log(&error);
        }
    }
    let _ = save_settings(&settings);
    platform::apply_priority(settings.high_priority);
    platform::apply_startup(settings.launch_on_startup);
    CAPTURE_ENABLED.store(settings.windows_capture, Ordering::SeqCst);

    let hwnd = unsafe { create_window()? };
    MAIN_HWND.store(hwnd.0 as isize, Ordering::SeqCst);
    let overlay = Overlay::new(hwnd)?;
    match tray::add(hwnd, "ToastDesk") {
        Ok(()) => platform::log("tray: icon added"),
        Err(error) => platform::log(&format!("tray: {error}")),
    }

    let mut app = App {
        hwnd,
        settings,
        toasts: Vec::new(),
        overlay,
        access: CaptureAccess::Disabled,
    };
    app.sync_debug_toast();
    app.sync_windows_toasts();
    APP.with(|cell| *cell.borrow_mut() = Some(app));

    capture::start();
    start_launch_timer();
    unsafe {
        SetTimer(Some(hwnd), TIMER_EXPIRE, 1000, None);
    }

    let mut message = MSG::default();
    unsafe {
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    APP.with(|cell| cell.borrow_mut().take());
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        crate::platform::log(&format!("ToastDesk failed: {error}"));
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use settings::{CardDuration, OverlayPlacement};

    #[test]
    fn default_duration_is_infinite() {
        assert_eq!(AppSettings::default().card_duration, CardDuration::Infinite);
        assert_eq!(AppSettings::default().overlay_placement, OverlayPlacement::TopRight);
    }
}

/// Puts back the Windows banner value when ToastDesk saved one, then returns without starting.
fn restore_windows_toasts_only() {
    let mut settings = load_settings();
    if let Some(original) = settings.saved_windows_toast_setting.take() {
        if let Err(error) = platform::write_windows_toasts(original) {
            platform::log(&error);
        }
        let _ = save_settings(&settings);
    }
}
