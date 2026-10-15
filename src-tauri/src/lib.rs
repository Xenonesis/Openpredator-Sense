pub mod models;
pub mod wmi_bridge;
pub mod hardware_manager;
pub mod win32_util;
pub mod nvapi;
pub mod game_sync;
pub mod app_center;

use std::sync::Arc;
use tauri::State;
use crate::models::{FanMode, PerformanceMode, SystemStats, WmiCmd};
use crate::hardware_manager::HardwareManager;
use crate::game_sync::{GameProfile, GameScanner};
use crate::app_center::{AppCenter, AppItem};

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
            scan_installed_games,
            get_app_center_items,
            launch_app,
            window_minimize,
            window_toggle_maximize,
            window_close,
            window_is_maximized,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
