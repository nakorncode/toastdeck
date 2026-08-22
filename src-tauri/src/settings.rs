use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

pub const DEFAULT_SOUND_PRESET: &str = "aosp-argon";

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

    pub fn id(self) -> &'static str {
        match self {
            Self::TopLeft => "topLeft",
            Self::TopCenter => "topCenter",
            Self::TopRight => "topRight",
            Self::MiddleLeft => "middleLeft",
            Self::Center => "center",
            Self::MiddleRight => "middleRight",
            Self::BottomLeft => "bottomLeft",
            Self::BottomCenter => "bottomCenter",
            Self::BottomRight => "bottomRight",
        }
    }

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

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|placement| placement.id() == id)
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
    pub const ALL: [Self; 4] = [
        Self::Seconds10,
        Self::Seconds30,
        Self::Minute,
        Self::Infinite,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::Seconds10 => "10s",
            Self::Seconds30 => "30s",
            Self::Minute => "1min",
            Self::Infinite => "infinite",
        }
    }

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

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|duration| duration.id() == id)
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
        }
    }
}

pub fn settings_path() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(|root| PathBuf::from(root).join("ToastDesk.v2").join("settings.json"))
}

pub fn load_settings() -> AppSettings {
    let Some(path) = settings_path() else {
        return AppSettings::default();
    };
    let Ok(bytes) = fs::read(&path) else {
        return AppSettings::default();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
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
    fn missing_windows_capture_defaults_on() {
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
    }
}
