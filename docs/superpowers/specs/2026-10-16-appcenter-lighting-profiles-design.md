# OpenPredatorSense - Batch 2: Custom App Center & Lighting Profile Manager Spec

**Date**: 2026-10-16  
**Component**: App Center, Lighting Engine, Local File Persistence, File Dialogs  
**Target Repository**: `predator-sense-tauri`  

---

## 1. Objective

Implement Batch 2 of the missing official PredatorSense features:
1. **Custom App Center**:
   - Native Windows executable file picker (`.exe` browser dialog).
   - User shortcut registration with custom name, path, and category.
   - Shortcut deletion from the launcher grid.
   - Persistence across sessions in `%APPDATA%\OpenPredatorSense\custom_apps.json`.
2. **Lighting Profile Manager**:
   - Capture complete lighting state: 4 zone colors, active ACPI dynamic effect (Static, Breathing, Wave, Zoom, Shift, Neon, Rainbow), speed (1x-3x), direction (0-3), and backlight brightness (0-100%).
   - Save custom named profiles to `%APPDATA%\OpenPredatorSense\lighting_profiles.json`.
   - 1-click profile apply applying static bitmask (`SetGamingRgbKb`) and 16-byte ACPI payload (`SetGamingKBBacklight`).
   - Profile deletion.
   - Export and import profiles as `.json` files via native file dialogs.

---

## 2. Architecture & File Storage

### 2.1 File System Persistence
All persistent user configurations are stored under:
```
%APPDATA%\OpenPredatorSense\
├── custom_apps.json
└── lighting_profiles.json
```
If directory or files do not exist on startup, the backend creates them automatically with empty lists `[]`.

### 2.2 Data Schemas

#### `custom_apps.json`
```json
[
  {
    "id": "app_1776297600000",
    "name": "Cyberpunk 2077",
    "exe_path": "D:\\Games\\Cyberpunk 2077\\bin\\x64\\Cyberpunk2077.exe",
    "category": "GAME",
    "installed": true,
    "custom": true
  }
]
```

#### `lighting_profiles.json`
```json
[
  {
    "id": "lp_1776297600000",
    "name": "Stealth Amber",
    "zones": ["#FF8800", "#FF4400", "#FF2200", "#FF8800"],
    "effect": "breathing",
    "speed": 2,
    "direction": 1,
    "brightness": 80
  }
]
```

---

## 3. Backend Implementation

### 3.1 App Center Module (`app_center.rs`)
- `get_custom_apps() -> Vec<AppItem>`: Reads `%APPDATA%\OpenPredatorSense\custom_apps.json`.
- `save_custom_app(app: AppItem) -> Result<(), String>`: Appends or updates the app in JSON.
- `delete_custom_app(id: String) -> Result<(), String>`: Removes app by ID and writes back to JSON.
- `pick_executable_dialog() -> Result<Option<String>, String>`: Uses Windows Common Item Dialog / PowerShell OpenFileDialog to return selected `.exe` path.

### 3.2 Lighting Profiles Module (`lighting_profiles.rs`)
- `get_lighting_profiles() -> Vec<LightingProfile>`: Reads `%APPDATA%\OpenPredatorSense\lighting_profiles.json`.
- `save_lighting_profile(profile: LightingProfile) -> Result<(), String>`: Saves profile to JSON.
- `delete_lighting_profile(id: String) -> Result<(), String>`: Deletes profile from JSON.
- `export_profile_dialog(profile: LightingProfile) -> Result<(), String>`: Prompts save file dialog to write `.json`.
- `import_profile_dialog() -> Result<Option<LightingProfile>, String>`: Prompts open file dialog and deserializes profile.

### 3.3 Tauri Command Registration (`lib.rs`)
Expose:
- `pick_executable_dialog`
- `get_custom_apps`
- `save_custom_app`
- `delete_custom_app`
- `get_lighting_profiles`
- `save_lighting_profile`
- `delete_lighting_profile`
- `export_profile_dialog`
- `import_profile_dialog`

---

## 4. Frontend GUI Implementation (`App.tsx`)

### 4.1 App Center
- Top action button: `[ + REGISTER APPLICATION ]`.
- Modal `showAddAppModal`:
  - Input field for name.
  - Path input with `[ BROWSE .EXE ]` button.
  - Category selector (`GAME`, `UTILITY`, `MEDIA`, `BENCHMARK`).
  - `[ SAVE SHORTCUT ]` button.
- Custom cards in grid include red `[ × ]` delete button.

### 4.2 Lighting Profile Toolbar
- Header of Lighting Tab contains:
  - Preset / Custom Profile chips.
  - `[ + SAVE AS PROFILE ]` button opening name prompt modal.
  - `[ EXPORT ]` and `[ IMPORT ]` buttons.
  - Delete `[ × ]` on user-created profile chips.
- Selecting any profile immediately calls `applyRgbToAll` / `applyRgb` and `set_lighting_effect` to synchronize ACPI BIOS hardware.

---

## 5. Verification & Acceptance Criteria

1. **Compilation**: `cargo check` and `pnpm build` pass with 0 errors.
2. **Persistence**: Custom apps and profiles persist across application restarts.
3. **File Dialogs**: Browsing `.exe` populates the path without blocking the UI thread.
4. **Hardware Sync**: Loading a saved profile updates all 4 keyboard zones and ACPI dynamic effects.
