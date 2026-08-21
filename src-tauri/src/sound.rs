use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::settings::DEFAULT_SOUND_PRESET;

#[derive(Clone, Copy)]
pub struct SoundPreset {
    pub id: &'static str,
    pub label: &'static str,
    pub file: &'static str,
}

pub const SOUND_PRESETS: &[SoundPreset] = &[
    SoundPreset {
        id: "aosp-acrux",
        label: "Acrux",
        file: "aosp-acrux.wav",
    },
    SoundPreset {
        id: "aosp-adara",
        label: "Adara",
        file: "aosp-adara.wav",
    },
    SoundPreset {
        id: "aosp-altair",
        label: "Altair",
        file: "aosp-altair.wav",
    },
    SoundPreset {
        id: "aosp-alya",
        label: "Alya",
        file: "aosp-alya.wav",
    },
    SoundPreset {
        id: "aosp-antares",
        label: "Antares",
        file: "aosp-antares.wav",
    },
    SoundPreset {
        id: DEFAULT_SOUND_PRESET,
        label: "Argon",
        file: "aosp-argon.wav",
    },
    SoundPreset {
        id: "aosp-capella",
        label: "Capella",
        file: "aosp-capella.wav",
    },
    SoundPreset {
        id: "aosp-vega",
        label: "Vega",
        file: "aosp-vega.wav",
    },
];

pub fn preset(id: &str) -> Option<&'static SoundPreset> {
    SOUND_PRESETS.iter().find(|preset| preset.id == id)
}

pub fn is_known_preset(id: &str) -> bool {
    preset(id).is_some()
}

#[cfg(windows)]
pub fn play(app: &AppHandle, preset_id: &str) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_FILENAME, SND_NODEFAULT};

    let file = preset(preset_id)
        .or_else(|| preset(DEFAULT_SOUND_PRESET))
        .map(|preset| preset.file)
        .unwrap_or("aosp-argon.wav");
    let path = sound_path(app, file).ok_or_else(|| format!("Sound file not found: {file}"))?;
    let mut wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let played = unsafe {
        PlaySoundW(
            PCWSTR(wide.as_mut_ptr()),
            None,
            SND_FILENAME | SND_ASYNC | SND_NODEFAULT,
        )
        .as_bool()
    };
    if played {
        Ok(())
    } else {
        Err(format!("Windows could not play {}", path.display()))
    }
}

#[cfg(not(windows))]
pub fn play(_app: &AppHandle, _preset_id: &str) -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
fn sound_path(app: &AppHandle, file: &str) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join(file));
        candidates.push(resource_dir.join("assets").join("sounds").join(file));
        candidates.push(
            resource_dir
                .join("_up_")
                .join("assets")
                .join("sounds")
                .join(file),
        );
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            candidates.push(exe_dir.join("assets").join("sounds").join(file));
        }
    }
    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join("assets").join("sounds").join(file));
    }
    candidates.into_iter().find(|path| path.is_file())
}
