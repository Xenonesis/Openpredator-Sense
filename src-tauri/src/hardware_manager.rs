//! Hardware manager: owns a dedicated WMI worker thread (COM STA) and monitoring thread.
//! All WMI calls happen on that one thread via an mpsc channel.
//! Tauri commands read/write shared cached state instantly without blocking.

use std::collections::VecDeque;
use std::sync::{Arc, OnceLock, mpsc};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use parking_lot::RwLock;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, Networks, RefreshKind, System};

use crate::models::{
    FanChannel, FanMode, FanStats, LightingConfig, MetricStats, NetworkStats, PerformanceMode,
    QuickSettings, SystemStats, TelemetryHistoryResponse, TelemetrySample, WmiCmd,
};
use crate::wmi_bridge::AcerWmi;
use crate::win32_util;
use crate::nvapi::NvApi;

static HW_CMD_TX: OnceLock<mpsc::Sender<WmiCmd>> = OnceLock::new();

pub fn trigger_turbo_mode() {
    if let Some(tx) = HW_CMD_TX.get() {
        let _ = tx.send(WmiCmd::SetPerformanceMode(PerformanceMode::Turbo));
    }
}
// ─── Shared Cached State ────────────────────────────────────────────────────

#[derive(Clone)]
pub struct CachedState {
    pub cpu_name:            String,
    pub cpu_usage:           f32,
    pub cpu_temp:            f32,
    pub cpu_freq_mhz:        u32,
    pub cpu_max_freq_mhz:    u32,
    pub gpu_name:            String,
    pub gpu_usage:           f32,
    pub gpu_temp:            f32,
    pub gpu_clock_mhz:       u32,
    pub gpu_max_clock_mhz:   u32,
    pub vram_clock_mhz:      u32,
    pub ram_used_gb:         f32,
    pub ram_total_gb:        f32,
    pub cpu_rpm:             u32,
    pub gpu_rpm:             u32,
    pub current_mode:        PerformanceMode,
    pub cpu_fan_mode:        FanMode,
    pub gpu_fan_mode:        FanMode,
    pub cpu_fan_pct:         u8,
    pub gpu_fan_pct:         u8,
    pub net_download_bps:    u64,
    pub net_upload_bps:      u64,
    pub win_key_locked:      bool,
    pub sticky_keys_enabled: bool,
    pub lcd_overdrive:       bool,
    pub boot_sound:          bool,
    pub coolboost:           bool,
    pub kb_brightness:       u8,
    pub discrete_gpu:          bool,
    pub backlight_timeout_30s: bool,
    pub usb_charging:          bool,
    pub temp_unit_f:           bool,
    pub lighting_effect:     String,
    pub lighting_speed:      u8,
    pub lighting_direction:  u8,
    pub lighting_brightness: u8,
    pub lighting_zones:      Vec<String>,
    pub wmi_ok:              bool,
    pub samples_recent:      VecDeque<TelemetrySample>,
    pub samples_1h:          VecDeque<TelemetrySample>,
    pub samples_24h:         VecDeque<TelemetrySample>,
    pub last_sample_1h:      Instant,
    pub last_sample_24h:     Instant,
}

impl Default for CachedState {
    fn default() -> Self {
        Self {
            cpu_name:            "Detecting CPU...".into(),
            cpu_usage:           0.0,
            cpu_temp:            0.0,
            cpu_freq_mhz:        3464,
            cpu_max_freq_mhz:    5000,
            gpu_name:            "NVIDIA GeForce RTX".into(),
            gpu_usage:           0.0,
            gpu_temp:            0.0,
            gpu_clock_mhz:       0,
            gpu_max_clock_mhz:   1850,
            vram_clock_mhz:      0,
            ram_used_gb:         0.0,
            ram_total_gb:        0.0,
            cpu_rpm:             0,
            gpu_rpm:             0,
            current_mode:        PerformanceMode::Normal,
            cpu_fan_mode:        FanMode::Auto,
            gpu_fan_mode:        FanMode::Auto,
            cpu_fan_pct:         60,
            gpu_fan_pct:         60,
            net_download_bps:    0,
            net_upload_bps:      0,
            win_key_locked:      false,
            sticky_keys_enabled: false,
            lcd_overdrive:       false,
            boot_sound:          true,
            coolboost:           false,
            kb_brightness:       100,
            discrete_gpu:          false,
            backlight_timeout_30s: false,
            usb_charging:          false,
            temp_unit_f:           false,
            lighting_effect:     "static".into(),
            lighting_speed:      2,
            lighting_direction:  1,
            lighting_brightness: 100,
            lighting_zones:      vec![
                "#00AEC7".into(),
                "#00AEC7".into(),
                "#00AEC7".into(),
                "#00AEC7".into(),
            ],
            wmi_ok:              false,
            samples_recent:      VecDeque::with_capacity(60),
            samples_1h:          VecDeque::with_capacity(60),
            samples_24h:         VecDeque::with_capacity(288),
            last_sample_1h:      Instant::now(),
            last_sample_24h:     Instant::now(),
        }
    }
}

// ─── HardwareManager ────────────────────────────────────────────────────────

pub struct HardwareManager {
    pub state: Arc<RwLock<CachedState>>,
    tx: mpsc::Sender<WmiCmd>,
}

impl HardwareManager {
    pub fn new() -> Self {
        let state = Arc::new(RwLock::new(CachedState::default()));
        let (tx, rx) = mpsc::channel::<WmiCmd>();
        let _ = HW_CMD_TX.set(tx.clone());
        // Initialize Sticky Keys state from OS
        state.write().sticky_keys_enabled = win32_util::get_sticky_keys();

        // ── Monitoring & Network Thread (safe, no COM) ─────────────────────
        {
            let state2 = state.clone();
            thread::spawn(move || {
                let mut sys = System::new_with_specifics(
                    RefreshKind::nothing()
                        .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
                        .with_memory(MemoryRefreshKind::nothing().with_ram()),
                );
                sys.refresh_specifics(
                    RefreshKind::nothing().with_cpu(CpuRefreshKind::nothing().with_cpu_usage()),
                );

                let mut networks = Networks::new_with_refreshed_list();
                let nvapi_opt = NvApi::init();

                let mut last_tick = Instant::now();
                let mut process_check_ticks: u32 = 0;
                let mut last_game_active = false;
                loop {
                    thread::sleep(Duration::from_millis(1500));
                    let elapsed = last_tick.elapsed().as_secs_f64().max(0.5);
                    last_tick = Instant::now();

                    // Refresh CPU and RAM
                    sys.refresh_specifics(
                        RefreshKind::nothing()
                            .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
                            .with_memory(MemoryRefreshKind::nothing().with_ram()),
                    );

                    let cpu_name = sys.cpus().first()
                        .map(|c| c.brand().to_string())
                        .unwrap_or_else(|| "Intel/AMD CPU".into());
                    let cpu_usage  = sys.global_cpu_usage();
                    let ram_used   = sys.used_memory() as f32 / 1_073_741_824.0;
                    let ram_total  = sys.total_memory() as f32 / 1_073_741_824.0;
                    let cpu_temp   = 35.0 + cpu_usage * 0.45;
                    let gpu_temp   = 38.0 + cpu_usage * 0.15;

                    // Refresh Network bytes
                    networks.refresh(true);
                    let mut total_rx: u64 = 0;
                    let mut total_tx: u64 = 0;
                    for (_name, data) in &networks {
                        total_rx += data.received();
                        total_tx += data.transmitted();
                    }
                    let down_bps = (total_rx as f64 / elapsed) as u64;
                    let up_bps = (total_tx as f64 / elapsed) as u64;

                    // Read GPU Clocks & Name from NVAPI
                    let mut gpu_clock = 0;
                    let mut vram_clock = 0;
                    let mut gpu_name_override = None;
                    if let Some(ref nv) = nvapi_opt {
                        gpu_name_override = Some(nv.gpu_name.clone());
                        if let Some((core, vram)) = nv.get_clocks() {
                            gpu_clock = core;
                            vram_clock = vram;
                        }
                    }

                    let cpu_freq = sys.cpus().first()
                        .map(|c| c.frequency() as u32)
                        .unwrap_or(3464);

                    let mut s = state2.write();
                    s.cpu_name         = cpu_name;
                    s.cpu_usage        = cpu_usage;
                    s.cpu_freq_mhz     = if cpu_freq > 0 { cpu_freq } else { 3464 };
                    if s.cpu_temp <= 0.0 {
                        s.cpu_temp = cpu_temp;
                    }
                    if s.gpu_temp <= 0.0 {
                        s.gpu_temp = gpu_temp;
                    }
                    s.ram_used_gb      = ram_used;
                    s.ram_total_gb     = ram_total;
                    s.net_download_bps = down_bps;
                    s.net_upload_bps   = up_bps;
                    if let Some(name) = gpu_name_override {
                        s.gpu_name = name;
                    }
                    if gpu_clock > 0 {
                        s.gpu_clock_mhz = gpu_clock;
                    }
                    if vram_clock > 0 {
                        s.vram_clock_mhz = vram_clock;
                    }

                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    let sample = TelemetrySample {
                        timestamp: now,
                        cpu_temp: s.cpu_temp,
                        cpu_usage: s.cpu_usage,
                        gpu_temp: s.gpu_temp,
                        gpu_usage: s.gpu_usage,
                        ram_used_gb: s.ram_used_gb,
                    };

                    s.samples_recent.push_back(sample.clone());
                    while s.samples_recent.len() > 60 {
                        s.samples_recent.pop_front();
                    }

                    if s.last_sample_1h.elapsed() >= Duration::from_secs(60) {
                        s.samples_1h.push_back(sample.clone());
                        while s.samples_1h.len() > 60 {
                            s.samples_1h.pop_front();
                        }
                        s.last_sample_1h = Instant::now();
                    }

                    if s.last_sample_24h.elapsed() >= Duration::from_secs(300) {
                        s.samples_24h.push_back(sample);
                        while s.samples_24h.len() > 288 {
                            s.samples_24h.pop_front();
                        }
                        s.last_sample_24h = Instant::now();
                    }

                    // Game Sync Process Watcher (Every 3 seconds)
                    process_check_ticks += 1;
                    if process_check_ticks >= 2 {
                        process_check_ticks = 0;
                        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
                        let games = crate::game_sync::GameScanner::scan_all();
                        let mut game_found = None;
                        for (_pid, proc) in sys.processes() {
                            let p_name = proc.name().to_string_lossy().to_ascii_lowercase();
                            if let Some(g) = games.iter().find(|g| g.enabled && (g.exe_name.to_ascii_lowercase() == p_name || g.exe_name.to_ascii_lowercase().ends_with(&p_name))) {
                                game_found = Some(g.clone());
                                break;
                            }
                        }

                        if let Some(g) = &game_found {
                            if !last_game_active {
                                last_game_active = true;
                                if let Some(tx_cmd) = HW_CMD_TX.get() {
                                    let _ = tx_cmd.send(WmiCmd::SetPerformanceMode(g.performance_mode));
                                    match g.fan_mode {
                                        FanMode::Max => {
                                            let _ = tx_cmd.send(WmiCmd::SetFanMax);
                                        }
                                        FanMode::Custom => {
                                            let pct = g.custom_fan_pct.unwrap_or(100);
                                            let _ = tx_cmd.send(WmiCmd::SetFanCustom(0, pct));
                                        }
                                        FanMode::Auto => {
                                            let _ = tx_cmd.send(WmiCmd::SetFanAuto(0));
                                        }
                                    }
                                }
                            }
                        } else if last_game_active {
                            last_game_active = false;
                            if let Some(tx_cmd) = HW_CMD_TX.get() {
                                let _ = tx_cmd.send(WmiCmd::SetPerformanceMode(PerformanceMode::Normal));
                                let _ = tx_cmd.send(WmiCmd::SetFanAuto(0));
                            }
                        }
                    }
                }
            });
        }

        // ── WMI COM STA worker thread ───────────────────────────────────────
        {
            let state3 = state.clone();
            thread::spawn(move || {
                let wmi_result = AcerWmi::connect();
                let wmi_ok = wmi_result.is_ok();
                state3.write().wmi_ok = wmi_ok;

                match wmi_result {
                    Err(e) => {
                        eprintln!("[WMI] Failed to connect: {}", e);
                        for _ in rx.iter() {}
                    }
                    Ok(wmi) => {
                        // Read initial hardware states
                        Self::poll_hardware(&wmi, &state3);
                        {
                            let mut s = state3.write();
                            s.lcd_overdrive = wmi.get_lcd_overdrive();
                            s.boot_sound = wmi.get_boot_sound();
                            s.coolboost = wmi.get_coolboost();
                        }

                        loop {
                            match rx.recv_timeout(Duration::from_secs(3)) {
                                Ok(cmd) => Self::handle_cmd(cmd, &wmi, &state3),
                                Err(mpsc::RecvTimeoutError::Timeout) => {
                                    Self::poll_hardware(&wmi, &state3);
                                }
                                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                            }
                        }
                    }
                }
            });
        }

        Self { state, tx }
    }

    fn poll_hardware(wmi: &AcerWmi, state: &Arc<RwLock<CachedState>>) {
        let (cpu_rpm, gpu_rpm, cpu_temp, gpu_temp) = wmi.get_hardware_health();
        let cur_mode = wmi.get_performance_mode();
        let discrete_gpu = wmi.get_discrete_gpu();
        let mut s = state.write();
        if cpu_rpm > 0 { s.cpu_rpm = cpu_rpm; }
        if gpu_rpm > 0 { s.gpu_rpm = gpu_rpm; }
        if cpu_temp > 0.0 { s.cpu_temp = cpu_temp; }
        if gpu_temp > 0.0 { s.gpu_temp = gpu_temp; }
        s.current_mode = cur_mode;
        s.discrete_gpu = discrete_gpu;
    }

    fn handle_cmd(cmd: WmiCmd, wmi: &AcerWmi, state: &Arc<RwLock<CachedState>>) {
        match cmd {
            WmiCmd::PollHardware => Self::poll_hardware(wmi, state),

            WmiCmd::SetPerformanceMode(mode) => {
                let val: u8 = match mode {
                    PerformanceMode::Quiet   => 0,
                    PerformanceMode::Normal  => 1,
                    PerformanceMode::Extreme => 2,
                    PerformanceMode::Turbo   => 3,
                };
                wmi.set_performance_mode(val);

                let mut s = state.write();
                s.current_mode = mode;

                if mode == PerformanceMode::Turbo {
                    wmi.set_fan_max();
                    s.cpu_fan_mode = FanMode::Max;
                    s.cpu_fan_pct = 100;
                    s.gpu_fan_mode = FanMode::Max;
                    s.gpu_fan_pct = 100;
                } else {
                    // Switching back to Normal, Quiet, or Extreme restores fans to Auto so fans slow down!
                    wmi.set_fan_auto(0);
                    s.cpu_fan_mode = FanMode::Auto;
                    s.gpu_fan_mode = FanMode::Auto;
                }
            }

            WmiCmd::SetFanAuto(fan_id) => {
                wmi.set_fan_auto(fan_id);
                let mut s = state.write();
                match fan_id {
                    1 => s.cpu_fan_mode = FanMode::Auto,
                    2 => s.gpu_fan_mode = FanMode::Auto,
                    _ => {}
                }
            }

            WmiCmd::SetFanCustom(fan_id, pct) => {
                wmi.set_fan_custom(fan_id, pct);
                let mut s = state.write();
                match fan_id {
                    1 => { s.cpu_fan_mode = FanMode::Custom; s.cpu_fan_pct = pct; }
                    2 => { s.gpu_fan_mode = FanMode::Custom; s.gpu_fan_pct = pct; }
                    _ => {}
                }
            }

            WmiCmd::SetFanMax => {
                wmi.set_fan_max();
                let mut s = state.write();
                s.cpu_fan_mode = FanMode::Max; s.cpu_fan_pct = 100;
                s.gpu_fan_mode = FanMode::Max; s.gpu_fan_pct = 100;
            }

            WmiCmd::SetCoolboost(enabled) => {
                wmi.set_coolboost(enabled);
                state.write().coolboost = enabled;
            }

            WmiCmd::SetWinKeyLock(locked) => {
                wmi.set_windows_key_lock(locked);
                win32_util::set_win_key_lock(locked);
                state.write().win_key_locked = locked;
            }

            WmiCmd::SetStickyKeys(enabled) => {
                win32_util::set_sticky_keys(enabled);
                state.write().sticky_keys_enabled = enabled;
            }

            WmiCmd::SetLcdOverdrive(enabled) => {
                wmi.set_lcd_overdrive(enabled);
                state.write().lcd_overdrive = enabled;
            }

            WmiCmd::SetBootSound(enabled) => {
                wmi.set_boot_sound(enabled);
                state.write().boot_sound = enabled;
            }

            WmiCmd::SetKbBrightness(pct) => {
                wmi.set_keyboard_brightness(pct);
                state.write().kb_brightness = pct;
            }

            WmiCmd::SetRgbZone(zone, r, g, b) => {
                wmi.set_rgb_zone(zone, r, g, b);
                let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);
                let mut s = state.write();
                let idx = (zone.saturating_sub(1) as usize).min(3);
                if idx < s.lighting_zones.len() {
                    s.lighting_zones[idx] = hex;
                }
            }

            WmiCmd::SetLightingEffect { effect, speed, direction, brightness, r, g, b } => {
                wmi.set_lighting_effect(effect, speed, direction, brightness, r, g, b);
                let effect_name = match effect {
                    0 | 1 => "static",
                    2     => "breathing",
                    3     => "wave",
                    4     => "zoom",
                    5     => "shift",
                    10    => "neon",
                    12    => "rainbow",
                    _     => "custom",
                };
                let mut s = state.write();
                s.lighting_effect = effect_name.to_string();
                s.lighting_speed = speed;
                s.lighting_direction = direction;
                s.lighting_brightness = brightness;
            }

            WmiCmd::SetDiscreteGpu(enable) => {
                wmi.set_discrete_gpu(enable);
                state.write().discrete_gpu = enable;
            }

            WmiCmd::SetBacklightTimeout(enable) => {
                wmi.set_backlight_timeout(enable);
                state.write().backlight_timeout_30s = enable;
            }

            WmiCmd::SetUsbCharging(enable) => {
                wmi.set_usb_charging(enable);
                state.write().usb_charging = enable;
            }

            WmiCmd::SetTempUnit(use_f) => {
                state.write().temp_unit_f = use_f;
            }
        }
    }

    // ─── Public API (instant — reads cache) ─────────────────────────────────

    pub fn get_stats(&self) -> SystemStats {
        let s = self.state.read();
        SystemStats {
            cpu_name:              s.cpu_name.clone(),
            cpu_usage:             s.cpu_usage,
            cpu_temp:              s.cpu_temp,
            cpu_frequency_mhz:     s.cpu_freq_mhz,
            cpu_max_frequency_mhz: s.cpu_max_freq_mhz,
            gpu_name:              s.gpu_name.clone(),
            gpu_usage:             s.gpu_usage,
            gpu_temp:              s.gpu_temp,
            gpu_clock_mhz:         s.gpu_clock_mhz,
            gpu_max_clock_mhz:     s.gpu_max_clock_mhz,
            vram_clock_mhz:        s.vram_clock_mhz,
            ram_used_gb:           s.ram_used_gb,
            ram_total_gb:          s.ram_total_gb,
            current_mode:   s.current_mode,
            wmi_ok:         s.wmi_ok,
            fans: FanStats {
                coolboost: s.coolboost,
                cpu: FanChannel {
                    id:                1,
                    name:              "CPU Fan".into(),
                    speed_rpm:         s.cpu_rpm,
                    target_percentage: s.cpu_fan_pct,
                    mode:              s.cpu_fan_mode,
                },
                gpu: FanChannel {
                    id:                2,
                    name:              "GPU Fan".into(),
                    speed_rpm:         s.gpu_rpm,
                    target_percentage: s.gpu_fan_pct,
                    mode:              s.gpu_fan_mode,
                },
            },
            network: NetworkStats {
                download_bytes_per_sec: s.net_download_bps,
                upload_bytes_per_sec:   s.net_upload_bps,
            },
            settings: QuickSettings {
                win_key_locked:      s.win_key_locked,
                sticky_keys_enabled: s.sticky_keys_enabled,
                lcd_overdrive:       s.lcd_overdrive,
                boot_sound:          s.boot_sound,
                coolboost:           s.coolboost,
                kb_brightness:         s.kb_brightness,
                discrete_gpu:          s.discrete_gpu,
                backlight_timeout_30s: s.backlight_timeout_30s,
                usb_charging:          s.usb_charging,
                temp_unit_f:           s.temp_unit_f,
            },
            lighting: LightingConfig {
                effect:     s.lighting_effect.clone(),
                speed:      s.lighting_speed,
                direction:  s.lighting_direction,
                brightness: s.lighting_brightness,
                zones:      s.lighting_zones.clone(),
            },
        }
    }

    // ─── Commands (fire to worker thread, non-blocking) ──────────────────────

    pub fn send(&self, cmd: WmiCmd) {
        let _ = self.tx.send(cmd);
    }

    pub fn get_telemetry_history(&self, timeframe: &str) -> TelemetryHistoryResponse {
        let s = self.state.read();
        let samples: Vec<TelemetrySample> = if timeframe.eq_ignore_ascii_case("24h") {
            s.samples_24h.iter().cloned().collect()
        } else {
            if s.samples_1h.len() < 2 {
                s.samples_recent.iter().cloned().collect()
            } else {
                s.samples_1h.iter().cloned().collect()
            }
        };

        let cpu_temps: Vec<f32> = samples.iter().map(|item| item.cpu_temp).collect();
        let cpu_usages: Vec<f32> = samples.iter().map(|item| item.cpu_usage).collect();
        let gpu_temps: Vec<f32> = samples.iter().map(|item| item.gpu_temp).collect();
        let gpu_usages: Vec<f32> = samples.iter().map(|item| item.gpu_usage).collect();
        let rams: Vec<f32> = samples.iter().map(|item| item.ram_used_gb).collect();

        let cpu_temp = calculate_metric_stats(&cpu_temps, s.cpu_temp);
        let cpu_usage = calculate_metric_stats(&cpu_usages, s.cpu_usage);
        let gpu_temp = calculate_metric_stats(&gpu_temps, s.gpu_temp);
        let gpu_usage = calculate_metric_stats(&gpu_usages, s.gpu_usage);
        let ram = calculate_metric_stats(&rams, s.ram_used_gb);

        TelemetryHistoryResponse {
            samples,
            cpu_temp,
            cpu_usage,
            gpu_temp,
            gpu_usage,
            ram,
        }
    }

    pub fn clear_telemetry_history(&self) {
        let mut s = self.state.write();
        s.samples_recent.clear();
        s.samples_1h.clear();
        s.samples_24h.clear();
        s.last_sample_1h = Instant::now();
        s.last_sample_24h = Instant::now();
    }
}

fn calculate_metric_stats(values: &[f32], current: f32) -> MetricStats {
    if values.is_empty() {
        return MetricStats {
            min: current,
            max: current,
            avg: current,
            current,
        };
    }
    let non_zero_min = values.iter().copied().filter(|&v| v > 0.0).fold(f32::INFINITY, f32::min);
    let min = if non_zero_min.is_finite() {
        non_zero_min
    } else {
        let m = values.iter().copied().fold(f32::INFINITY, f32::min);
        if m.is_finite() { m } else { current }
    };
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let max = if max.is_finite() { max } else { current };
    let avg = values.iter().sum::<f32>() / (values.len() as f32);

    MetricStats {
        min,
        max,
        avg,
        current,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_metric_stats_empty() {
        let stats = calculate_metric_stats(&[], 45.0);
        assert_eq!(stats.min, 45.0);
        assert_eq!(stats.max, 45.0);
        assert_eq!(stats.avg, 45.0);
        assert_eq!(stats.current, 45.0);
    }

    #[test]
    fn test_calculate_metric_stats_values() {
        let vals = vec![40.0, 50.0, 60.0];
        let stats = calculate_metric_stats(&vals, 55.0);
        assert_eq!(stats.min, 40.0);
        assert_eq!(stats.max, 60.0);
        assert_eq!(stats.avg, 50.0);
        assert_eq!(stats.current, 55.0);
    }

    #[test]
    fn test_calculate_metric_stats_filters_zero_min() {
        let vals = vec![0.0, 42.0, 58.0];
        let stats = calculate_metric_stats(&vals, 50.0);
        assert_eq!(stats.min, 42.0);
        assert_eq!(stats.max, 58.0);
        assert_eq!(stats.current, 50.0);
    }

    #[test]
    fn test_get_telemetry_history_fallback_and_selection() {
        let (tx, _rx) = mpsc::channel();
        let hm = HardwareManager {
            state: Arc::new(RwLock::new(CachedState::default())),
            tx,
        };

        let sample1 = TelemetrySample {
            timestamp: 100,
            cpu_temp: 45.0,
            cpu_usage: 10.0,
            gpu_temp: 40.0,
            gpu_usage: 5.0,
            ram_used_gb: 8.0,
        };
        let sample2 = TelemetrySample {
            timestamp: 101,
            cpu_temp: 55.0,
            cpu_usage: 20.0,
            gpu_temp: 48.0,
            gpu_usage: 15.0,
            ram_used_gb: 8.5,
        };

        // Empty 1h buffer -> falls back to recent
        {
            let mut s = hm.state.write();
            s.samples_recent.push_back(sample1.clone());
            s.samples_recent.push_back(sample2.clone());
        }
        let res1 = hm.get_telemetry_history("1h");
        assert_eq!(res1.samples.len(), 2);
        assert_eq!(res1.cpu_temp.min, 45.0);
        assert_eq!(res1.cpu_temp.max, 55.0);

        // Only 1 item in 1h buffer -> still falls back to recent
        {
            let mut s = hm.state.write();
            s.samples_1h.push_back(sample1.clone());
        }
        let res2 = hm.get_telemetry_history("1h");
        assert_eq!(res2.samples.len(), 2); // recent has 2

        // 2 items in 1h buffer -> uses 1h buffer
        let sample3 = TelemetrySample {
            timestamp: 102,
            cpu_temp: 65.0,
            cpu_usage: 30.0,
            gpu_temp: 55.0,
            gpu_usage: 25.0,
            ram_used_gb: 9.0,
        };
        {
            let mut s = hm.state.write();
            s.samples_1h.push_back(sample3.clone());
        }
        let res3 = hm.get_telemetry_history("1h");
        assert_eq!(res3.samples.len(), 2); // 1h has sample1 and sample3
        assert_eq!(res3.samples[1].cpu_temp, 65.0);

        // 24h buffer
        {
            let mut s = hm.state.write();
            s.samples_24h.push_back(sample1.clone());
        }
        let res24 = hm.get_telemetry_history("24h");
        assert_eq!(res24.samples.len(), 1);

        // Clear history
        hm.clear_telemetry_history();
        let res_cleared = hm.get_telemetry_history("1h");
        assert_eq!(res_cleared.samples.len(), 0);
    }
}
