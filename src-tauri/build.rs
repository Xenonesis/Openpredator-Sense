fn main() {
    if std::env::var_os("DEP_TAURI_DEV").is_none() {
        unsafe {
            std::env::set_var("DEP_TAURI_DEV", "false");
        }
    }

    tauri_build::build()
}
