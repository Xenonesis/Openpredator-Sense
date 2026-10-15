//! App Center module: Detects installed gaming tools and launches them.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
pub use crate::models::AppItem;

pub fn get_app_data_dir() -> PathBuf {
    let base = env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"));
    let dir = base.join("OpenPredatorSense");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn get_custom_apps_path() -> PathBuf {
    get_app_data_dir().join("custom_apps.json")
}

pub fn get_custom_apps() -> Vec<AppItem> {
    let path = get_custom_apps_path();
    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(apps) = serde_json::from_str::<Vec<AppItem>>(&data) {
            return apps;
        }
    }
    Vec::new()
}

pub fn save_custom_app(app: AppItem) -> Result<(), String> {
    let mut apps = get_custom_apps();
    apps.retain(|a| a.id != app.id);
    apps.push(app);
    let path = get_custom_apps_path();
    let json = serde_json::to_string_pretty(&apps).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn delete_custom_app(id: String) -> Result<(), String> {
    let mut apps = get_custom_apps();
    apps.retain(|a| a.id != id);
    let path = get_custom_apps_path();
    let json = serde_json::to_string_pretty(&apps).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn pick_executable_dialog() -> Result<Option<String>, String> {
    let script = r#"
    Add-Type -AssemblyName System.Windows.Forms
    $f = New-Object System.Windows.Forms.OpenFileDialog
    $f.Filter = "Executable Files (*.exe)|*.exe|All Files (*.*)|*.*"
    $f.Title = "Select Application Executable"
    if ($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
        Write-Output $f.FileName
    }
    "#;
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()
        .map_err(|e| e.to_string())?;
    let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path_str.is_empty() {
        Ok(None)
    } else {
        Ok(Some(path_str))
    }
}

pub struct AppCenter;

impl AppCenter {
    pub fn get_apps() -> Vec<AppItem> {
        let candidates = vec![
            (
                "care_center",
                "Acer Care Center",
                "OEM Diagnostics",
                "System health check, tune-up, and Acer recovery tools.",
                r"C:\Program Files (x86)\Acer\Care Center\CareCenter.exe",
                "Wrench",
            ),
            (
                "killer_control",
                "Killer Control Center",
                "Network Optimizer",
                "Prioritize network traffic for gaming and minimize latency.",
                r"C:\Program Files\Killer Networking\Killer Control Center\KillerControlCenter.exe",
                "Wifi",
            ),
            (
                "geforce_experience",
                "NVIDIA GeForce Experience",
                "GPU Utilities",
                "Driver updates, game optimization, and ShadowPlay recording.",
                r"C:\Program Files\NVIDIA Corporation\NVIDIA GeForce Experience\NVIDIA GeForce Experience.exe",
                "Monitor",
            ),
            (
                "nvidia_app",
                "NVIDIA App",
                "GPU Utilities",
                "Unified NVIDIA control center and GPU driver manager.",
                r"C:\Program Files\NVIDIA Corporation\NVIDIA App\CEF\NVIDIA App.exe",
                "Monitor",
            ),
            (
                "steam",
                "Steam",
                "Game Platform",
                "Ultimate online game store and community platform.",
                r"C:\Program Files (x86)\Steam\steam.exe",
                "Gamepad2",
            ),
            (
                "discord",
                "Discord",
                "Communications",
                "Voice, video, and text communication for gamers.",
                r"C:\Users\Acer\AppData\Local\Discord\Update.exe",
                "MessageSquare",
            ),
            (
                "obs_studio",
                "OBS Studio",
                "Broadcasting",
                "Live streaming and video recording software.",
                r"C:\Program Files\obs-studio\bin\64bit\obs64.exe",
                "Video",
            ),
            (
                "xsplit",
                "XSplit Gamecaster",
                "Broadcasting",
                "Easy live streaming and recording for PC gaming.",
                r"C:\Program Files (x86)\SplitmediaLabs\XSplit Gamecaster\XSplit.Gamecaster.exe",
                "Radio",
            ),
        ];

        let mut apps = Vec::new();
        for (id, name, cat, desc, path, icon) in candidates {
            let is_inst = Path::new(path).exists();
            apps.push(AppItem {
                id: id.to_string(),
                name: name.to_string(),
                category: cat.to_string(),
                description: desc.to_string(),
                exe_path: path.to_string(),
                installed: is_inst,
                icon_name: icon.to_string(),
                custom: false,
            });
        }

        apps
    }

    pub fn launch(path: &str) -> Result<(), String> {
        let p = Path::new(path);
        if !p.exists() {
            return Err(format!("Executable not found: {}", path));
        }

        Command::new(p)
            .spawn()
            .map_err(|e| format!("Failed to launch {}: {}", path, e))?;

        Ok(())
    }
}
