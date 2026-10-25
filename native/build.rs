// Embeds the ToastDesk icon into ToastDeskV3.exe so Explorer, the taskbar and the Start menu show it.
// The icon is the same icon.ico the installers use.

fn main() {
    println!("cargo:rerun-if-changed=icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("icon.ico");
        res.compile().expect("could not embed icon.ico into the executable");
    }
}
