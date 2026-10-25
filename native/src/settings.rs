//! Settings kept in `%LOCALAPPDATA%\ToastDesk.v3\settings.json`.
//! Field names match v2 so a copied file reads without changes.

use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

pub const DEFAULT_SOUND_PRESET: &str = "aosp-argon";
const APP_FOLDER: &str = "ToastDesk.v3";
const LEGACY_FOLDER: &str = "ToastDesk.v2";

fn default_true() -> bool {
    true
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OverlayPlacement {
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    Center,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

impl OverlayPlacement {
    pub const ALL: [Self; 9] = [
        Self::TopLeft,
        Self::TopCenter,
        Self::TopRight,
        Self::MiddleLeft,
        Self::Center,
        Self::MiddleRight,
        Self::BottomLeft,
        Self::BottomCenter,
        Self::BottomRight,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::TopLeft => "Top left",
            Self::TopCenter => "Top center",
            Self::TopRight => "Top right",
            Self::MiddleLeft => "Middle left",
            Self::Center => "Center",
            Self::MiddleRight => "Middle right",
            Self::BottomLeft => "Bottom left",
            Self::BottomCenter => "Bottom center",
            Self::BottomRight => "Bottom right",
        }
    }


    pub fn from_bottom(self) -> bool {
        matches!(self, Self::BottomLeft | Self::BottomCenter | Self::BottomRight)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CardDuration {
    Seconds10,
    Seconds30,
    Minute,
    Infinite,
}

impl CardDuration {
    pub const ALL: [Self; 4] = [Self::Seconds10, Self::Seconds30, Self::Minute, Self::Infinite];

    pub fn label(self) -> &'static str {
        match self {
            Self::Seconds10 => "10 seconds",
            Self::Seconds30 => "30 seconds",
            Self::Minute => "1 minute",
            Self::Infinite => "Infinite",
        }
    }

    pub fn millis(self) -> Option<u64> {
        match self {
            Self::Seconds10 => Some(10_000),
            Self::Seconds30 => Some(30_000),
            Self::Minute => Some(60_000),
            Self::Infinite => None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub launch_on_startup: bool,
    pub sound_enabled: bool,
    pub sound_preset: String,
    pub overlay_placement: OverlayPlacement,
    pub card_duration: CardDuration,
    pub debug_overlay: bool,
    #[serde(default = "default_true")]
    pub windows_capture: bool,
    #[serde(default = "default_true")]
    pub show_launch_toast: bool,
    #[serde(default = "default_true")]
    pub high_priority: bool,
    /// Hide Windows toast banners while ToastDesk runs. Notification Center keeps working.
    #[serde(default = "default_true")]
    pub suppress_windows_toasts: bool,
    /// The banner value found before we changed it: -1 when the value did not exist.
    #[serde(default)]
    pub saved_windows_toast_setting: Option<i64>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            launch_on_startup: true,
            sound_enabled: true,
            sound_preset: DEFAULT_SOUND_PRESET.into(),
            overlay_placement: OverlayPlacement::TopRight,
            card_duration: CardDuration::Infinite,
            debug_overlay: false,
            windows_capture: true,
            show_launch_toast: true,
            high_priority: true,
            suppress_windows_toasts: true,
            saved_windows_toast_setting: None,
        }
    }
}

fn local_app_data() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
}

pub fn settings_path() -> Option<PathBuf> {
    local_app_data().map(|root| root.join(APP_FOLDER).join("settings.json"))
}

fn legacy_settings_path() -> Option<PathBuf> {
    local_app_data().map(|root| root.join(LEGACY_FOLDER).join("settings.json"))
}

/// Loads v3 settings. On the first run, copies the v2 file once and never writes to v2.
/// A file that cannot be parsed is kept as `settings.json.bad` before defaults are used.
pub fn load_settings() -> AppSettings {
    let Some(path) = settings_path() else {
        return AppSettings::default();
    };
    if !path.exists() {
        if let Some(legacy) = legacy_settings_path().filter(|legacy| legacy.is_file()) {
            if let Some(dir) = path.parent() {
                let _ = fs::create_dir_all(dir);
            }
            let _ = fs::copy(&legacy, &path);
        }
    }
    let Ok(bytes) = fs::read(&path) else {
        return AppSettings::default();
    };
    match parse_settings(&bytes) {
        Ok(settings) => settings,
        Err(_) => {
            let _ = fs::rename(&path, unused_bad_path(&path));
            AppSettings::default()
        }
    }
}

/// Parses settings bytes. Errors leave the caller to back up the broken file.
pub fn parse_settings(bytes: &[u8]) -> Result<AppSettings, serde_json::Error> {
    serde_json::from_slice(bytes)
}

/// First free `settings.json.bad`, `settings.json.bad.1`, ... so an older backup is never overwritten.
pub fn unused_bad_path(path: &std::path::Path) -> PathBuf {
    let base = path.with_extension("json.bad");
    if !base.exists() {
        return base;
    }
    (1..)
        .map(|n| path.with_extension(format!("json.bad.{n}")))
        .find(|candidate| !candidate.exists())
        .unwrap_or(base)
}

pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path().ok_or("LOCALAPPDATA is not set")?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(settings).map_err(|error| error.to_string())?;
    fs::write(path, bytes).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_default_on() {
        let json = r#"{
            "launchOnStartup": true,
            "soundEnabled": true,
            "soundPreset": "aosp-argon",
            "overlayPlacement": "topRight",
            "cardDuration": "infinite",
            "debugOverlay": false
        }"#;
        let settings: AppSettings = serde_json::from_str(json).expect("legacy settings");
        assert!(settings.windows_capture);
        assert!(settings.show_launch_toast);
        assert!(settings.high_priority);
    }

    #[test]
    fn broken_json_is_rejected() {
        assert!(parse_settings(b"{ not json").is_err());
    }

    #[test]
    fn bad_backup_never_overwrites_an_older_one() {
        let dir = std::env::temp_dir().join(format!("toastdesk-bad-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("settings.json");
        let first = unused_bad_path(&path);
        fs::write(&first, b"old backup").unwrap();
        let second = unused_bad_path(&path);
        assert_ne!(first, second);
        assert!(!second.exists());
        let _ = fs::remove_dir_all(&dir);
    }
}
