# Batch 2: Custom App Center & Lighting Profile Manager Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Custom App Center registration (file picker, shortcut adder, deletion) and Lighting Profile Manager (save, load, delete, export, import) with `%APPDATA%` JSON persistence in OpenPredatorSense.

**Architecture:** Extend Rust backend with persistent JSON storage in `%APPDATA%\OpenPredatorSense\`, native Windows file picker dialogs, and Tauri IPC handlers. Build tactical Add App modal, custom app deletion, and Lighting Profile Toolbar in React.

**Tech Stack:** Rust 2024, Tauri v2, Windows Win32 Common Dialogs, React 19, TypeScript, Tailwind CSS v4, Lucide Icons.

**Spec:** `docs/superpowers/specs/2026-10-16-appcenter-lighting-profiles-design.md`

## Global Constraints

- Standalone executable on Windows 10/11 x64 with embedded `requireAdministrator` manifest.
- All persistent configuration files stored under `%APPDATA%\OpenPredatorSense\`.
- Native file dialogs must run on worker threads or non-blocking tasks to prevent freezing the UI.
- Applying a lighting profile must synchronize both static zones (`SetGamingRgbKb`) and ACPI dynamic effects (`SetGamingKBBacklight`).

---

### Task 1: Backend Data Models & App Center File Persistence

**Files:**
- Modify: `src-tauri/src/models.rs:70-95`
- Modify: `src-tauri/src/app_center.rs:1-120`

**Interfaces:**
- Produces: `AppItem.custom: bool`.
- Produces: `LightingProfile` struct with `id, name, zones, effect, speed, direction, brightness`.
- Produces: `app_center::get_custom_apps`, `app_center::save_custom_app`, `app_center::delete_custom_app`, `app_center::pick_executable_dialog`.

- [ ] **Step 1: Update models in `src-tauri/src/models.rs`**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppItem {
    pub id: String,
    pub name: String,
    pub exe_path: String,
    pub category: String,
    pub installed: bool,
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
```

- [ ] **Step 2: Implement storage & file picker in `src-tauri/src/app_center.rs`**

```rust
use std::fs;
use std::path::PathBuf;

fn get_app_data_dir() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("C:\\ProgramData"));
    path.push("OpenPredatorSense");
    let _ = fs::create_dir_all(&path);
    path
}

fn get_custom_apps_path() -> PathBuf {
    let mut p = get_app_data_dir();
    p.push("custom_apps.json");
    p
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
    // Uses PowerShell OpenFileDialog for native Windows file selection
    let script = r#"
    Add-Type -AssemblyName System.Windows.Forms
    $f = New-Object System.Windows.Forms.OpenFileDialog
    $f.Filter = "Executable Files (*.exe)|*.exe|All Files (*.*)|*.*"
    $f.Title = "Select Application Executable"
    if ($f.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
        Write-Output $f.FileName
    }
    "#;
    let output = std::process::Command::new("powershell")
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
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/models.rs src-tauri/src/app_center.rs
git commit -m "feat(app_center): add custom app models, JSON persistence, and executable file picker"
```

---

### Task 2: Lighting Profiles Backend Module & Tauri Commands

**Files:**
- Create: `src-tauri/src/lighting_profiles.rs`
- Modify: `src-tauri/src/lib.rs:1-170`

**Interfaces:**
- Produces: `lighting_profiles::get_profiles`, `lighting_profiles::save_profile`, `lighting_profiles::delete_profile`, `lighting_profiles::export_profile`, `lighting_profiles::import_profile`.
- Produces: Tauri commands: `get_custom_apps`, `save_custom_app`, `delete_custom_app`, `pick_executable_dialog`, `get_lighting_profiles`, `save_lighting_profile`, `delete_lighting_profile`, `export_lighting_profile`, `import_lighting_profile`.

- [ ] **Step 1: Create `src-tauri/src/lighting_profiles.rs`**

Implement JSON persistence in `%APPDATA%\OpenPredatorSense\lighting_profiles.json` and file export/import dialogs.

- [ ] **Step 2: Expose commands in `src-tauri/src/lib.rs`**

Register all App Center and Lighting Profile commands in `tauri::generate_handler!`.

- [ ] **Step 3: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lighting_profiles.rs src-tauri/src/lib.rs
git commit -m "feat(lighting): add persistent lighting profile engine and Tauri IPC handlers"
```

---

### Task 3: Frontend Custom App Center UI

**Files:**
- Modify: `src/App.tsx:2000-2100`

**Interfaces:**
- Consumes: `get_custom_apps`, `save_custom_app`, `delete_custom_app`, `pick_executable_dialog`.
- Produces: Full reactive App Center with user shortcut registration and card deletion.

- [ ] **Step 1: Add App Center state & fetch logic in `App.tsx`**

Fetch custom apps on mount and merge with detected OEM tools.

- [ ] **Step 2: Add `[ + REGISTER APPLICATION ]` button and `showAddAppModal`**

Modal with App Name input, Path input with `[ BROWSE .EXE ]` button, category dropdown, and save action.

- [ ] **Step 3: Add delete `[ × ]` button on custom cards**

Clicking deletes shortcut from `%APPDATA%` and updates state.

- [ ] **Step 4: Verify build**

Run: `pnpm build`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/App.tsx
git commit -m "feat(ui): add custom app registration modal, file browser, and shortcut deletion in App Center"
```

---

### Task 4: Frontend Lighting Profile Manager UI

**Files:**
- Modify: `src/App.tsx:1500-1700`

**Interfaces:**
- Consumes: `get_lighting_profiles`, `save_lighting_profile`, `delete_lighting_profile`, `export_lighting_profile`, `import_lighting_profile`.
- Produces: Interactive Profile Toolbar in Lighting tab.

- [ ] **Step 1: Add profile state & load logic in `App.tsx`**

Load user lighting profiles on mount.

- [ ] **Step 2: Implement Profile Toolbar in Lighting Tab**

Display profile chips (OEM presets + user custom profiles), `[ + SAVE AS PROFILE ]` button, `[ EXPORT ]`, and `[ IMPORT ]` buttons.

- [ ] **Step 3: Implement 1-click Profile Apply**

Selecting a profile applies its 4-zone colors and invokes `set_lighting_effect` with saved effect, speed, direction, and brightness.

- [ ] **Step 4: Verify build**

Run: `pnpm build`
Expected: PASS with 0 errors.

- [ ] **Step 5: Commit**

```bash
git add src/App.tsx
git commit -m "feat(ui): implement lighting profile manager with save, load, export, and import"
```

---

### Task 5: End-to-End Build, Manifest Embedding & Verification

**Files:**
- Verify: `src-tauri/target/release/predator-sense-tauri.exe`

- [ ] **Step 1: Compile full release binary**

Run: `pnpm tauri build --no-bundle`

- [ ] **Step 2: Embed `requireAdministrator` manifest**

Run `mt.exe` to embed manifest.

- [ ] **Step 3: Smoke test live binary**

Test:
1. Adding a custom `.exe` via file picker dialog.
2. Launching custom app.
3. Deleting custom app.
4. Saving a custom lighting profile.
5. Switching profiles.

- [ ] **Step 4: Push to GitHub**

Run: `git push origin main`
