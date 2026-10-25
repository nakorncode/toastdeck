//! Finds the app icon for a notification source.
//!
//! Windows' `AppInfo.DisplayInfo.GetLogo` returns nothing for classic Win32 apps, so the icon
//! comes from the executable instead. The executable is found through the Start menu shortcut
//! whose target matches the notification's AppUserModelID (for example DeepSeek Harness, Discord,
//! Steam, Helium). When nothing matches, the caller falls back to the bell emoji.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

/// Start menu folders searched for `.lnk` shortcuts, per user and machine-wide.
fn start_menu_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(appdata) = std::env::var_os("APPDATA") {
        roots.push(PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    if let Some(program_data) = std::env::var_os("ProgramData") {
        roots.push(PathBuf::from(program_data).join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    roots
}

/// Collects every `.lnk` shortcut file under the Start menu roots.
fn shortcuts(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            shortcuts(&path, out);
        } else if path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("lnk")) {
            out.push(path);
        }
    }
}

/// Lower-cased file name without extension, used to match an AUMID or display name.
fn normalize(text: &str) -> String {
    text.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase()
}

/// Maps a shortcut name to its target `.exe`, reading each `.lnk` once.
#[cfg(windows)]
fn shortcut_targets() -> Vec<(String, PathBuf)> {
    use windows::{
        core::{Interface, PCWSTR},
        Win32::{
            Storage::FileSystem::WIN32_FIND_DATAW,
            System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, IPersistFile, STGM_READ},
            UI::Shell::{IShellLinkW, ShellLink},
        },
    };

    let mut files = Vec::new();
    for root in start_menu_roots() {
        shortcuts(&root, &mut files);
    }

    let mut out = Vec::new();
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let Ok(link) = CoCreateInstance::<_, IShellLinkW>(&ShellLink, None, CLSCTX_INPROC_SERVER) else {
            return out;
        };
        let Ok(persist) = link.cast::<IPersistFile>() else {
            return out;
        };
        for file in files {
            let wide: Vec<u16> = file.as_os_str().encode_wide_null();
            if persist.Load(PCWSTR(wide.as_ptr()), STGM_READ).is_err() {
                continue;
            }
            let mut buffer = [0u16; 260];
            let mut data = WIN32_FIND_DATAW::default();
            if link.GetPath(&mut buffer, &mut data, 0).is_err() {
                continue;
            }
            let len = buffer.iter().position(|&c| c == 0).unwrap_or(0);
            let target = PathBuf::from(String::from_utf16_lossy(&buffer[..len]));
            if target.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("exe")) && target.is_file() {
                let name = file.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                out.push((name, target));
            }
        }
    }
    out
}

#[cfg(windows)]
trait NullTerminated {
    fn encode_wide_null(&self) -> Vec<u16>;
}

#[cfg(windows)]
impl NullTerminated for std::ffi::OsStr {
    fn encode_wide_null(&self) -> Vec<u16> {
        use std::os::windows::ffi::OsStrExt;
        self.encode_wide().chain(std::iter::once(0)).collect()
    }
}

/// Executable for a source AUMID, or None when no Start menu shortcut matches it.
///
/// Results are cached: the Start menu is scanned once per AUMID.
#[cfg(windows)]
pub fn exe_for_aumid(aumid: &str) -> Option<PathBuf> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<PathBuf>>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(map) = cache.lock() {
        if let Some(found) = map.get(aumid) {
            return found.clone();
        }
    }

    let wanted = normalize(aumid);
    let found = shortcut_targets()
        .into_iter()
        .find(|(name, target)| {
            let stem = target.file_stem().map(|s| normalize(&s.to_string_lossy())).unwrap_or_default();
            let name = normalize(name);
            !wanted.is_empty() && (wanted.contains(&stem) || wanted.contains(&name) || (!stem.is_empty() && stem.contains(&wanted)))
        })
        .map(|(_, target)| target);

    if let Ok(mut map) = cache.lock() {
        map.insert(aumid.to_string(), found.clone());
    }
    found
}

#[cfg(not(windows))]
pub fn exe_for_aumid(_aumid: &str) -> Option<PathBuf> {
    None
}

/// Small (16px) icon of an executable, as a raw HICON value kept for the life of the process.
/// Zero means the executable has no icon. Cached per path so each exe is read only once.
#[cfg(windows)]
pub fn small_icon(exe: &Path) -> isize {
    use windows::{
        core::PCWSTR,
        Win32::UI::{Shell::ExtractIconExW, WindowsAndMessaging::HICON},
    };

    static CACHE: OnceLock<Mutex<HashMap<PathBuf, isize>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(map) = cache.lock() {
        if let Some(handle) = map.get(exe) {
            return *handle;
        }
    }

    let wide: Vec<u16> = std::os::windows::ffi::OsStrExt::encode_wide(exe.as_os_str())
        .chain(std::iter::once(0))
        .collect();
    let mut small = HICON::default();
    let count = unsafe { ExtractIconExW(PCWSTR(wide.as_ptr()), 0, None, Some(&mut small), 1) };
    let handle = if count > 0 && !small.is_invalid() { small.0 as isize } else { 0 };

    if let Ok(mut map) = cache.lock() {
        map.insert(exe.to_path_buf(), handle);
    }
    handle
}
