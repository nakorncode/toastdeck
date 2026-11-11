//! Windows integration: identity, single instance, startup entry, process priority,
//! and WinRT toast notifications for the test and launch messages.

use std::path::{Path, PathBuf};

pub const AUMID: &str = "NakornCode.ToastDesk.v3";
pub const APP_NAME: &str = "ToastDesk.v3";

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

/// Name of the Startup-folder shortcut. It is the second, registry-free autostart path: if the Run
/// entry is removed (MSI component removal, a cleaner, an Explorer refresh) the app still starts, and
/// the next run repairs the Run entry again.
const STARTUP_SHORTCUT: &str = "ToastDesk v3.lnk";

/// The command written to the Run entry: the quoted executable path.
fn run_value_for(exe: &Path) -> String {
    format!("\"{}\"", exe.display())
}

fn exe_path() -> Option<PathBuf> {
    std::env::current_exe().ok()
}

/// The per-user Startup folder, `%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup`.
fn startup_folder() -> Option<PathBuf> {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|root| root.join(r"Microsoft\Windows\Start Menu\Programs\Startup"))
}

fn shortcut_path() -> Option<PathBuf> {
    startup_folder().map(|folder| folder.join(STARTUP_SHORTCUT))
}

/// Compares two paths the way Windows does, ignoring case.
fn same_path(a: &Path, b: &Path) -> bool {
    a.to_string_lossy().eq_ignore_ascii_case(&b.to_string_lossy())
}

#[cfg(windows)]
fn wide(text: &std::ffi::OsStr) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    text.encode_wide().chain(std::iter::once(0)).collect()
}

#[cfg(windows)]
pub fn set_process_identity() {
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;

    let mut app_id = wide(std::ffi::OsStr::new(AUMID));
    unsafe {
        let _ = SetCurrentProcessExplicitAppUserModelID(PCWSTR(app_id.as_mut_ptr()));
    }
}

/// Keeps one copy running. Returns false when another instance already holds the mutex.
#[cfg(windows)]
pub fn acquire_single_instance() -> bool {
    use windows::core::w;
    use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows::Win32::System::Threading::CreateMutexW;

    let handle = unsafe { CreateMutexW(None, true, w!("Local\\NakornCode.ToastDesk.v3")) };
    // The handle stays open for the life of the process so the mutex is held.
    std::mem::forget(handle);
    unsafe { GetLastError() != ERROR_ALREADY_EXISTS }
}

#[cfg(windows)]
pub fn apply_priority(enabled: bool) {
    use windows::Win32::System::Threading::{
        GetCurrentProcess, SetPriorityClass, ABOVE_NORMAL_PRIORITY_CLASS, HIGH_PRIORITY_CLASS,
        NORMAL_PRIORITY_CLASS,
    };

    let process = unsafe { GetCurrentProcess() };
    let class = if enabled { HIGH_PRIORITY_CLASS } else { NORMAL_PRIORITY_CLASS };
    if unsafe { SetPriorityClass(process, class) }.is_ok() {
        return;
    }
    // HIGH_PRIORITY_CLASS needs SeIncreaseBasePriorityPrivilege, which standard accounts may lack.
    if enabled {
        if let Err(error) = unsafe { SetPriorityClass(process, ABOVE_NORMAL_PRIORITY_CLASS) } {
            crate::platform::log(&format!("priority: {error}"));
        }
    }
}

/// Current Run entry value, `Ok(None)` when the value does not exist.
#[cfg(windows)]
fn read_run_entry() -> Result<Option<String>, String> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    let run = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(RUN_KEY)
        .map_err(|error| format!("could not open the Run key: {error}"))?;
    match run.get_value::<String, _>(APP_NAME) {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("could not read the Run value: {error}")),
    }
}

#[cfg(windows)]
fn write_run_entry(value: &str) -> Result<(), String> {
    use winreg::{enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE}, RegKey};

    let (run, _) = RegKey::predef(HKEY_CURRENT_USER)
        .create_subkey_with_flags(RUN_KEY, KEY_READ | KEY_WRITE)
        .map_err(|error| format!("could not open the Run key: {error}"))?;
    run.set_value(APP_NAME, &value)
        .map_err(|error| format!("could not write the Run value: {error}"))
}

#[cfg(windows)]
fn delete_run_entry() -> Result<(), String> {
    use winreg::{enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE}, RegKey};

    let Ok(run) = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey_with_flags(RUN_KEY, KEY_READ | KEY_WRITE)
    else {
        return Ok(());
    };
    match run.delete_value(APP_NAME) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("could not remove the Run value: {error}")),
    }
}

/// Target of an existing `.lnk`, if it can be read.
#[cfg(windows)]
fn read_shortcut_target(path: &Path) -> Option<PathBuf> {
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::Storage::FileSystem::WIN32_FIND_DATAW;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, IPersistFile, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, STGM_READ,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        let persist: IPersistFile = link.cast().ok()?;
        let file = wide(path.as_os_str());
        persist.Load(PCWSTR(file.as_ptr()), STGM_READ).ok()?;

        let mut buffer = [0u16; 1024];
        let mut data = WIN32_FIND_DATAW::default();
        link.GetPath(&mut buffer, &mut data, 0).ok()?;
        let len = buffer.iter().position(|&c| c == 0).unwrap_or(0);
        Some(PathBuf::from(String::from_utf16_lossy(&buffer[..len])))
    }
}

/// Creates or replaces the Startup-folder shortcut so it points at `exe`.
#[cfg(windows)]
fn write_shortcut(path: &Path, exe: &Path) -> Result<(), String> {
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, IPersistFile, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

    if let Some(folder) = path.parent() {
        std::fs::create_dir_all(folder).map_err(|error| format!("could not create the Startup folder: {error}"))?;
    }
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
            .map_err(|error| format!("could not open a shell link: {error}"))?;
        let exe_wide = wide(exe.as_os_str());
        link.SetPath(PCWSTR(exe_wide.as_ptr()))
            .map_err(|error| format!("could not set the shortcut target: {error}"))?;
        if let Some(folder) = exe.parent() {
            let folder_wide = wide(folder.as_os_str());
            let _ = link.SetWorkingDirectory(PCWSTR(folder_wide.as_ptr()));
        }
        let description = wide(std::ffi::OsStr::new("ToastDesk v3"));
        let _ = link.SetDescription(PCWSTR(description.as_ptr()));
        let _ = link.SetIconLocation(PCWSTR(exe_wide.as_ptr()), 0);

        let persist: IPersistFile = link
            .cast()
            .map_err(|error| format!("could not save the shortcut: {error}"))?;
        let file = wide(path.as_os_str());
        persist
            .Save(PCWSTR(file.as_ptr()), true)
            .map_err(|error| format!("could not save the shortcut: {error}"))?;
    }
    Ok(())
}

#[cfg(windows)]
fn remove_shortcut() -> Result<(), String> {
    let Some(path) = shortcut_path() else {
        return Ok(());
    };
    match std::fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("could not remove the Startup shortcut: {error}")),
    }
}

/// Makes the Run entry name `exe`. Returns a short status for the log line.
#[cfg(windows)]
fn sync_run_entry(exe: &Path) -> String {
    let wanted = run_value_for(exe);
    match read_run_entry() {
        Err(error) => format!("failed: {error}"),
        Ok(Some(current)) if current == wanted => "ok".into(),
        Ok(Some(current)) => match write_run_entry(&wanted) {
            Ok(()) => format!("repaired, was {current}"),
            Err(error) => format!("failed: {error}"),
        },
        Ok(None) => match write_run_entry(&wanted) {
            Ok(()) => "created".into(),
            Err(error) => format!("failed: {error}"),
        },
    }
}

/// Makes the Startup shortcut point at `exe`. Returns a short status for the log line.
#[cfg(windows)]
fn sync_shortcut(exe: &Path) -> String {
    let Some(path) = shortcut_path() else {
        return "failed: APPDATA is not set".into();
    };
    match read_shortcut_target(&path) {
        Some(target) if same_path(&target, exe) => "ok".into(),
        Some(target) => match write_shortcut(&path, exe) {
            Ok(()) => format!("repaired, was {}", target.display()),
            Err(error) => format!("failed: {error}"),
        },
        None => match write_shortcut(&path, exe) {
            Ok(()) => "created".into(),
            Err(error) => format!("failed: {error}"),
        },
    }
}

/// Writes or removes both autostart entries. Two independent mechanisms are kept on purpose: a Run
/// entry and a Startup-folder shortcut. When one of them is stripped the other still starts the app,
/// and that start repairs the stripped one. The log line says what was found and what was changed, so
/// a future failure leaves evidence instead of silence.
#[cfg(windows)]
pub fn apply_startup(enabled: bool) {
    let Some(exe) = exe_path() else {
        crate::platform::log("startup: could not resolve the executable path");
        return;
    };
    if enabled {
        let run = sync_run_entry(&exe);
        let shortcut = sync_shortcut(&exe);
        crate::platform::log(&format!("startup: run={run} shortcut={shortcut}"));
        return;
    }
    if let Err(error) = delete_run_entry() {
        crate::platform::log(&format!("startup: {error}"));
    }
    if let Err(error) = remove_shortcut() {
        crate::platform::log(&format!("startup: {error}"));
    }
}

/// Removes both autostart entries without touching anything else. Used by the uninstallers, which
/// must clear them even when the saved setting says autostart is on.
#[cfg(windows)]
pub fn remove_autostart() {
    if let Err(error) = delete_run_entry() {
        crate::platform::log(&format!("startup: {error}"));
    }
    if let Err(error) = remove_shortcut() {
        crate::platform::log(&format!("startup: {error}"));
    }
}

/// Registers ToastDesk's AppUserModelID display name and icon, so Windows shows the right icon for
/// notifications that name this app. The icon is the icon.png next to the running exe.
#[cfg(windows)]
pub fn register_aumid() -> Result<(), String> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(format!(r"SOFTWARE\Classes\AppUserModelId\{AUMID}"))
        .map_err(|error| format!("Could not register AppUserModelId: {error}"))?;
    key.set_value("DisplayName", &"ToastDesk")
        .map_err(|error| format!("Could not write notification display name: {error}"))?;
    key.set_value("IconBackgroundColor", &"0")
        .map_err(|error| format!("Could not write notification icon color: {error}"))?;
    // Point the icon at the icon.png next to the running exe, so an installed copy finds its own icon
    // and an older path from another build is replaced.
    if let Some(dir) = std::env::current_exe().ok().and_then(|exe| exe.parent().map(|dir| dir.to_path_buf())) {
        let icon = dir.join("icon.png");
        if icon.is_file() {
            let uri = format!("file:///{}", icon.display().to_string().replace('\\', "/"));
            key.set_value("IconUri", &uri)
                .map_err(|error| format!("Could not write notification icon: {error}"))?;
        }
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn set_process_identity() {}

#[cfg(not(windows))]
pub fn register_aumid() -> Result<(), String> {
    Ok(())
}

#[cfg(not(windows))]
pub fn acquire_single_instance() -> bool {
    true
}

#[cfg(not(windows))]
pub fn apply_priority(_enabled: bool) {}

#[cfg(not(windows))]
pub fn apply_startup(_enabled: bool) {}

#[cfg(not(windows))]
pub fn remove_autostart() {}

/// Appends a line to `%LOCALAPPDATA%\ToastDesk.v3\app.log`. The release build has no console,
/// so errors that would otherwise go to stderr are written here.
pub fn log(message: &str) {
    use std::io::Write;

    let Some(root) = std::env::var_os("LOCALAPPDATA") else {
        return;
    };
    let dir = std::path::PathBuf::from(root).join("ToastDesk.v3");
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs())
        .unwrap_or(0);
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("app.log")) {
        let _ = writeln!(file, "{stamp} {message}");
    }
}


const TOAST_SETTING_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Notifications\Settings";
const TOAST_SETTING_VALUE: &str = "NOC_GLOBAL_SETTING_TOASTS_ENABLED";

/// Current value of the "Get notifications from apps" banner switch. None when Windows has no value
/// (it is then on by default).
#[cfg(windows)]
pub fn read_windows_toasts() -> Option<i64> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey(TOAST_SETTING_KEY).ok()?;
    key.get_value::<u32, _>(TOAST_SETTING_VALUE).ok().map(i64::from)
}

/// Writes the banner switch, or removes it when `value` is -1 (restoring "no value").
#[cfg(windows)]
pub fn write_windows_toasts(value: i64) -> Result<(), String> {
    use winreg::{enums::{HKEY_CURRENT_USER, KEY_READ, KEY_WRITE}, RegKey};

    let (key, _) = RegKey::predef(HKEY_CURRENT_USER)
        .create_subkey_with_flags(TOAST_SETTING_KEY, KEY_READ | KEY_WRITE)
        .map_err(|error| format!("Could not open notification settings: {error}"))?;
    if value < 0 {
        let _ = key.delete_value(TOAST_SETTING_VALUE);
        return Ok(());
    }
    key.set_value(TOAST_SETTING_VALUE, &(value as u32))
        .map_err(|error| format!("Could not write banner setting: {error}"))
}

#[cfg(not(windows))]
pub fn read_windows_toasts() -> Option<i64> {
    None
}

#[cfg(not(windows))]
pub fn write_windows_toasts(_value: i64) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_value_quotes_the_executable_path() {
        assert_eq!(
            run_value_for(Path::new(r"C:\Users\test\AppData\Local\ToastDesk.v3\ToastDeskV3.exe")),
            "\"C:\\Users\\test\\AppData\\Local\\ToastDesk.v3\\ToastDeskV3.exe\""
        );
    }

    #[test]
    fn same_path_ignores_case() {
        assert!(same_path(Path::new(r"C:\Apps\ToastDeskV3.exe"), Path::new(r"c:\apps\toastdeskv3.EXE")));
        assert!(!same_path(Path::new(r"C:\Apps\ToastDeskV3.exe"), Path::new(r"C:\Other\ToastDeskV3.exe")));
    }

    #[test]
    fn shortcut_path_sits_in_the_startup_folder() {
        // %APPDATA% may be unset outside a desktop session; both outcomes are valid.
        match shortcut_path() {
            Some(path) => {
                assert!(path.ends_with(STARTUP_SHORTCUT));
                assert!(path.to_string_lossy().contains("Start Menu"));
            }
            None => assert!(std::env::var_os("APPDATA").is_none()),
        }
    }
}
