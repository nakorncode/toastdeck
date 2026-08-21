use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

use crate::settings::{AppSettings, OverlayPlacement};

const OVERLAY_WIDTH: f64 = 440.0;
const OVERLAY_MIN_HEIGHT: f64 = 144.0;
const OVERLAY_DEBUG_HEIGHT: f64 = 520.0;
const MARGIN: f64 = 24.0;

pub fn sync_overlay(app: &AppHandle, settings: &AppSettings, toast_count: usize) {
    let Some(window) = app.get_webview_window("toast") else {
        return;
    };
    let should_show = settings.debug_overlay || toast_count > 0;
    if !should_show {
        let _ = window.set_ignore_cursor_events(true);
        let _ = window.hide();
        return;
    }

    place_overlay(app, &window, settings, None);
    let _ = window.set_always_on_top(true);
    let _ = window.set_ignore_cursor_events(false);
    let _ = show_without_focus(&window);
}

pub fn resize_for_content(app: &AppHandle, settings: &AppSettings, content_height: f64) {
    let Some(window) = app.get_webview_window("toast") else {
        return;
    };
    place_overlay(app, &window, settings, Some(content_height));
}

fn place_overlay(
    app: &AppHandle,
    window: &WebviewWindow,
    settings: &AppSettings,
    content_height: Option<f64>,
) {
    let Some(monitor) = app.primary_monitor().ok().flatten() else {
        return;
    };
    let work_area = monitor.work_area();
    let scale = monitor.scale_factor();
    let margin = MARGIN * scale;
    let width = OVERLAY_WIDTH * scale;
    let min_height = if settings.debug_overlay {
        OVERLAY_DEBUG_HEIGHT * scale
    } else {
        OVERLAY_MIN_HEIGHT * scale
    };
    let max_height = (work_area.size.height as f64 - margin * 2.0).max(min_height);
    let height = content_height
        .map(|value| (value * scale).clamp(min_height, max_height))
        .unwrap_or(min_height);
    let (x, y) = overlay_position(
        settings.overlay_placement,
        work_area.position.x as f64,
        work_area.position.y as f64,
        work_area.size.width as f64,
        work_area.size.height as f64,
        width,
        height,
        margin,
    );
    let _ = window.set_size(PhysicalSize::new(width.round() as u32, height.round() as u32));
    let _ = window.set_position(PhysicalPosition::new(x as i32, y as i32));
}

fn overlay_position(
    placement: OverlayPlacement,
    screen_x: f64,
    screen_y: f64,
    screen_width: f64,
    screen_height: f64,
    overlay_width: f64,
    overlay_height: f64,
    margin: f64,
) -> (f64, f64) {
    let left = screen_x + margin;
    let center_x = screen_x + (screen_width - overlay_width) / 2.0;
    let right = screen_x + screen_width - overlay_width - margin;
    let top = screen_y + margin;
    let center_y = screen_y + (screen_height - overlay_height) / 2.0;
    let bottom = screen_y + screen_height - overlay_height - margin;

    match placement {
        OverlayPlacement::TopLeft => (left, top),
        OverlayPlacement::TopCenter => (center_x, top),
        OverlayPlacement::TopRight => (right, top),
        OverlayPlacement::MiddleLeft => (left, center_y),
        OverlayPlacement::Center => (center_x, center_y),
        OverlayPlacement::MiddleRight => (right, center_y),
        OverlayPlacement::BottomLeft => (left, bottom),
        OverlayPlacement::BottomCenter => (center_x, bottom),
        OverlayPlacement::BottomRight => (right, bottom),
    }
}

#[cfg(windows)]
fn show_without_focus(window: &WebviewWindow) -> Result<(), String> {
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, ShowWindow, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
        SWP_SHOWWINDOW, SW_SHOWNOACTIVATE,
    };

    let hwnd = window.hwnd().map_err(|error| error.to_string())?;
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn show_without_focus(window: &WebviewWindow) -> Result<(), String> {
    window.show().map_err(|error| error.to_string())
}
