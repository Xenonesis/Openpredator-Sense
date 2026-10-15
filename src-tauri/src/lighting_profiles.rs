//! Lighting Profile Manager: Persistent lighting profiles in %APPDATA%\OpenPredatorSense

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
pub use crate::models::LightingProfile;

fn get_app_data_dir() -> PathBuf {
    let base = env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"));
    let dir = base.join("OpenPredatorSense");
    let _ = fs::create_dir_all(&dir);
    dir
}

fn get_lighting_profiles_path() -> PathBuf {
    get_app_data_dir().join("lighting_profiles.json")
}

pub fn get_lighting_profiles() -> Vec<LightingProfile> {
    let path = get_lighting_profiles_path();
    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(profiles) = serde_json::from_str::<Vec<LightingProfile>>(&data) {
            return profiles;
        }
    }
    Vec::new()
}

pub fn save_lighting_profile(profile: LightingProfile) -> Result<(), String> {
    let mut profiles = get_lighting_profiles();
    profiles.retain(|p| p.id != profile.id);
    profiles.push(profile);
    let path = get_lighting_profiles_path();
    let json = serde_json::to_string_pretty(&profiles).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn delete_lighting_profile(id: String) -> Result<(), String> {
    let mut profiles = get_lighting_profiles();
    profiles.retain(|p| p.id != id);
    let path = get_lighting_profiles_path();
    let json = serde_json::to_string_pretty(&profiles).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn export_profile_dialog(profile: LightingProfile) -> Result<(), String> {
    let script = r#"
    Add-Type -AssemblyName System.Windows.Forms
    $f = New-Object System.Windows.Forms.SaveFileDialog
    $f.Filter = "JSON Files (*.json)|*.json|All Files (*.*)|*.*"
    $f.Title = "Export Lighting Profile"
    $f.FileName = "lighting_profile.json"
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
        return Ok(());
    }
    let json = serde_json::to_string_pretty(&profile).map_err(|e| e.to_string())?;
    fs::write(path_str, json).map_err(|e| e.to_string())
}

pub fn import_profile_dialog() -> Result<Option<LightingProfile>, String> {
    let script = r#"
    Add-Type -AssemblyName System.Windows.Forms
    $f = New-Object System.Windows.Forms.OpenFileDialog
    $f.Filter = "JSON Files (*.json)|*.json|All Files (*.*)|*.*"
    $f.Title = "Import Lighting Profile"
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
        return Ok(None);
    }
    let data = fs::read_to_string(&path_str).map_err(|e| e.to_string())?;
    let profile = serde_json::from_str::<LightingProfile>(&data).map_err(|e| e.to_string())?;
    Ok(Some(profile))
}
