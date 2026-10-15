use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FanMode {
    Auto,
    Max,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PerformanceMode {
    Quiet,
    Normal,
    Extreme,
    Turbo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanChannel {
    pub id: u32,
    pub name: String,
    pub speed_rpm: u32,
    pub target_percentage: u8,
    pub mode: FanMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickSettings {
    pub win_key_locked: bool,
    pub sticky_keys_enabled: bool,
    pub lcd_overdrive: bool,
    pub boot_sound: bool,
    pub coolboost: bool,
    pub kb_brightness: u8,
    pub discrete_gpu: bool,
    pub backlight_timeout_30s: bool,
    pub usb_charging: bool,
    pub temp_unit_f: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub download_bytes_per_sec: u64,
    pub upload_bytes_per_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingConfig {
    pub effect: String,
    pub speed: u8,
    pub direction: u8,
    pub brightness: u8,
    pub zones: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppItem {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub exe_path: String,
    pub installed: bool,
    pub icon_name: String,
    #[serde(default)]
    pub custom: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightingProfile {
    pub id: String,
    pub name: String,
    pub zones: Vec<String>,
    pub effect: String,
    pub speed: u8,
    pub direction: u8,
    pub brightness: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu_name: String,
    pub cpu_usage: f32,
    pub cpu_temp: f32,
    pub cpu_frequency_mhz: u32,
    pub cpu_max_frequency_mhz: u32,
    pub gpu_name: String,
    pub gpu_usage: f32,
    pub gpu_temp: f32,
    pub gpu_clock_mhz: u32,
    pub gpu_max_clock_mhz: u32,
    pub vram_clock_mhz: u32,
    pub ram_used_gb: f32,
    pub ram_total_gb: f32,
    pub current_mode: PerformanceMode,
    pub fans: FanStats,
    pub network: NetworkStats,
    pub settings: QuickSettings,
    pub lighting: LightingConfig,
    pub wmi_ok: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanStats {
    pub cpu: FanChannel,
    pub gpu: FanChannel,
    pub coolboost: bool,
}

// Commands sent from Tauri handlers to the WMI worker thread
pub enum WmiCmd {
    SetPerformanceMode(PerformanceMode),
    SetFanAuto(u32),
    SetFanCustom(u32, u8),
    SetFanMax,
    SetCoolboost(bool),
    SetWinKeyLock(bool),
    SetStickyKeys(bool),
    SetLcdOverdrive(bool),
    SetBootSound(bool),
    SetKbBrightness(u8),
    SetRgbZone(u8, u8, u8, u8),
    SetLightingEffect {
        effect: u8,
        speed: u8,
        direction: u8,
        brightness: u8,
        r: u8,
        g: u8,
        b: u8,
    },
    SetDiscreteGpu(bool),
    SetBacklightTimeout(bool),
    SetUsbCharging(bool),
    SetTempUnit(bool),
    PollHardware,
}
