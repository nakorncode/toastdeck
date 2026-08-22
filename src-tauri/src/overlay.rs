use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, WebviewWindow};

use crate::settings::{AppSettings, OverlayPlacement};

const OVERLAY_WIDTH: f64 = 380.0;
const PAD: f64 = 12.0;
/// Measured from `pnpm overlay:layout-loop` against two solid-sonner cards.
const CARD_HEIGHT: f64 = 74.0;
const GAP: f64 = 12.0;
const MARGIN: f64 = 16.0;

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

    place_overlay(&window, app, settings, toast_count.max(1));
    let _ = window.set_always_on_top(true);
    let _ = window.set_ignore_cursor_events(false);
    let _ = show_without_focus(&window);
}

fn overlay_height(toast_count: usize) -> f64 {
    let count = toast_count.max(1) as f64;
    PAD * 2.0 + count * CARD_HEIGHT + (count - 1.0) * GAP
}

fn place_overlay(window: &WebviewWindow, app: &AppHandle, settings: &AppSettings, toast_count: usize) {
    let Some(monitor) = app.primary_monitor().ok().flatten() else {
        return;
    };
    let work_area = monitor.work_area();
    let scale = monitor.scale_factor().max(0.1);
    let screen_x = work_area.position.x as f64 / scale;
    let screen_y = work_area.position.y as f64 / scale;
    let screen_width = work_area.size.width as f64 / scale;
    let screen_height = work_area.size.height as f64 / scale;
    let width = OVERLAY_WIDTH;
    let max_height = (screen_height - MARGIN * 2.0).max(CARD_HEIGHT + PAD * 2.0);
    let height = overlay_height(toast_count).min(max_height);
    let (x, y) = overlay_position(
        settings.overlay_placement,
        screen_x,
        screen_y,
        screen_width,
        screen_height,
        width,
        height,
        MARGIN,
    );
    let _ = window.set_size(LogicalSize::new(width, height));
    let _ = window.set_position(LogicalPosition::new(x, y));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_height_hugs_two_cards() {
        assert_eq!(overlay_height(2), 184.0);
    }

    #[test]
    fn overlay_height_hugs_one_card() {
        assert_eq!(overlay_height(1), 98.0);
    }
}
