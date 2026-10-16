pub mod models;
pub mod wmi_bridge;
pub mod hardware_manager;
pub mod win32_util;
pub mod nvapi;
pub mod game_sync;
pub mod app_center;
pub mod lighting_profiles;
pub mod macro_engine;
use std::sync::Arc;
use tauri::State;
use crate::models::{FanMode, PerformanceMode, SystemStats, TelemetryHistoryResponse, WmiCmd};
use crate::hardware_manager::HardwareManager;
use crate::game_sync::{GameProfile, GameScanner};
use crate::app_center::{AppCenter, AppItem};
use crate::models::LightingProfile;
use crate::models::MacroGroup;

// ─── Tauri Commands ──────────────────────────────────────────────────────────
// All commands read from cache (instant) or fire to worker (non-blocking).

#[tauri::command]
fn get_system_stats(mgr: State<'_, Arc<HardwareManager>>) -> SystemStats {
    mgr.get_stats()
}

#[tauri::command]
fn set_performance_mode(mode: PerformanceMode, mgr: State<'_, Arc<HardwareManager>>) {
    mgr.send(WmiCmd::SetPerformanceMode(mode));
}

#[tauri::command]
fn set_fan_mode(fan_id: u32, mode: FanMode, custom_pct: Option<u8>, mgr: State<'_, Arc<HardwareManager>>) {
    let cmd = match mode {
        FanMode::Auto   => WmiCmd::SetFanAuto(fan_id),
        FanMode::Max    => WmiCmd::SetFanMax,
        FanMode::Custom => WmiCmd::SetFanCustom(fan_id, custom_pct.unwrap_or(60)),
    };
    mgr.send(cmd);
}

#[tauri::command]
fn set_coolboost(enabled: bool, mgr: State<'_, Arc<HardwareManager>>) {
    mgr.send(WmiCmd::SetCoolboost(enabled));
}

#[tauri::command]
fn set_win_key_lock(locked: bool, mgr: State<'_, Arc<HardwareManager>>) {
    mgr.send(WmiCmd::SetWinKeyLock(locked));
}

#[tauri::command]
fn set_sticky_keys(enabled: bool, mgr: State<'_, Arc<HardwareManager>>) {
    mgr.send(WmiCmd::SetStickyKeys(enabled));
}

#[tauri::command]
fn set_lcd_overdrive(enabled: bool, mgr: State<'_, Arc<HardwareManager>>) {
    mgr.send(WmiCmd::SetLcdOverdrive(enabled));
}

#[tauri::command]
fn set_boot_sound(enabled: bool, mgr: State<'_, Arc<HardwareManager>>) {
    mgr.send(WmiCmd::SetBootSound(enabled));
}

#[tauri::command]
fn set_kb_brightness(pct: u8, mgr: State<'_, Arc<HardwareManager>>) {
    mgr.send(WmiCmd::SetKbBrightness(pct));
}

#[tauri::command]
fn set_rgb_zone(zone: u8, r: u8, g: u8, b: u8, mgr: State<'_, Arc<HardwareManager>>) {
    mgr.send(WmiCmd::SetRgbZone(zone, r, g, b));
}

#[tauri::command]
fn set_lighting_effect(
    effect: u8,
    speed: u8,
    direction: u8,
    brightness: u8,
    r: u8,
    g: u8,
    b: u8,
    mgr: State<'_, Arc<HardwareManager>>,
) {
    mgr.send(WmiCmd::SetLightingEffect {
        effect,
        speed,
        direction,
        brightness,
        r,
        g,
        b,
    });
}

#[tauri::command]
fn set_discrete_gpu(enabled: bool, mgr: State<'_, Arc<HardwareManager>>) {
    mgr.send(WmiCmd::SetDiscreteGpu(enabled));
}

#[tauri::command]
fn set_backlight_timeout(enabled: bool, mgr: State<'_, Arc<HardwareManager>>) {
    mgr.send(WmiCmd::SetBacklightTimeout(enabled));
}

#[tauri::command]
fn set_usb_charging(enabled: bool, mgr: State<'_, Arc<HardwareManager>>) {
    mgr.send(WmiCmd::SetUsbCharging(enabled));
}

#[tauri::command]
fn set_temp_unit(use_f: bool, mgr: State<'_, Arc<HardwareManager>>) {
    mgr.send(WmiCmd::SetTempUnit(use_f));
}

#[tauri::command]
fn get_telemetry_history(timeframe: String, mgr: State<'_, Arc<HardwareManager>>) -> TelemetryHistoryResponse {
    mgr.get_telemetry_history(&timeframe)
}

#[tauri::command]
fn clear_telemetry_history(mgr: State<'_, Arc<HardwareManager>>) {
    mgr.clear_telemetry_history();
}

#[tauri::command]
fn system_reboot() -> Result<(), String> {
    win32_util::reboot_system()
}

#[tauri::command]
fn scan_installed_games() -> Vec<GameProfile> {
    GameScanner::scan_all()
}

#[tauri::command]
fn get_app_center_items() -> Vec<AppItem> {
    AppCenter::get_apps()
}

#[tauri::command]
fn launch_app(path: String) -> Result<(), String> {
    AppCenter::launch(&path)
}

#[tauri::command]
fn get_custom_apps() -> Vec<AppItem> {
    app_center::get_custom_apps()
}

#[tauri::command]
fn save_custom_app(app: AppItem) -> Result<(), String> {
    app_center::save_custom_app(app)
}

#[tauri::command]
fn delete_custom_app(id: String) -> Result<(), String> {
    app_center::delete_custom_app(id)
}

#[tauri::command]
fn pick_executable_dialog() -> Result<Option<String>, String> {
    app_center::pick_executable_dialog()
}

#[tauri::command]
fn get_lighting_profiles() -> Vec<LightingProfile> {
    lighting_profiles::get_lighting_profiles()
}

#[tauri::command]
fn save_lighting_profile(profile: LightingProfile) -> Result<(), String> {
    lighting_profiles::save_lighting_profile(profile)
}

#[tauri::command]
fn delete_lighting_profile(id: String) -> Result<(), String> {
    lighting_profiles::delete_lighting_profile(id)
}

#[tauri::command]
fn export_lighting_profile(profile: LightingProfile) -> Result<(), String> {
    lighting_profiles::export_profile_dialog(profile)
}

#[tauri::command]
fn import_lighting_profile() -> Result<Option<LightingProfile>, String> {
    lighting_profiles::import_profile_dialog()
}

#[tauri::command]
fn window_minimize(window: tauri::Window) {
    let _ = window.minimize();
}

#[tauri::command]
fn window_toggle_maximize(window: tauri::Window) {
    if let Ok(is_max) = window.is_maximized() {
        if is_max {
            let _ = window.unmaximize();
        } else {
            let _ = window.maximize();
        }
    } else {
        let _ = window.maximize();
    }
}

#[tauri::command]
fn window_close(window: tauri::Window) {
    let _ = window.close();
}

#[tauri::command]
fn window_is_maximized(window: tauri::Window) -> bool {
    window.is_maximized().unwrap_or(false)
}

#[tauri::command]
fn get_macro_groups() -> Vec<MacroGroup> {
    macro_engine::get_macro_groups()
}

#[tauri::command]
fn save_macro_group(group: MacroGroup) -> Result<(), String> {
    macro_engine::save_macro_group(group)
}

#[tauri::command]
fn play_macro(group_id: String) -> Result<(), String> {
    macro_engine::play_macro(group_id)
}

#[tauri::command]
fn set_sound_mode(mode: String) -> Result<(), String> {
    win32_util::set_sound_mode(&mode)
}

#[tauri::command]
fn get_sound_mode() -> String {
    win32_util::get_sound_mode()
}

// ─── App Entry ───────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mgr = Arc::new(HardwareManager::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(mgr)
        .invoke_handler(tauri::generate_handler![
            get_system_stats,
            set_performance_mode,
            set_fan_mode,
            set_coolboost,
            set_win_key_lock,
            set_sticky_keys,
            set_lcd_overdrive,
            set_boot_sound,
            set_kb_brightness,
            set_rgb_zone,
            set_lighting_effect,
            set_discrete_gpu,
            set_backlight_timeout,
            set_usb_charging,
            set_temp_unit,
            system_reboot,
            scan_installed_games,
            get_app_center_items,
            launch_app,
            window_minimize,
            window_toggle_maximize,
            window_close,
            window_is_maximized,
            get_custom_apps,
            save_custom_app,
            delete_custom_app,
            pick_executable_dialog,
            get_lighting_profiles,
            save_lighting_profile,
            delete_lighting_profile,
            export_lighting_profile,
            import_lighting_profile,
            get_telemetry_history,
            clear_telemetry_history,
            get_macro_groups,
            save_macro_group,
            play_macro,
            set_sound_mode,
            get_sound_mode,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
