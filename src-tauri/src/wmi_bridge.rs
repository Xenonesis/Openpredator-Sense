//! Native WMI bridge calling exact Acer ACPI BIOS methods.
//! Matches PSSvc.exe implementation.

use serde::{Deserialize, Serialize};
use wmi::{WMIConnection, WMIError};
use crate::models::PerformanceMode;
#[allow(non_camel_case_types, non_snake_case)]
#[derive(Deserialize)]
struct AcerGamingFunction {
    __Path: String,
}

#[allow(non_camel_case_types, non_snake_case)]
#[derive(Deserialize)]
struct AcerGenericMethod {
    __Path: String,
}

#[allow(non_snake_case)]
#[derive(Serialize)]
struct UiInput32 {
    uiInput: u32,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct UiOutput32 {
    uiOutput: Option<u32>,
}
#[allow(non_snake_case)]
#[derive(Serialize)]
struct GmInput32 {
    gmInput: u32,
}

#[allow(non_snake_case)]
#[derive(Serialize)]
struct GmInput64 {
    gmInput: u64,
}

#[allow(non_snake_case)]
#[derive(Serialize)]
struct GmInputBytes16 {
    gmInput: [u8; 16],
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct GmOutput64 {
    gmOutput: Option<u64>,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
struct GmOutput32 {
    gmOutput: Option<u32>,
}

pub struct AcerWmi {
    con: WMIConnection,
    path: String,
    generic_path: Option<String>,
}

unsafe impl Send for AcerWmi {}

impl AcerWmi {
    pub fn connect() -> Result<Self, String> {
        let con = WMIConnection::with_namespace_path("ROOT\\WMI")
            .map_err(|e| format!("WMI connect ROOT\\WMI: {:?}", e))?;

        let instances: Vec<AcerGamingFunction> = con
            .query()
            .map_err(|e| format!("Query AcerGamingFunction: {:?}", e))?;

        let path = instances
            .into_iter()
            .next()
            .map(|i| i.__Path)
            .ok_or_else(|| "AcerGamingFunction instance not found".to_string())?;

        let generic_path = con
            .query::<AcerGenericMethod>()
            .ok()
            .and_then(|v| v.into_iter().next())
            .map(|i| i.__Path);

        Ok(Self { con, path, generic_path })
    }

    /// Read real Fan RPM and Temperatures via GetGamingSysInfo
    /// Input 0x0201 = CPU Fan RPM, 0x0601 = GPU Fan RPM
    /// Input 0x0101 = CPU Temp, 0x0301 = GPU Temp
    pub fn get_hardware_health(&self) -> (u32, u32, f32, f32) {
        let cpu_rpm = self.get_sysinfo_value(0x0201);
        let gpu_rpm = self.get_sysinfo_value(0x0601);
        let cpu_temp = self.get_sysinfo_value(0x0101) as f32;
        let gpu_temp = self.get_sysinfo_value(0x0301) as f32;
        (cpu_rpm, gpu_rpm, cpu_temp, gpu_temp)
    }

    pub fn get_fan_rpm(&self, fan_index: u32) -> u32 {
        let sub_cmd = match fan_index {
            1 => 0x0201, // CPU Fan RPM
            2 => 0x0601, // GPU Fan RPM
            _ => 0x0201,
        };
        let rpm = self.get_sysinfo_value(sub_cmd);
        if rpm > 0 {
            return rpm;
        }

        // Fallback to legacy GetGamingFanSpeed if GetGamingSysInfo is 0
        let res: Result<GmOutput64, WMIError> = self.con.exec_instance_method::<AcerGamingFunction, _>(
            &self.path,
            "GetGamingFanSpeed",
            GmInput32 { gmInput: fan_index },
        );
        match res {
            Ok(out) => {
                let raw = out.gmOutput.unwrap_or(0);
                let r = (raw >> 16) as u32;
                if r > 0 { r } else { (raw & 0xFFFF) as u32 }
            }
            Err(_) => 0,
        }
    }

    fn get_sysinfo_value(&self, input: u32) -> u32 {
        let res: Result<GmOutput64, WMIError> = self.con.exec_instance_method::<AcerGamingFunction, _>(
            &self.path,
            "GetGamingSysInfo",
            GmInput32 { gmInput: input },
        );
        match res {
            Ok(out) => {
                let raw = out.gmOutput.unwrap_or(0);
                ((raw >> 8) & 0xFFFF) as u32
            }
            Err(_) => 0,
        }
    }

    /// Set Fan Auto Mode:
    /// Sets both CPU and GPU fans to automatic curve: 0x00410009
    pub fn set_fan_auto(&self, _fan_index: u32) {
        let _: Result<GmOutput32, _> = self.con.exec_instance_method::<AcerGamingFunction, _>(
            &self.path,
            "SetGamingFanBehavior",
            GmInput64 { gmInput: 0x00410009 },
        );
    }

    /// Set Manual Fan Speed:
    /// Custom Behavior: 0x00C30009
    /// CPU: SetGamingFanSpeed(1 | (pct << 16))
    /// GPU: SetGamingFanSpeed(4 | (pct << 16))
    pub fn set_fan_custom(&self, fan_index: u32, pct: u8) {
        let _: Result<GmOutput32, _> = self.con.exec_instance_method::<AcerGamingFunction, _>(
            &self.path,
            "SetGamingFanBehavior",
            GmInput64 { gmInput: 0x00C30009 },
        );

        let fan_id: u64 = match fan_index {
            1 => 1,
            2 => 4, // GPU fan index is 4 on PH315-55!
            _ => 1,
        };

        let payload: u64 = fan_id | ((pct.min(100) as u64) << 16);
        let _: Result<GmOutput32, _> = self.con.exec_instance_method::<AcerGamingFunction, _>(
            &self.path,
            "SetGamingFanSpeed",
            GmInput64 { gmInput: payload },
        );
    }

    /// Set Dual Fans to Maximum Roar: 0x00820009
    pub fn set_fan_max(&self) {
        let _: Result<GmOutput32, _> = self.con.exec_instance_method::<AcerGamingFunction, _>(
            &self.path,
            "SetGamingFanBehavior",
            GmInput64 { gmInput: 0x00820009 },
        );

        // Also enforce 100% speed payloads on CPU (1) and GPU (4)
        let _: Result<GmOutput32, _> = self.con.exec_instance_method::<AcerGamingFunction, _>(
            &self.path,
            "SetGamingFanSpeed",
            GmInput64 { gmInput: 1 | (100u64 << 16) },
        );
        let _: Result<GmOutput32, _> = self.con.exec_instance_method::<AcerGamingFunction, _>(
            &self.path,
            "SetGamingFanSpeed",
            GmInput64 { gmInput: 4 | (100u64 << 16) },
        );
    }

    /// Set Performance Mode in ACPI BIOS: Quiet (0), Default/Normal (1), Extreme (2), Turbo (3)
    /// ACPI Payload format: 1 | (mode << 8)
    pub fn set_performance_mode(&self, mode_id: u8) {
        let payload: u64 = 1 | ((mode_id as u64) << 8);
        let _: Result<GmOutput32, _> = self.con.exec_instance_method::<AcerGamingFunction, _>(
            &self.path,
            "SetGamingMiscSetting",
            GmInput64 { gmInput: payload },
        );
    }

    /// Read Performance Mode from ACPI BIOS: GetGamingMiscSetting(1)
    pub fn get_performance_mode(&self) -> PerformanceMode {
        let res: Result<GmOutput64, WMIError> = self.con.exec_instance_method::<AcerGamingFunction, _>(
            &self.path,
            "GetGamingMiscSetting",
            GmInput32 { gmInput: 1 },
        );
        match res {
            Ok(out) => {
                let raw = out.gmOutput.unwrap_or(0);
                let mode_id = ((raw >> 8) & 0xFF) as u8;
                match mode_id {
                    0 => PerformanceMode::Quiet,
                    1 => PerformanceMode::Normal,
                    2 => PerformanceMode::Extreme,
                    3 => PerformanceMode::Turbo,
                    _ => PerformanceMode::Normal,
                }
            }
            Err(_) => PerformanceMode::Normal,
        }
    }

    /// Set RGB Keyboard Color via SetGamingRgbKb
    pub fn set_rgb_zone(&self, zone: u8, r: u8, g: u8, b: u8) {
        // Ensure static keyboard layer is enabled: 0x0F << 40
        let _: Result<GmOutput32, _> = self.con.exec_instance_method::<AcerGamingFunction, _>(
            &self.path,
            "SetGamingLEDBehavior",
            GmInput64 { gmInput: 0x00000F0000000000 },
        );

        let zone_mask: u64 = match zone {
            1 => 0x01,
            2 => 0x02,
            3 => 0x04,
            4 => 0x08,
            _ => 0x0F,
        };
        let payload_rgb_kb: u64 = zone_mask
            | ((r as u64) << 8)
            | ((g as u64) << 16)
            | ((b as u64) << 24);

        let _: Result<GmOutput32, _> = self.con.exec_instance_method::<AcerGamingFunction, _>(
            &self.path,
            "SetGamingRgbKb",
            GmInput64 { gmInput: payload_rgb_kb },
        );
    }

    /// Set Windows Key Lock in BIOS ACPI
    pub fn set_windows_key_lock(&self, locked: bool) {
        let payload: u64 = 2 | if locked { 1u64 << 24 } else { 0 };
        let _: Result<GmOutput32, _> = self.con.exec_instance_method::<AcerGamingFunction, _>(
            &self.path,
            "SetGamingProfile",
            GmInput64 { gmInput: payload },
        );
    }

    /// Set LCD OverDrive (3ms panel response time boost)
    pub fn set_lcd_overdrive(&self, enabled: bool) {
        let payload: u64 = 16 | if enabled { 1u64 << 48 } else { 0 };
        let _: Result<GmOutput32, _> = self.con.exec_instance_method::<AcerGamingFunction, _>(
            &self.path,
            "SetGamingProfile",
            GmInput64 { gmInput: payload },
        );
    }

    /// Read LCD OverDrive status from ACPI
    pub fn get_lcd_overdrive(&self) -> bool {
        let res: Result<GmOutput64, _> = self.con.exec_instance_method::<AcerGamingFunction, _>(
            &self.path,
            "GetGamingProfile",
            GmInput32 { gmInput: 16 },
        );
        match res {
            Ok(out) => {
                let raw = out.gmOutput.unwrap_or(0);
                ((raw >> 48) & 0xFF) == 1
            }
            Err(_) => false,
        }
    }

    /// Set Boot Animation Sound in BIOS ACPI
    pub fn set_boot_sound(&self, enabled: bool) {
        let payload: u64 = 6 | if enabled { 1u64 << 8 } else { 0 };
        let _: Result<GmOutput32, _> = self.con.exec_instance_method::<AcerGamingFunction, _>(
            &self.path,
            "SetGamingMiscSetting",
            GmInput64 { gmInput: payload },
        );
    }

    /// Read Boot Animation Sound status from ACPI
    pub fn get_boot_sound(&self) -> bool {
        let res: Result<GmOutput64, _> = self.con.exec_instance_method::<AcerGamingFunction, _>(
            &self.path,
            "GetGamingMiscSetting",
            GmInput32 { gmInput: 6 },
        );
        match res {
            Ok(out) => {
                let raw = out.gmOutput.unwrap_or(0);
                ((raw >> 8) & 0xFF) == 1
            }
            Err(_) => false,
        }
    }

    /// Set CoolBoost state on AcerGenericMethod
    pub fn set_coolboost(&self, enabled: bool) {
        if let Some(ref gp) = self.generic_path {
            let payload: u32 = 7 | if enabled { 1 << 16 } else { 0 };
            let _: Result<UiOutput32, _> = self.con.exec_instance_method::<AcerGenericMethod, _>(
                gp,
                "SetFunction",
                UiInput32 { uiInput: payload },
            );
        }
    }

    /// Read CoolBoost state from AcerGenericMethod
    pub fn get_coolboost(&self) -> bool {
        if let Some(ref gp) = self.generic_path {
            let res: Result<UiOutput32, _> = self.con.exec_instance_method::<AcerGenericMethod, _>(
                gp,
                "GetFunction",
                UiInput32 { uiInput: 0x00000207 },
            );
            if let Ok(out) = res {
                let raw = out.uiOutput.unwrap_or(0);
                return ((raw >> 8) & 0xFF) == 1;
            }
        }
        false
    }

    /// Set Keyboard Backlight Brightness (0..100) via 16-byte array
    pub fn set_keyboard_brightness(&self, pct: u8) {
        let mut arr = [0u8; 16];
        arr[2] = pct.min(100);
        arr[8] = 0x0F;
        let _: Result<GmOutput32, _> = self.con.exec_instance_method::<AcerGamingFunction, _>(
            &self.path,
            "SetGamingKBBacklight",
            GmInputBytes16 { gmInput: arr },
        );
    }

    /// Set Dynamic RGB Lighting Effect
    pub fn set_lighting_effect(
        &self,
        effect: u8,
        speed: u8,
        direction: u8,
        brightness: u8,
        r: u8,
        g: u8,
        b: u8,
    ) {
        let mut arr = [0u8; 16];
        arr[0] = 0;
        arr[1] = speed;
        arr[2] = brightness.min(100);
        arr[3] = direction;
        arr[5] = r;
        arr[6] = g;
        arr[7] = b;
        arr[8] = 0x0F;
        arr[9] = effect;

        let _: Result<GmOutput32, _> = self.con.exec_instance_method::<AcerGamingFunction, _>(
            &self.path,
            "SetGamingKBBacklight",
            GmInputBytes16 { gmInput: arr },
        );

        let payload: u64 = (effect as u64)
            | ((speed as u64) << 8)
            | (((brightness.min(100)) as u64) << 16)
            | ((direction as u64) << 32)
            | ((r as u64) << 40)
            | ((g as u64) << 48)
            | ((b as u64) << 56);
        let _: Result<GmOutput32, _> = self.con.exec_instance_method::<AcerGamingFunction, _>(
            &self.path,
            "SetGamingLEDBehavior",
            GmInput64 { gmInput: payload },
        );
    }
}
