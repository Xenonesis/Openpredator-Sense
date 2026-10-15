//! Game Sync module: Scans installed games and auto-applies performance & fan profiles.

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::models::{FanMode, PerformanceMode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameProfile {
    pub id: String,
    pub name: String,
    pub exe_name: String,
    pub path: String,
    pub performance_mode: PerformanceMode,
    pub fan_mode: FanMode,
    pub custom_fan_pct: Option<u8>,
    pub lighting_effect: Option<String>,
    pub enabled: bool,
}

impl GameProfile {
    pub fn new_default(name: &str, exe_name: &str, path: &str) -> Self {
        Self {
            id: format!("{}_{}", name.to_lowercase().replace(' ', "_"), exe_name.to_lowercase()),
            name: name.to_string(),
            exe_name: exe_name.to_string(),
            path: path.to_string(),
            performance_mode: PerformanceMode::Turbo,
            fan_mode: FanMode::Max,
            custom_fan_pct: Some(100),
            lighting_effect: Some("wave".to_string()),
            enabled: true,
        }
    }
}

pub struct GameScanner;

impl GameScanner {
    /// Scans Steam and Epic Games libraries for installed games.
    pub fn scan_all() -> Vec<GameProfile> {
        let mut list = Vec::new();
        Self::scan_steam(&mut list);
        Self::scan_epic(&mut list);

        // Fallback common known games if not detected through manifests
        Self::scan_common_paths(&mut list);

        // Deduplicate by exe_name
        list.sort_by(|a, b| a.exe_name.cmp(&b.exe_name));
        list.dedup_by(|a, b| a.exe_name.eq_ignore_ascii_case(&b.exe_name));

        list
    }

    fn scan_steam(list: &mut Vec<GameProfile>) {
        let steam_paths = [
            PathBuf::from(r"C:\Program Files (x86)\Steam"),
            PathBuf::from(r"C:\Program Files\Steam"),
            PathBuf::from(r"D:\Steam"),
            PathBuf::from(r"E:\Steam"),
        ];

        for sp in &steam_paths {
            let vdf = sp.join(r"steamapps\libraryfolders.vdf");
            if vdf.exists() {
                if let Ok(content) = fs::read_to_string(&vdf) {
                    // Extract path values from libraryfolders.vdf
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("\"path\"") {
                            let parts: Vec<&str> = trimmed.split('"').collect();
                            if parts.len() >= 4 {
                                let lib_path = PathBuf::from(parts[3].replace(r"\\", r"\"));
                                let common = lib_path.join(r"steamapps\common");
                                if common.exists() {
                                    Self::scan_directory_for_games(&common, list);
                                }
                            }
                        }
                    }
                }
            }

            // Also check main steamapps/common directly
            let main_common = sp.join(r"steamapps\common");
            if main_common.exists() {
                Self::scan_directory_for_games(&main_common, list);
            }
        }
    }

    fn scan_epic(list: &mut Vec<GameProfile>) {
        let epic_manifests = PathBuf::from(r"C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests");
        if epic_manifests.exists() {
            if let Ok(entries) = fs::read_dir(epic_manifests) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("item") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            let mut app_name = String::new();
                            let mut install_loc = String::new();
                            let mut launch_exe = String::new();

                            for line in content.lines() {
                                let l = line.trim();
                                if l.starts_with("\"DisplayName\"") {
                                    let p: Vec<&str> = l.split('"').collect();
                                    if p.len() >= 4 { app_name = p[3].to_string(); }
                                } else if l.starts_with("\"InstallLocation\"") {
                                    let p: Vec<&str> = l.split('"').collect();
                                    if p.len() >= 4 { install_loc = p[3].replace(r"\\", r"\"); }
                                } else if l.starts_with("\"LaunchExecutable\"") {
                                    let p: Vec<&str> = l.split('"').collect();
                                    if p.len() >= 4 { launch_exe = p[3].to_string(); }
                                }
                            }

                            if !launch_exe.is_empty() {
                                let full_path = Path::new(&install_loc).join(&launch_exe);
                                list.push(GameProfile::new_default(
                                    if !app_name.is_empty() { &app_name } else { &launch_exe },
                                    &launch_exe,
                                    full_path.to_str().unwrap_or(""),
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    fn scan_directory_for_games(dir: &Path, list: &mut Vec<GameProfile>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    let folder_name = p.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown Game");
                    // Search top level of game folder for .exe
                    if let Ok(sub_entries) = fs::read_dir(&p) {
                        for sub in sub_entries.flatten() {
                            let sub_path = sub.path();
                            if sub_path.extension().and_then(|s| s.to_str()) == Some("exe") {
                                let exe_name = sub_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                                // Skip installer/uninstaller/helper exes
                                let lower = exe_name.to_lowercase();
                                if !lower.contains("setup")
                                    && !lower.contains("unins")
                                    && !lower.contains("crash")
                                    && !lower.contains("update")
                                    && !lower.contains("redist")
                                    && !lower.contains("dxsetup")
                                {
                                    list.push(GameProfile::new_default(folder_name, exe_name, sub_path.to_str().unwrap_or("")));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn scan_common_paths(list: &mut Vec<GameProfile>) {
        let common = [
            ("Cyberpunk 2077", "Cyberpunk2077.exe", r"C:\Program Files (x86)\GOG Galaxy\Games\Cyberpunk 2077\bin\x64\Cyberpunk2077.exe"),
            ("Valorant", "VALORANT-Win64-Shipping.exe", r"C:\Riot Games\VALORANT\live\ShooterGame\Binaries\Win64\VALORANT-Win64-Shipping.exe"),
            ("League of Legends", "LeagueClient.exe", r"C:\Riot Games\League of Legends\LeagueClient.exe"),
            ("Genshin Impact", "GenshinImpact.exe", r"C:\Program Files\Genshin Impact\Genshin Impact Game\GenshinImpact.exe"),
            ("Counter-Strike 2", "cs2.exe", r"C:\Program Files (x86)\Steam\steamapps\common\Counter-Strike Global Offensive\game\bin\win64\cs2.exe"),
            ("Grand Theft Auto V", "GTA5.exe", r"C:\Program Files\Rockstar Games\Grand Theft Auto V\GTA5.exe"),
        ];

        for (name, exe, path) in common {
            if Path::new(path).exists() {
                list.push(GameProfile::new_default(name, exe, path));
            }
        }
    }
}
