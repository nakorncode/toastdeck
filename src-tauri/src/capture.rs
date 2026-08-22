use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

use crate::{push_toast, remove_toast, AppState, Toast};

pub const CAPTURE_DENIED_ID: &str = "capture-denied";
const EVENT_BACKUP_POLL: Duration = Duration::from_secs(5);
const FAST_POLL: Duration = Duration::from_millis(500);
const WAIT_SLICE: Duration = Duration::from_millis(250);

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

pub fn start(app: AppHandle) {
    std::thread::spawn(move || run_loop(app));
}

pub fn request_retry(app: &AppHandle) {
    app.state::<AppState>()
        .capture_retry
        .store(true, Ordering::SeqCst);
}

pub fn overlay_id(windows_id: u32) -> String {
    format!("windows-{windows_id}")
}

pub fn title_and_body(texts: &[String], source: Option<&str>, id: u32) -> (String, String) {
    match texts {
        [] => (
            source.unwrap_or("Windows notification").to_string(),
            format!("Notification ID {id}"),
        ),
        [title] => (
            title.clone(),
            source.map_or_else(
                || format!("Notification ID {id}"),
                |name| format!("{name}\nNotification ID {id}"),
            ),
        ),
        [title, rest @ ..] => (title.clone(), rest.join("\n")),
    }
}

fn capture_enabled(app: &AppHandle) -> bool {
    app.state::<AppState>()
        .settings
        .lock()
        .map(|settings| settings.windows_capture)
        .unwrap_or(true)
}

fn retry_pending(app: &AppHandle) -> bool {
    app.state::<AppState>()
        .capture_retry
        .load(Ordering::SeqCst)
}

fn take_retry(app: &AppHandle) -> bool {
    app.state::<AppState>()
        .capture_retry
        .swap(false, Ordering::SeqCst)
}

fn set_access(app: &AppHandle, access: CaptureAccess) {
    if let Ok(mut current) = app.state::<AppState>().capture_access.lock() {
        *current = access;
    }
    apply_tooltip(app, access.tooltip());
}

fn apply_tooltip(app: &AppHandle, text: &str) {
    let state = app.state::<AppState>();
    let Ok(tray) = state.tray.lock() else {
        return;
    };
    if let Some(tray) = tray.as_ref() {
        let _ = tray.set_tooltip(Some(text));
    }
}

fn announce_denied(app: &AppHandle) {
    let announced = &app.state::<AppState>().capture_denied_announced;
    if announced.swap(true, Ordering::SeqCst) {
        return;
    }
    push_toast(
        app,
        Toast::overlay(
            CAPTURE_DENIED_ID,
            "ToastDesk",
            "Notification access is off. Right-click the tray and choose Retry notification access.",
            "capture",
        ),
        false,
    );
}

fn clear_denied(app: &AppHandle) {
    app.state::<AppState>()
        .capture_denied_announced
        .store(false, Ordering::SeqCst);
    remove_toast(app, CAPTURE_DENIED_ID);
}

fn wait_for_next(
    app: &AppHandle,
    wake_rx: &mpsc::Receiver<()>,
    interval: Duration,
    stop: &AtomicBool,
) {
    let deadline = Instant::now() + interval;
    loop {
        if stop.load(Ordering::SeqCst) || retry_pending(app) || !capture_enabled(app) {
            return;
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return;
        }
        match wake_rx.recv_timeout(remaining.min(WAIT_SLICE)) {
            Ok(()) | Err(RecvTimeoutError::Disconnected) => return,
            Err(RecvTimeoutError::Timeout) => {}
        }
    }
}

#[cfg(windows)]
fn run_loop(app: AppHandle) {
    let stop = AtomicBool::new(false);
    let (wake_tx, wake_rx) = mpsc::sync_channel::<()>(1);
    let mut seeded = false;
    let mut listener = None;
    let mut poll = FAST_POLL;

    loop {
        let force = take_retry(&app);
        if !capture_enabled(&app) {
            listener = None;
            seeded = false;
            poll = FAST_POLL;
            set_access(&app, CaptureAccess::Disabled);
            if force {
                let _ = ensure_access(&app, true);
            }
            wait_for_next(&app, &wake_rx, WAIT_SLICE, &stop);
            continue;
        }

        if force || listener.is_none() {
            match ensure_access(&app, force) {
                Ok(CaptureAccess::Allowed) => {
                    clear_denied(&app);
                    set_access(&app, CaptureAccess::Allowed);
                }
                Ok(access) => {
                    listener = None;
                    seeded = false;
                    set_access(&app, access);
                    if matches!(access, CaptureAccess::Denied | CaptureAccess::Unspecified) {
                        announce_denied(&app);
                    }
                    wait_for_next(&app, &wake_rx, FAST_POLL, &stop);
                    continue;
                }
                Err(error) => {
                    listener = None;
                    seeded = false;
                    set_access(&app, CaptureAccess::Unavailable);
                    eprintln!("capture: {error}");
                    wait_for_next(&app, &wake_rx, FAST_POLL, &stop);
                    continue;
                }
            }

            if listener.is_none() {
                match current_listener() {
                    Ok(current) => listener = Some(current),
                    Err(error) => {
                        set_access(&app, CaptureAccess::Unavailable);
                        eprintln!("capture: {error}");
                        wait_for_next(&app, &wake_rx, FAST_POLL, &stop);
                        continue;
                    }
                }
            }
        }

        let Some(current) = listener.as_ref() else {
            wait_for_next(&app, &wake_rx, FAST_POLL, &stop);
            continue;
        };

        if !seeded {
            seed_existing(&app, current);
            seeded = true;
            poll = subscribe_changes(current, wake_tx.clone());
        }

        capture_new(&app, current);
        wait_for_next(&app, &wake_rx, poll, &stop);
    }
}

#[cfg(not(windows))]
fn run_loop(app: AppHandle) {
    set_access(&app, CaptureAccess::Unavailable);
}

#[cfg(windows)]
fn current_listener() -> Result<windows::UI::Notifications::Management::UserNotificationListener, String>
{
    windows::UI::Notifications::Management::UserNotificationListener::Current()
        .map_err(|error| format!("UserNotificationListener is unavailable: {error}"))
}

#[cfg(windows)]
fn ensure_access(app: &AppHandle, force_prompt: bool) -> Result<CaptureAccess, String> {
    on_ui_thread(app, move || read_or_request_access(force_prompt))
}

#[cfg(windows)]
fn on_ui_thread<T, F>(app: &AppHandle, f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    app.run_on_main_thread(move || {
        let _ = tx.send(f());
    })
    .map_err(|error| error.to_string())?;
    rx.recv().map_err(|error| error.to_string())?
}

#[cfg(windows)]
fn read_or_request_access(force_prompt: bool) -> Result<CaptureAccess, String> {
    use windows::UI::Notifications::Management::UserNotificationListenerAccessStatus;

    let listener = current_listener()?;
    let mut access = listener
        .GetAccessStatus()
        .map_err(|error| format!("Could not read notification access: {error}"))?;

    if access == UserNotificationListenerAccessStatus::Unspecified || force_prompt {
        access = listener
            .RequestAccessAsync()
            .and_then(|operation| operation.get())
            .map_err(|error| format!("Notification access request failed: {error}"))?;
    }

    Ok(map_access(access))
}

#[cfg(windows)]
fn map_access(
    access: windows::UI::Notifications::Management::UserNotificationListenerAccessStatus,
) -> CaptureAccess {
    use windows::UI::Notifications::Management::UserNotificationListenerAccessStatus;

    if access == UserNotificationListenerAccessStatus::Allowed {
        CaptureAccess::Allowed
    } else if access == UserNotificationListenerAccessStatus::Denied {
        CaptureAccess::Denied
    } else {
        CaptureAccess::Unspecified
    }
}

#[cfg(windows)]
fn subscribe_changes(
    listener: &windows::UI::Notifications::Management::UserNotificationListener,
    wake_tx: mpsc::SyncSender<()>,
) -> Duration {
    use windows::Foundation::TypedEventHandler;
    use windows::UI::Notifications::Management::UserNotificationListener;
    use windows::UI::Notifications::{UserNotificationChangedEventArgs, UserNotificationChangedKind};

    let handler = TypedEventHandler::<UserNotificationListener, UserNotificationChangedEventArgs>::new(
        move |_sender, args| {
            if let Ok(args) = args.ok() {
                if args.ChangeKind() == Ok(UserNotificationChangedKind::Added) {
                    let _ = wake_tx.try_send(());
                }
            }
            Ok(())
        },
    );

    match listener.NotificationChanged(&handler) {
        Ok(_) => EVENT_BACKUP_POLL,
        Err(_) => FAST_POLL,
    }
}

#[cfg(windows)]
fn seed_existing(
    app: &AppHandle,
    listener: &windows::UI::Notifications::Management::UserNotificationListener,
) {
    let notifications = match current_toasts(listener) {
        Ok(notifications) => notifications,
        Err(error) => {
            write_capture_log(0, &error);
            return;
        }
    };
    let state = app.state::<AppState>();
    let Ok(mut ids) = state.captured_ids.lock() else {
        return;
    };
    for (id, _) in notifications {
        ids.insert(id);
    }
}

#[cfg(windows)]
fn capture_new(
    app: &AppHandle,
    listener: &windows::UI::Notifications::Management::UserNotificationListener,
) {
    let notifications = match current_toasts(listener) {
        Ok(notifications) => notifications,
        Err(error) => {
            write_capture_log(0, &error);
            return;
        }
    };

    for (id, notification) in notifications {
        let is_new = {
            let state = app.state::<AppState>();
            let Ok(mut ids) = state.captured_ids.lock() else {
                continue;
            };
            ids.insert(id)
        };
        if !is_new {
            continue;
        }

        match extract_details(id, &notification) {
            Ok((title, body, aumid)) => {
                push_toast(
                    app,
                    Toast {
                        id: overlay_id(id),
                        title,
                        body,
                        kind: "windows".into(),
                        source_app_user_model_id: aumid,
                    },
                    true,
                );
            }
            Err(error) => write_capture_log(id, &error),
        }
    }
}

#[cfg(windows)]
fn current_toasts(
    listener: &windows::UI::Notifications::Management::UserNotificationListener,
) -> Result<Vec<(u32, windows::UI::Notifications::UserNotification)>, String> {
    use windows::UI::Notifications::NotificationKinds;

    let notifications = listener
        .GetNotificationsAsync(NotificationKinds::Toast)
        .and_then(|operation| operation.get())
        .map_err(|error| format!("GetNotificationsAsync failed: {error}"))?;
    let size = notifications
        .Size()
        .map_err(|error| format!("Could not read notification list size: {error}"))?;

    let mut items = Vec::new();
    for index in 0..size {
        let Ok(notification) = notifications.GetAt(index) else {
            continue;
        };
        let Ok(id) = notification.Id() else {
            continue;
        };
        items.push((id, notification));
    }
    Ok(items)
}

#[cfg(windows)]
fn extract_details(
    id: u32,
    notification: &windows::UI::Notifications::UserNotification,
) -> Result<(String, String, Option<String>), String> {
    use windows::UI::Notifications::KnownNotificationBindings;

    let payload = notification
        .Notification()
        .map_err(|error| format!("Skipped notification; Windows did not expose toast payload: {error}"))?;
    let visual = payload
        .Visual()
        .map_err(|error| format!("Skipped notification; Windows did not expose toast visual: {error}"))?;
    let template = KnownNotificationBindings::ToastGeneric()
        .map_err(|error| format!("Skipped notification; ToastGeneric binding is unavailable: {error}"))?;
    let binding = visual
        .GetBinding(&template)
        .map_err(|error| format!("Skipped notification because Windows did not expose readable toast text: {error}"))?;
    let text_elements = binding
        .GetTextElements()
        .map_err(|error| format!("Skipped notification; toast text elements are unavailable: {error}"))?;
    let size = text_elements
        .Size()
        .map_err(|error| format!("Skipped notification; toast text size is unavailable: {error}"))?;

    let mut texts = Vec::new();
    for index in 0..size {
        if let Ok(element) = text_elements.GetAt(index) {
            if let Ok(text) = element.Text() {
                let text = text.to_string_lossy();
                if !text.trim().is_empty() {
                    texts.push(text);
                }
            }
        }
    }

    let (source, aumid) = extract_source(notification);
    let (title, body) = title_and_body(&texts, source.as_deref(), id);
    Ok((title, body, aumid))
}

#[cfg(windows)]
fn extract_source(
    notification: &windows::UI::Notifications::UserNotification,
) -> (Option<String>, Option<String>) {
    let Ok(app_info) = notification.AppInfo() else {
        return (None, None);
    };
    let name = app_info
        .DisplayInfo()
        .and_then(|info| info.DisplayName())
        .map(|name| name.to_string_lossy())
        .ok()
        .filter(|name| !name.trim().is_empty());
    let aumid = app_info
        .AppUserModelId()
        .map(|id| id.to_string_lossy())
        .ok()
        .filter(|id| !id.trim().is_empty());
    (name, aumid)
}

fn write_capture_log(id: u32, message: &str) {
    let Some(root) = std::env::var_os("LOCALAPPDATA") else {
        return;
    };
    let dir = std::path::PathBuf::from(root).join("ToastDesk.v2");
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    };
    let line = format!("{:?} Notification ID {id}: {message}\n", std::time::SystemTime::now());
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("notification-capture.log"))
        .and_then(|mut file| {
            use std::io::Write;
            file.write_all(line.as_bytes())
        });
}

pub fn open_source_app(aumid: &str) -> Result<(), String> {
    std::process::Command::new("explorer.exe")
        .arg(format!("shell:AppsFolder\\{aumid}"))
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("Could not open {aumid}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_id_uses_windows_prefix() {
        assert_eq!(overlay_id(42), "windows-42");
    }

    #[test]
    fn title_and_body_join_rest() {
        let texts = ["Hello".into(), "Line two".into(), "Line three".into()];
        let (title, body) = title_and_body(&texts, Some("Slack"), 7);
        assert_eq!(title, "Hello");
        assert_eq!(body, "Line two\nLine three");
    }

    #[test]
    fn title_and_body_fallback_without_text() {
        let (title, body) = title_and_body(&[], Some("Mail"), 9);
        assert_eq!(title, "Mail");
        assert_eq!(body, "Notification ID 9");
    }
}
