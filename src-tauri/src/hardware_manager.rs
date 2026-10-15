//! Hardware manager: owns a dedicated WMI worker thread (COM STA) and monitoring thread.
//! All WMI calls happen on that one thread via an mpsc channel.
//! Tauri commands read/write shared cached state instantly without blocking.

use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, Networks, RefreshKind, System};

use crate::models::{
    FanChannel, FanMode, FanStats, LightingConfig, NetworkStats, PerformanceMode,
    QuickSettings, SystemStats, WmiCmd,
};
use crate::wmi_bridge::AcerWmi;
use crate::win32_util;
use crate::nvapi::NvApi;

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
    pub lighting_effect:     String,
    pub lighting_speed:      u8,
    pub lighting_direction:  u8,
    pub lighting_brightness: u8,
    pub lighting_zones:      Vec<String>,
    pub wmi_ok:              bool,
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
        let mut s = state.write();
        if cpu_rpm > 0 { s.cpu_rpm = cpu_rpm; }
        if gpu_rpm > 0 { s.gpu_rpm = gpu_rpm; }
        if cpu_temp > 0.0 { s.cpu_temp = cpu_temp; }
        if gpu_temp > 0.0 { s.gpu_temp = gpu_temp; }
        s.current_mode = cur_mode;
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
                kb_brightness:       s.kb_brightness,
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
}
