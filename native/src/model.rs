//! Toast model and the hand-off between the capture thread and the UI thread.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::{mpsc::Sender, Mutex};
use std::time::Instant;

pub const DEBUG_TOAST_ID: &str = "debug-sample";
pub const CAPTURE_DENIED_ID: &str = "capture-denied";
pub const MAX_TOASTS: usize = 4;

/// Posted to the UI window whenever a background event is queued.
pub const WM_APP_EVENTS: u32 = 0x8001;

#[derive(Clone, Debug)]
pub struct Toast {
    pub id: String,
    pub title: String,
    pub body: String,
    pub source_aumid: Option<String>,
    /// True for notifications captured from Windows. Only these are cleared by "Dismiss all".
    pub captured: bool,
    pub created: Instant,
}

impl Toast {
    pub fn new(id: impl Into<String>, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            body: body.into(),
            source_aumid: None,
            captured: false,
            created: Instant::now(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaptureAccess {
    Disabled,
    Allowed,
    Denied,
    Unspecified,
    Unavailable,
}

impl CaptureAccess {
    pub fn tooltip(self) -> &'static str {
        match self {
            Self::Disabled => "ToastDesk — capture off",
            Self::Allowed => "ToastDesk — capture on",
            Self::Denied => "ToastDesk — notification access denied",
            Self::Unspecified => "ToastDesk — notification access needed",
            Self::Unavailable => "ToastDesk — capture unavailable",
        }
    }
}

pub enum Event {
    Push { toast: Toast, play_sound: bool },
    Remove(String),
    Access(CaptureAccess),
    /// The capture thread asks the UI thread to read or request notification access.
    AccessRequest { force: bool, reply: Sender<Result<CaptureAccess, String>> },
    /// Sent once capture is ready (or after a timeout) to show the launch message.
    Launch,
}

static EVENTS: Mutex<VecDeque<Event>> = Mutex::new(VecDeque::new());
pub static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);
pub static CAPTURE_ENABLED: AtomicBool = AtomicBool::new(true);
pub static CAPTURE_RETRY: AtomicBool = AtomicBool::new(false);
pub static CAPTURE_READY: AtomicBool = AtomicBool::new(false);

pub fn post_event(event: Event) {
    if let Ok(mut queue) = EVENTS.lock() {
        queue.push_back(event);
    }
    wake_ui();
}

pub fn take_events() -> Vec<Event> {
    EVENTS
        .lock()
        .map(|mut queue| queue.drain(..).collect())
        .unwrap_or_default()
}

#[cfg(windows)]
pub fn wake_ui() {
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::PostMessageW;

    let hwnd = MAIN_HWND.load(Ordering::SeqCst);
    if hwnd != 0 {
        let target = HWND(hwnd as *mut core::ffi::c_void);
        unsafe {
            let _ = PostMessageW(Some(target), WM_APP_EVENTS, WPARAM(0), LPARAM(0));
        }
    }
}

#[cfg(not(windows))]
pub fn wake_ui() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_toast_is_not_captured() {
        let toast = Toast::new("launch", "ToastDesk", "started");
        assert!(!toast.captured);
        assert!(toast.source_aumid.is_none());
    }
}
