//! Windows integration: identity, single instance, startup entry, process priority,
//! and WinRT toast notifications for the test and launch messages.

pub const AUMID: &str = "NakornCode.ToastDesk.v3";
pub const APP_NAME: &str = "ToastDesk.v3";

#[cfg(windows)]
pub fn set_process_identity() {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;

    let mut wide: Vec<u16> = std::ffi::OsStr::new(AUMID).encode_wide().chain(std::iter::once(0)).collect();
    unsafe {
        let _ = SetCurrentProcessExplicitAppUserModelID(PCWSTR(wide.as_mut_ptr()));
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
            eprintln!("priority: {error}");
        }
    }
}

/// Writes or removes the per-user Run entry used for launch on startup.
#[cfg(windows)]
pub fn apply_startup(enabled: bool) {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    let run = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(
        r"Software\Microsoft\Windows\CurrentVersion\Run",
        winreg::enums::KEY_READ | winreg::enums::KEY_WRITE,
    );
    let Ok(run) = run else {
        eprintln!("startup: could not open Run key");
        return;
    };
    if enabled {
        if let Ok(exe) = std::env::current_exe() {
            let value = format!("\"{}\"", exe.display());
            if let Err(error) = run.set_value(APP_NAME, &value) {
                eprintln!("startup: {error}");
            }
        }
    } else {
        let _ = run.delete_value(APP_NAME);
    }
}

#[cfg(windows)]
fn register_aumid() -> Result<(), String> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(format!(r"SOFTWARE\Classes\AppUserModelId\{AUMID}"))
        .map_err(|error| format!("Could not register AppUserModelId: {error}"))?;
    key.set_value("DisplayName", &"ToastDesk")
        .map_err(|error| format!("Could not write notification display name: {error}"))?;
    key.set_value("IconBackgroundColor", &"0")
        .map_err(|error| format!("Could not write notification icon color: {error}"))?;
    Ok(())
}

/// Shows a plain WinRT toast without sound.
#[cfg(windows)]
pub fn show_native_toast(title: &str, body: &str) -> Result<(), String> {
    use tauri_winrt_notification::{Duration, Toast};

    register_aumid()?;
    Toast::new(AUMID)
        .title(title)
        .text1(body)
        .duration(Duration::Long)
        .sound(None)
        .show()
        .map_err(|error| format!("Could not send WinRT notification: {error}"))
}

#[cfg(not(windows))]
pub fn set_process_identity() {}

#[cfg(not(windows))]
pub fn acquire_single_instance() -> bool {
    true
}

#[cfg(not(windows))]
pub fn apply_priority(_enabled: bool) {}

#[cfg(not(windows))]
pub fn apply_startup(_enabled: bool) {}

#[cfg(not(windows))]
pub fn show_native_toast(_title: &str, _body: &str) -> Result<(), String> {
    Ok(())
}
