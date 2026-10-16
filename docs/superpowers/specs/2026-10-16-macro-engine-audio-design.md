# OpenPredatorSense - Batch 4: Macro Key Engine & Sound Profiles Spec

**Date**: 2026-10-16  
**Component**: Win32 Macro Replay Engine, Hotkey Hook, Acer Sound Mode  
**Target Repository**: `predator-sense-tauri`  

---

## 1. Objective

Implement Batch 4 of the missing official PredatorSense features:
1. **Macro Key Engine**:
   - 3 Macro Groups matching Predator OEM specifications:
     - Group 1 (P1): Indicator `#009BF0`, Default Hotkey `Ctrl+Alt+1`
     - Group 2 (P2): Indicator `#00FF00`, Default Hotkey `Ctrl+Alt+2`
     - Group 3 (P3): Indicator `#FF0000`, Default Hotkey `Ctrl+Alt+3`
   - Real-time keystroke sequence recorder with millisecond delay fidelity.
   - Action modes: `Keystrokes`, `Instant Turbo Overdrive`, `Launch Executable`.
   - Replay engine using Windows `SendInput` on a dedicated background thread.
   - Persistent storage in `%APPDATA%\OpenPredatorSense\macros.json`.
2. **Acer TrueHarmony / DTS Sound Profiles**:
   - 6 Tactical Sound Modes: `AUTOMATIC`, `SHOOTER`, `RPG`, `STRATEGY`, `MOVIE`, `MUSIC`, `VOICE`.
   - 1-Click switching on Command Deck with persistence in registry and system settings.

---

## 2. Architecture & Data Structures

```
[ Frontend: KEY MACROS Tab / Recorder Modal ]
       │
       ▼ (Tauri IPC commands: get_macro_groups, save_macro_group, play_macro, set_sound_mode)
[ Macro Engine & Win32 Hooks (macro_engine.rs, win32_util.rs) ]
       │
       ├── SendInput Win32 API (Background worker thread execution)
       ├── Local JSON storage: %APPDATA%\OpenPredatorSense\macros.json
       └── Registry sync: SOFTWARE\OEM\PredatorSense\SoundMode
```

### 2.1 Data Schemas (`models.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroKeyAction {
    pub key: String,         // Key name, e.g. "Q", "Enter", "Space"
    pub key_code: u16,       // Virtual key code (VK_*)
    pub is_down: bool,       // true for keydown, false for keyup
    pub delay_ms: u32,       // Inter-action delay in milliseconds
    pub extra_payload: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroGroup {
    pub id: String,          // "group_1", "group_2", "group_3"
    pub name: String,        // "P1 - Turbo Trigger", etc.
    pub kind: String,        // "keystroke" | "turbo" | "app"
    pub trigger_hotkey: String, // "Ctrl+Alt+1"
    pub color_hex: String,   // "#009BF0"
    pub actions: Vec<MacroKeyAction>,
    pub enabled: bool,
}
```

---

## 3. Backend Implementation

### 3.1 Macro Engine Module (`macro_engine.rs`)
- `get_macro_groups() -> Vec<MacroGroup>`: Loads `%APPDATA%\OpenPredatorSense\macros.json` (or creates defaults for P1, P2, P3).
- `save_macro_group(group: MacroGroup) -> Result<(), String>`: Serializes and saves updated group.
- `play_macro(group_id: String) -> Result<(), String>`: Spawns detached worker thread to replay `actions` using `SendInput` and `std::thread::sleep`.

### 3.2 Sound Mode Sync
- Registry write to `SOFTWARE\OEM\PredatorSense\SoundMode`.
- Expose `set_sound_mode(mode: String)` and `get_sound_mode() -> String`.

### 3.3 Tauri Command Registration (`lib.rs`)
Expose:
- `get_macro_groups`
- `save_macro_group`
- `play_macro`
- `set_sound_mode`

---

## 4. Frontend GUI Implementation (`App.tsx`)

### 4.1 Navigation
- Sidebar updated with **`KEY MACROS`** tab (`icon: Keyboard`).

### 4.2 Key Macros Page (`activeTab === "macros"`)
- 3 Tactical Macro Cards (P1, P2, P3).
- Hotkey badge, action timeline summary, `[ RECORD / EDIT ]`, and `[ TEST REPLAY ]` buttons.

### 4.3 Macro Recorder Modal (`showMacroRecordModal`)
- Capture keyboard events in real time.
- Timeline breadcrumb display with latency bars.
- Action type selector (`Keystroke Sequence`, `Instant Turbo`, `Launch App`).

### 4.4 Sound Mode Deck (Home Tab)
- 6 Preset buttons in Command Deck with active highlight and 1-click apply.

---

## 5. Verification & Acceptance Criteria

1. **Compilation**: `cargo check` and `pnpm build` pass with 0 errors.
2. **Replay Fidelity**: `play_macro` correctly simulates keydown/keyup events with timing delays.
3. **Persistence**: Macro groups and sound mode persist across laptop restarts in `%APPDATA%`.
