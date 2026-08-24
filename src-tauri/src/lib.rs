mod capture;
mod native;
mod overlay;
mod settings;
mod sound;
mod tray;

use serde::Serialize;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_autostart::ManagerExt;

use overlay as overlay_mod;
use settings::{load_settings, save_settings, AppSettings, OverlayPlacement};
use sound::is_known_preset;

pub const DEBUG_TOAST_ID: &str = "debug-sample";
const MAX_TOASTS: usize = 4;

pub struct AppState {
    pub settings: Mutex<AppSettings>,
    pub toasts: Mutex<Vec<Toast>>,
    pub tray: Mutex<Option<tauri::tray::TrayIcon>>,
    pub captured_ids: Mutex<HashSet<u32>>,
    pub capture_access: Mutex<capture::CaptureAccess>,
    pub capture_retry: AtomicBool,
    pub capture_denied_announced: AtomicBool,
    pub overlay_content_height: Mutex<Option<f64>>,
    pub capture_ready: AtomicBool,
    pub overlay_ready: AtomicBool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Toast {
    pub id: String,
    pub title: String,
    pub body: String,
    pub kind: String,
    pub source_app_user_model_id: Option<String>,
}

impl Toast {
    pub fn overlay(
        id: impl Into<String>,
        title: impl Into<String>,
        body: impl Into<String>,
        kind: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            body: body.into(),
            kind: kind.into(),
            source_app_user_model_id: None,
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverlayState {
    pub settings: AppSettings,
    pub toasts: Vec<Toast>,
    pub sonner_position: &'static str,
    pub duration_ms: Option<u64>,
}

fn snapshot(app: &AppHandle) -> OverlayState {
    let state = app.state::<AppState>();
    let settings = state
        .settings
        .lock()
        .map(|settings| settings.clone())
        .unwrap_or_default();
    let toasts = state.toasts.lock().map(|toasts| toasts.clone()).unwrap_or_default();
    OverlayState {
        sonner_position: sonner_position(settings.overlay_placement),
        duration_ms: settings.card_duration.millis(),
        settings,
        toasts,
    }
}

fn sonner_position(placement: OverlayPlacement) -> &'static str {
    match placement {
        OverlayPlacement::TopLeft | OverlayPlacement::MiddleLeft => "top-left",
        OverlayPlacement::TopCenter | OverlayPlacement::Center => "top-center",
        OverlayPlacement::TopRight | OverlayPlacement::MiddleRight => "top-right",
        OverlayPlacement::BottomLeft => "bottom-left",
        OverlayPlacement::BottomCenter => "bottom-center",
        OverlayPlacement::BottomRight => "bottom-right",
    }
}

fn emit_state(app: &AppHandle) {
    let _ = app.emit_to("toast", "toastdesk://state", snapshot(app));
}

fn apply_startup(app: &AppHandle, enabled: bool) {
    let autostart = app.autolaunch();
    let result = if enabled {
        autostart.enable()
    } else {
        autostart.disable()
    };
    if let Err(error) = result {
        eprintln!("autostart: {error}");
    }
}

pub(crate) fn push_toast(app: &AppHandle, toast: Toast, play_sound: bool) {
    let state = app.state::<AppState>();
    if let Ok(mut toasts) = state.toasts.lock() {
        toasts.retain(|item| item.id != toast.id);
        toasts.insert(0, toast);
        if toasts.len() > MAX_TOASTS {
            if let Some(index) = toasts.iter().rposition(|item| item.id != DEBUG_TOAST_ID) {
                toasts.remove(index);
            } else {
                toasts.pop();
            }
        }
    }
    if play_sound {
        if let Ok(settings) = state.settings.lock() {
            if settings.sound_enabled {
                let _ = sound::play(app, &settings.sound_preset);
            }
        }
    }
    let settings = state
        .settings
        .lock()
        .map(|settings| settings.clone())
        .unwrap_or_default();
    let count = state.toasts.lock().map(|toasts| toasts.len()).unwrap_or(0);
    overlay_mod::sync_overlay(app, &settings, count, overlay_content_height(app));
    emit_state(app);
}

pub(crate) fn overlay_content_height(app: &AppHandle) -> Option<f64> {
    app.state::<AppState>()
        .overlay_content_height
        .lock()
        .ok()
        .and_then(|height| *height)
}

pub(crate) fn remove_toast(app: &AppHandle, id: &str) {
    let state = app.state::<AppState>();
    if let Ok(mut toasts) = state.toasts.lock() {
        toasts.retain(|toast| toast.id != id);
    }
    let settings = state
        .settings
        .lock()
        .map(|settings| settings.clone())
        .unwrap_or_default();
    let count = state.toasts.lock().map(|toasts| toasts.len()).unwrap_or(0);
    if count == 0 {
        if let Ok(mut height) = state.overlay_content_height.lock() {
            *height = None;
        }
    }
    overlay_mod::sync_overlay(app, &settings, count, overlay_content_height(app));
    emit_state(app);
}

fn open_captured_toast(app: &AppHandle, id: &str) {
    if id == DEBUG_TOAST_ID || id == capture::CAPTURE_DENIED_ID {
        return;
    }
    let aumid = {
        let state = app.state::<AppState>();
        state.toasts.lock().ok().and_then(|toasts| {
            toasts
                .iter()
                .find(|toast| toast.id == id)
                .and_then(|toast| toast.source_app_user_model_id.clone())
        })
    };
    if let Some(aumid) = aumid {
        if let Err(error) = capture::open_source_app(&aumid) {
            eprintln!("open toast: {error}");
        }
    }
    remove_toast(app, id);
}

#[tauri::command]
fn get_overlay_state(app: AppHandle) -> OverlayState {
    snapshot(&app)
}

#[tauri::command]
fn dismiss_toast(app: AppHandle, id: String) {
    remove_toast(&app, &id);
}

#[tauri::command]
fn open_toast(app: AppHandle, id: String) {
    open_captured_toast(&app, &id);
}

#[tauri::command]
fn report_overlay_height(app: AppHandle, height: f64) {
    if let Ok(mut current) = app.state::<AppState>().overlay_content_height.lock() {
        *current = (height.is_finite() && height > 0.0).then_some(height);
    }
    let settings = app
        .state::<AppState>()
        .settings
        .lock()
        .map(|settings| settings.clone())
        .unwrap_or_default();
    let count = app
        .state::<AppState>()
        .toasts
        .lock()
        .map(|toasts| toasts.len())
        .unwrap_or(0);
    overlay_mod::sync_overlay(&app, &settings, count, overlay_content_height(&app));
}

#[tauri::command]
fn mark_overlay_ready(app: AppHandle) {
    app.state::<AppState>()
        .overlay_ready
        .store(true, Ordering::SeqCst);
}

fn schedule_launch_toast(app: &AppHandle, enabled: bool) {
    if !enabled {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            let state = app.state::<AppState>();
            if state.capture_ready.load(Ordering::SeqCst)
                && state.overlay_ready.load(Ordering::SeqCst)
            {
                break;
            }
            drop(state);
            std::thread::sleep(Duration::from_millis(50));
        }
        std::thread::sleep(Duration::from_millis(150));
        show_launch_toast(&app);
    });
}

fn show_launch_toast(app: &AppHandle) {
    const TITLE: &str = "ToastDesk";
    const BODY: &str = "ToastDesk has started.";
    let (tx, rx) = std::sync::mpsc::channel();
    let _ = app.run_on_main_thread(move || {
        let _ = tx.send(native::show(TITLE, BODY));
    });
    match rx.recv() {
        Ok(Err(error)) => eprintln!("launch toast: {error}"),
        Err(error) => eprintln!("launch toast: {error}"),
        Ok(Ok(())) => {}
    }
    if capture_will_display(app) {
        return;
    }
    push_toast(app, Toast::overlay("launch", TITLE, BODY, "test"), true);
}

fn capture_will_display(app: &AppHandle) -> bool {
    let enabled = app
        .state::<AppState>()
        .settings
        .lock()
        .map(|settings| settings.windows_capture)
        .unwrap_or(true);
    let access = app
        .state::<AppState>()
        .capture_access
        .lock()
        .map(|access| *access)
        .unwrap_or(capture::CaptureAccess::Disabled);
    enabled && access == capture::CaptureAccess::Allowed
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {}))
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .app_name("ToastDesk.v2")
                .build(),
        )
        .manage(AppState {
            settings: Mutex::new(AppSettings::default()),
            toasts: Mutex::new(Vec::new()),
            tray: Mutex::new(None),
            captured_ids: Mutex::new(HashSet::new()),
            capture_access: Mutex::new(capture::CaptureAccess::Disabled),
            capture_retry: AtomicBool::new(false),
            capture_denied_announced: AtomicBool::new(false),
            overlay_content_height: Mutex::new(None),
            capture_ready: AtomicBool::new(false),
            overlay_ready: AtomicBool::new(false),
        })
        .invoke_handler(tauri::generate_handler![
            get_overlay_state,
            dismiss_toast,
            open_toast,
            report_overlay_height,
            mark_overlay_ready
        ])
        .setup(|app| {
            let mut settings = load_settings();
            if !is_known_preset(&settings.sound_preset) {
                settings.sound_preset = settings::DEFAULT_SOUND_PRESET.into();
            }
            let _ = save_settings(&settings);
            apply_startup(app.handle(), settings.launch_on_startup);
            native::prepare();

            let state = app.state::<AppState>();
            if let Ok(mut current) = state.settings.lock() {
                *current = settings.clone();
            }

            let tray_icon = tray::build_tray(app.handle(), &settings)?;
            if let Ok(mut tray) = state.tray.lock() {
                *tray = Some(tray_icon);
            }

            if settings.debug_overlay {
                push_toast(
                    app.handle(),
                    Toast::overlay(
                        DEBUG_TOAST_ID,
                        "Debug",
                        "Overlay bounds are visible.",
                        "debug",
                    ),
                    false,
                );
            } else {
                overlay_mod::sync_overlay(app.handle(), &settings, 0, None);
            }

            capture::start(app.handle().clone());
            schedule_launch_toast(app.handle(), settings.show_launch_toast);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
