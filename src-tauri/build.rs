fn main() {
    if std::env::var_os("DEP_TAURI_DEV").is_none() {
        unsafe {
            std::env::set_var("DEP_TAURI_DEV", "false");
        }
    }

    let windows = tauri_build::WindowsAttributes::new().app_manifest(include_str!("app.manifest"));
    let attrs = tauri_build::Attributes::new().windows_attributes(windows);
    tauri_build::try_build(attrs).expect("failed to run tauri-build");
}
