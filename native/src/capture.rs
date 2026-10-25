//! Captures Windows toast notifications through `UserNotificationListener`.
//! Port of the v2 capture loop: same polling, same access rules, same text rules.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use crate::model::{
    post_event, Event, CaptureAccess, Toast, CAPTURE_DENIED_ID, CAPTURE_ENABLED, CAPTURE_READY,
    CAPTURE_RETRY,
};

const EVENT_BACKUP_POLL: Duration = Duration::from_secs(5);
const FAST_POLL: Duration = Duration::from_millis(500);
const WAIT_SLICE: Duration = Duration::from_millis(250);

static DENIED_ANNOUNCED: AtomicBool = AtomicBool::new(false);

pub fn start() {
    std::thread::spawn(run_loop);
}

pub fn request_retry() {
    CAPTURE_RETRY.store(true, Ordering::SeqCst);
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

fn wait_for_next(wake_rx: &mpsc::Receiver<()>, interval: Duration) {
    let deadline = Instant::now() + interval;
    loop {
        if CAPTURE_RETRY.load(Ordering::SeqCst) || !CAPTURE_ENABLED.load(Ordering::SeqCst) {
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

fn set_access(access: CaptureAccess) {
    post_event(Event::Access(access));
}

fn announce_denied() {
    if DENIED_ANNOUNCED.swap(true, Ordering::SeqCst) {
        return;
    }
    post_event(Event::Push {
        toast: Toast::new(
            CAPTURE_DENIED_ID,
            "ToastDesk",
            "Notification access is off. Right-click the tray and choose Retry notification access.",
        ),
        play_sound: false,
    });
}

fn clear_denied() {
    DENIED_ANNOUNCED.store(false, Ordering::SeqCst);
    post_event(Event::Remove(CAPTURE_DENIED_ID.to_string()));
}

#[cfg(windows)]
fn run_loop() {
    use windows::UI::Notifications::Management::UserNotificationListener;

    let (wake_tx, wake_rx) = mpsc::sync_channel::<()>(1);
    let mut seeded = false;
    let mut listener: Option<UserNotificationListener> = None;
    let mut seen: HashSet<u32> = HashSet::new();
    let mut poll = FAST_POLL;

    loop {
        let force = CAPTURE_RETRY.swap(false, Ordering::SeqCst);
        if !CAPTURE_ENABLED.load(Ordering::SeqCst) {
            listener = None;
            seeded = false;
            poll = FAST_POLL;
            set_access(CaptureAccess::Disabled);
            CAPTURE_READY.store(true, Ordering::SeqCst);
            if force {
                let _ = ensure_access(true);
            }
            wait_for_next(&wake_rx, WAIT_SLICE);
            continue;
        }

        if force || listener.is_none() {
            match ensure_access(force) {
                Ok(CaptureAccess::Allowed) => {
                    clear_denied();
                    set_access(CaptureAccess::Allowed);
                }
                Ok(access) => {
                    listener = None;
                    seeded = false;
                    set_access(access);
                    CAPTURE_READY.store(true, Ordering::SeqCst);
                    if matches!(access, CaptureAccess::Denied | CaptureAccess::Unspecified) {
                        announce_denied();
                    }
                    wait_for_next(&wake_rx, FAST_POLL);
                    continue;
                }
                Err(error) => {
                    listener = None;
                    seeded = false;
                    set_access(CaptureAccess::Unavailable);
                    CAPTURE_READY.store(true, Ordering::SeqCst);
                    crate::platform::log(&format!("capture: {error}"));
                    wait_for_next(&wake_rx, FAST_POLL);
                    continue;
                }
            }

            if listener.is_none() {
                match current_listener() {
                    Ok(current) => listener = Some(current),
                    Err(error) => {
                        set_access(CaptureAccess::Unavailable);
                        CAPTURE_READY.store(true, Ordering::SeqCst);
                        crate::platform::log(&format!("capture: {error}"));
                        wait_for_next(&wake_rx, FAST_POLL);
                        continue;
                    }
                }
            }
        }

        let Some(current) = listener.as_ref() else {
            wait_for_next(&wake_rx, FAST_POLL);
            continue;
        };

        if !seeded {
            seed_existing(current, &mut seen);
            seeded = true;
            poll = subscribe_changes(current, wake_tx.clone());
            CAPTURE_READY.store(true, Ordering::SeqCst);
        }

        capture_new(current, &mut seen);
        wait_for_next(&wake_rx, poll);
    }
}

#[cfg(not(windows))]
fn run_loop() {
    set_access(CaptureAccess::Unavailable);
    CAPTURE_READY.store(true, Ordering::SeqCst);
}

/// Reads or requests notification access on the UI thread, as v2 did.
/// The capture thread waits for the answer through a reply channel.
#[cfg(windows)]
fn ensure_access(force_prompt: bool) -> Result<CaptureAccess, String> {
    let (reply_tx, reply_rx) = mpsc::channel();
    post_event(Event::AccessRequest { force: force_prompt, reply: reply_tx });
    reply_rx.recv().map_err(|error| error.to_string())?
}

/// Runs on the UI thread when an `AccessRequest` event is handled.
#[cfg(windows)]
pub fn read_or_request_access(force_prompt: bool) -> Result<CaptureAccess, String> {
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
fn current_listener() -> Result<windows::UI::Notifications::Management::UserNotificationListener, String>
{
    windows::UI::Notifications::Management::UserNotificationListener::Current()
        .map_err(|error| format!("UserNotificationListener is unavailable: {error}"))
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
    listener: &windows::UI::Notifications::Management::UserNotificationListener,
    seen: &mut HashSet<u32>,
) {
    match current_toasts(listener) {
        Ok(notifications) => {
            for (id, _) in notifications {
                seen.insert(id);
            }
        }
        Err(error) => write_capture_log(0, &error),
    }
}

#[cfg(windows)]
fn capture_new(
    listener: &windows::UI::Notifications::Management::UserNotificationListener,
    seen: &mut HashSet<u32>,
) {
    let notifications = match current_toasts(listener) {
        Ok(notifications) => notifications,
        Err(error) => {
            write_capture_log(0, &error);
            return;
        }
    };

    for (id, notification) in notifications {
        if !seen.insert(id) {
            continue;
        }
        match extract_details(id, &notification) {
            Ok((title, body, aumid)) => {
                let mut toast = Toast::new(overlay_id(id), title, body);
                toast.captured = true;
                toast.source_exe = aumid.as_deref().and_then(crate::icon::exe_for_aumid);
                toast.source_aumid = aumid;
                post_event(Event::Push { toast, play_sound: true });
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

/// Same log file as v2, in the v3 settings folder.
fn write_capture_log(id: u32, message: &str) {
    let Some(root) = std::env::var_os("LOCALAPPDATA") else {
        return;
    };
    let dir = std::path::PathBuf::from(root).join("ToastDesk.v3");
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
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

/// Opens the app that sent a notification through its AppUserModelID.
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

