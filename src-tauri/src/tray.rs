use tauri::menu::{CheckMenuItem, CheckMenuItemBuilder, Menu, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager, Wry};

use crate::capture;
use crate::overlay;
use crate::settings::{
    save_settings, AppSettings, CardDuration, OverlayPlacement, DEFAULT_SOUND_PRESET,
};
use crate::sound::{self, SOUND_PRESETS};
use crate::{apply_startup, emit_state, push_toast, remove_toast, AppState, DEBUG_TOAST_ID, Toast};

pub fn build_tray(app: &AppHandle, settings: &AppSettings) -> tauri::Result<tauri::tray::TrayIcon> {
    let menu = build_menu(app, settings)?;
    let mut tray = TrayIconBuilder::with_id("toastdesk")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .tooltip("ToastDesk")
        .on_menu_event(|app, event| {
            handle_menu_event(app, event.id().as_ref());
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)
}

pub fn refresh_menu(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let menu = build_menu(app, settings).map_err(|error| error.to_string())?;
    let state = app.state::<AppState>();
    let tray = state.tray.lock().map_err(|error| error.to_string())?;
    if let Some(tray) = tray.as_ref() {
        tray.set_menu(Some(menu)).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn build_menu(app: &AppHandle, settings: &AppSettings) -> tauri::Result<Menu<Wry>> {
    let startup = CheckMenuItemBuilder::with_id("startup", "Launch on startup")
        .checked(settings.launch_on_startup)
        .build(app)?;
    let launch_toast = CheckMenuItemBuilder::with_id("launch-toast", "Show toast on launch")
        .checked(settings.show_launch_toast)
        .build(app)?;
    let capture = CheckMenuItemBuilder::with_id("capture", "Capture Windows notifications")
        .checked(settings.windows_capture)
        .build(app)?;
    let capture_retry =
        MenuItemBuilder::with_id("capture-retry", "Retry notification access").build(app)?;
    let debug = CheckMenuItemBuilder::with_id("debug", "Debug overlay")
        .checked(settings.debug_overlay)
        .build(app)?;
    let test_toast = MenuItemBuilder::with_id("test-toast", "Show test toast").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Exit").build(app)?;

    let sound = build_sound_menu(app, settings)?;
    let position = build_position_menu(app, settings)?;
    let duration = build_duration_menu(app, settings)?;

    MenuBuilder::new(app)
        .item(&startup)
        .item(&launch_toast)
        .item(&sound)
        .item(&position)
        .item(&duration)
        .item(&capture)
        .item(&capture_retry)
        .item(&debug)
        .item(&test_toast)
        .item(&quit)
        .build()
}

fn build_sound_menu(app: &AppHandle, settings: &AppSettings) -> tauri::Result<tauri::menu::Submenu<Wry>> {
    let enabled = CheckMenuItemBuilder::with_id("sound-enabled", "Enabled")
        .checked(settings.sound_enabled)
        .build(app)?;
    let presets = build_sound_preset_items(app, settings)?;
    let mut sound = SubmenuBuilder::new(app, "Sound").item(&enabled).separator();
    for preset in &presets {
        sound = sound.item(preset);
    }
    sound.build()
}

fn build_sound_preset_items(
    app: &AppHandle,
    settings: &AppSettings,
) -> tauri::Result<Vec<CheckMenuItem<Wry>>> {
    SOUND_PRESETS
        .iter()
        .map(|preset| {
            CheckMenuItemBuilder::with_id(format!("sound-preset-{}", preset.id), preset.label)
                .checked(settings.sound_preset == preset.id)
                .build(app)
        })
        .collect()
}

fn build_position_menu(
    app: &AppHandle,
    settings: &AppSettings,
) -> tauri::Result<tauri::menu::Submenu<Wry>> {
    let items = OverlayPlacement::ALL
        .into_iter()
        .map(|placement| {
            CheckMenuItemBuilder::with_id(format!("position-{}", placement.id()), placement.label())
                .checked(settings.overlay_placement == placement)
                .build(app)
        })
        .collect::<tauri::Result<Vec<_>>>()?;
    let mut menu = SubmenuBuilder::new(app, "Position");
    for item in &items {
        menu = menu.item(item);
    }
    menu.build()
}

fn build_duration_menu(
    app: &AppHandle,
    settings: &AppSettings,
) -> tauri::Result<tauri::menu::Submenu<Wry>> {
    let items = CardDuration::ALL
        .into_iter()
        .map(|duration| {
            CheckMenuItemBuilder::with_id(format!("duration-{}", duration.id()), duration.label())
                .checked(settings.card_duration == duration)
                .build(app)
        })
        .collect::<tauri::Result<Vec<_>>>()?;
    let mut menu = SubmenuBuilder::new(app, "Duration");
    for item in &items {
        menu = menu.item(item);
    }
    menu.build()
}

fn handle_menu_event(app: &AppHandle, id: &str) {
    match id {
        "quit" => app.exit(0),
        "test-toast" => push_test_toast(app),
        "capture-retry" => capture::request_retry(app),
        "capture" => update_settings(app, |settings| {
            settings.windows_capture = !settings.windows_capture;
        }),
        "startup" => update_settings(app, |settings| {
            settings.launch_on_startup = !settings.launch_on_startup;
        }),
        "launch-toast" => update_settings(app, |settings| {
            settings.show_launch_toast = !settings.show_launch_toast;
        }),
        "sound-enabled" => update_settings(app, |settings| {
            settings.sound_enabled = !settings.sound_enabled;
        }),
        "debug" => update_settings(app, |settings| {
            settings.debug_overlay = !settings.debug_overlay;
        }),
        id if id.starts_with("sound-preset-") => {
            let preset = id.trim_start_matches("sound-preset-");
            if sound::is_known_preset(preset) {
                let preset = preset.to_string();
                update_settings(app, |settings| {
                    settings.sound_preset = preset;
                    settings.sound_enabled = true;
                });
            }
        }
        id if id.starts_with("position-") => {
            if let Some(placement) = OverlayPlacement::from_id(id.trim_start_matches("position-")) {
                update_settings(app, |settings| {
                    settings.overlay_placement = placement;
                });
            }
        }
        id if id.starts_with("duration-") => {
            if let Some(duration) = CardDuration::from_id(id.trim_start_matches("duration-")) {
                update_settings(app, |settings| {
                    settings.card_duration = duration;
                });
            }
        }
        _ => {}
    }
}

fn update_settings(app: &AppHandle, mutate: impl FnOnce(&mut AppSettings)) {
    let state = app.state::<AppState>();
    let mut settings = match state.settings.lock() {
        Ok(settings) => settings,
        Err(_) => return,
    };
    mutate(&mut settings);
    if !sound::is_known_preset(&settings.sound_preset) {
        settings.sound_preset = DEFAULT_SOUND_PRESET.into();
    }
    let snapshot = settings.clone();
    drop(settings);

    let _ = save_settings(&snapshot);
    apply_startup(app, snapshot.launch_on_startup);
    sync_debug_toast(app, snapshot.debug_overlay);
    let _ = refresh_menu(app, &snapshot);
    let toast_count = state.toasts.lock().map(|toasts| toasts.len()).unwrap_or(0);
    overlay::sync_overlay(app, &snapshot, toast_count, crate::overlay_content_height(app));
    emit_state(app);
}

fn sync_debug_toast(app: &AppHandle, debug: bool) {
    if debug {
        push_toast(
            app,
            Toast::overlay(
                DEBUG_TOAST_ID,
                "Debug",
                "Overlay bounds are visible.",
                "debug",
            ),
            false,
        );
    } else {
        remove_toast(app, DEBUG_TOAST_ID);
    }
}

fn push_test_toast(_app: &AppHandle) {
    if let Err(error) = crate::native::show("Test", "ToastDesk overlay is working.") {
        eprintln!("native toast: {error}");
    }
}
