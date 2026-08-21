const AUMID: &str = "NakornCode.ToastDesk.v2";

pub fn prepare() {
    #[cfg(windows)]
    {
        let _ = register_aumid();
        let _ = set_process_aumid();
    }
}

#[cfg(windows)]
pub fn show(title: &str, body: &str) -> Result<(), String> {
    use tauri_winrt_notification::{Duration, Toast};

    register_aumid()?;
    set_process_aumid()?;

    Toast::new(AUMID)
        .title(title)
        .text1(body)
        .duration(Duration::Short)
        .sound(None)
        .show()
        .map_err(|error| format!("Could not send WinRT notification: {error}"))
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
    if let Ok(exe) = std::env::current_exe() {
        let _ = key.set_value("IconUri", &exe.to_string_lossy().to_string());
    }
    Ok(())
}

#[cfg(windows)]
fn set_process_aumid() -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::SetCurrentProcessExplicitAppUserModelID;

    let mut wide: Vec<u16> = std::ffi::OsStr::new(AUMID)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    unsafe { SetCurrentProcessExplicitAppUserModelID(PCWSTR(wide.as_mut_ptr())) }
        .map_err(|error| format!("Could not set process AppUserModelID: {error}"))
}

#[cfg(not(windows))]
pub fn show(_title: &str, _body: &str) -> Result<(), String> {
    Ok(())
}
