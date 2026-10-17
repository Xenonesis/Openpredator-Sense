//! Macro Key Engine: Persistent macro groups in %APPDATA%\OpenPredatorSense\macros.json
//! and Win32 SendInput/keybd_event replay.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
pub use crate::models::{MacroGroup, MacroKeyAction};

fn get_macros_path() -> PathBuf {
    let base = env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"));
    let dir = base.join("OpenPredatorSense");
    let _ = fs::create_dir_all(&dir);
    dir.join("macros.json")
}

fn default_macro_groups() -> Vec<MacroGroup> {
    vec![
        MacroGroup {
            id: "group_1".to_string(),
            name: "P1 - Flight Profile".to_string(),
            kind: "turbo".to_string(),
            trigger_hotkey: "Ctrl+Alt+1".to_string(),
            color_hex: "#009BF0".to_string(),
            actions: Vec::new(),
            enabled: true,
        },
        MacroGroup {
            id: "group_2".to_string(),
            name: "P2 - Tactical Macro".to_string(),
            kind: "keystroke".to_string(),
            trigger_hotkey: "Ctrl+Alt+2".to_string(),
            color_hex: "#00FF00".to_string(),
            actions: Vec::new(),
            enabled: true,
        },
        MacroGroup {
            id: "group_3".to_string(),
            name: "P3 - Utility Launch".to_string(),
            kind: "app".to_string(),
            trigger_hotkey: "Ctrl+Alt+3".to_string(),
            color_hex: "#FF0000".to_string(),
            actions: Vec::new(),
            enabled: true,
        },
    ]
}

pub fn get_macro_groups() -> Vec<MacroGroup> {
    let path = get_macros_path();
    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(groups) = serde_json::from_str::<Vec<MacroGroup>>(&data) {
            return groups;
        }
    }
    default_macro_groups()
}

pub fn save_macro_group(group: MacroGroup) -> Result<(), String> {
    let mut groups = get_macro_groups();
    if let Some(pos) = groups.iter().position(|g| g.id == group.id) {
        groups[pos] = group;
    } else {
        groups.push(group);
    }
    let path = get_macros_path();
    let json = serde_json::to_string_pretty(&groups).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

#[link(name = "user32")]
unsafe extern "system" {
    fn keybd_event(bVk: u8, bScan: u8, dwFlags: u32, dwExtraInfo: usize);
}

const KEYEVENTF_KEYUP: u32 = 0x0002;

pub fn play_macro(group_id: String) -> Result<(), String> {
    let groups = get_macro_groups();
    let group = groups
        .into_iter()
        .find(|g| g.id == group_id)
        .ok_or_else(|| format!("Macro group not found: {}", group_id))?;

    if !group.enabled {
        return Ok(());
    }

    match group.kind.as_str() {
        "keystroke" => {
            let actions = group.actions;
            thread::spawn(move || {
                for action in actions {
                    if action.delay_ms > 0 {
                        thread::sleep(Duration::from_millis(action.delay_ms as u64));
                    }
                    let vk = (action.key_code & 0xFF) as u8;
                    let flags = if action.is_down { 0 } else { KEYEVENTF_KEYUP };
                    unsafe {
                        keybd_event(vk, 0, flags, 0);
                    }
                }
            });
        }
        "turbo" => {
            crate::hardware_manager::trigger_turbo_mode();
        }
        "app" => {
            if let Some(first_act) = group.actions.first() {
                if let Some(path) = &first_act.extra_payload {
                    let _ = std::process::Command::new(path).spawn();
                }
            }
        }
        _ => {}
    }
    Ok(())
}
