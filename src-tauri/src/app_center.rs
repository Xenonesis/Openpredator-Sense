//! App Center module: Detects installed gaming tools and launches them.

use std::path::Path;
use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppItem {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub exe_path: String,
    pub installed: bool,
    pub icon_name: String,
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
