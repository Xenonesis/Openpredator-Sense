# Batch 1: Hardware BIOS & System Toggles Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Discrete GPU (MUX switch), Reboot Confirmation Modal, 30s Backlight Idle Timeout, Power-off USB Charging, and Temperature Unit Switcher (°C / °F) in OpenPredatorSense.

**Architecture:** Extend in-process COM WMI bridge on STA background thread with ACPI methods on `AcerGamingFunction` and `AcerGenericMethod`. Expose non-blocking Tauri IPC commands. Build 8-card tactical hardware modal, reboot modal, and dynamic °C/°F conversion in React.

**Tech Stack:** Rust 2024, Tauri v2, Windows WMI (`root\wmi:AcerGamingFunction`), React 19, TypeScript, Tailwind CSS v4, Lucide Icons.

**Spec:** `docs/superpowers/specs/2026-10-15-hardware-toggles-design.md`

## Global Constraints

- Standalone executable must run on Windows 10/11 x64 with embedded `requireAdministrator` manifest.
- Zero external child processes during normal polling; in-process COM STA thread only.
- Discrete GPU MUX changes require user confirmation modal before system reboot.
- Temperature conversion must apply consistently across gauges, cards, tachometers, and charts without altering backend Celsius telemetry.

---

### Task 1: Backend Data Models & State

**Files:**
- Modify: `src-tauri/src/models.rs:20-50`
- Modify: `src-tauri/src/hardware_manager.rs:15-60`

**Interfaces:**
- Produces: `QuickSettings` fields: `discrete_gpu: bool`, `backlight_timeout_30s: bool`, `usb_charging: bool`, `temp_unit_f: bool`.
- Produces: `WmiCmd` variants: `SetDiscreteGpu(bool)`, `SetBacklightTimeout(bool)`, `SetUsbCharging(bool)`, `SetTempUnit(bool)`.

- [ ] **Step 1: Update `QuickSettings` in `src-tauri/src/models.rs`**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickSettings {
    pub win_key_locked: bool,
    pub sticky_keys_enabled: bool,
    pub lcd_overdrive: bool,
    pub boot_sound: bool,
    pub discrete_gpu: bool,
    pub backlight_timeout_30s: bool,
    pub usb_charging: bool,
    pub temp_unit_f: bool,
}
```

- [ ] **Step 2: Update `CachedState` and defaults in `src-tauri/src/hardware_manager.rs`**

Add fields to `CachedState`:
```rust
pub discrete_gpu: bool,
pub backlight_timeout_30s: bool,
pub usb_charging: bool,
pub temp_unit_f: bool,
```

Add variants to `WmiCmd`:
```rust
SetDiscreteGpu(bool),
SetBacklightTimeout(bool),
SetUsbCharging(bool),
SetTempUnit(bool),
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: PASS (warnings only)

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/models.rs src-tauri/src/hardware_manager.rs
git commit -m "feat(models): add discrete GPU, backlight timeout, usb charge, and temp unit fields"
```

---

### Task 2: WMI Bridge & ACPI Hardware Implementation

**Files:**
- Modify: `src-tauri/src/wmi_bridge.rs:240-300`
- Modify: `src-tauri/src/hardware_manager.rs:250-320`

**Interfaces:**
- Consumes: `AcerGamingFunction.SetGamingMiscSetting`, `AcerGenericMethod.SetFunction`.
- Produces: `AcerWmi::set_discrete_gpu`, `AcerWmi::get_discrete_gpu`, `AcerWmi::set_backlight_timeout`, `AcerWmi::set_usb_charging`.

- [ ] **Step 1: Add WMI methods in `src-tauri/src/wmi_bridge.rs`**

```rust
/// Set Discrete GPU (MUX switch): 0 = Optimus / Dynamic, 1 = Discrete GPU only
pub fn set_discrete_gpu(&self, enable: bool) {
    let mode: u64 = if enable { 1 } else { 0 };
    let payload = 2 | (mode << 8);
    let _: Result<GmOutput32, _> = self.con.exec_instance_method::<AcerGamingFunction, _>(
        &self.path,
        "SetGamingMiscSetting",
        GmInput64 { gmInput: payload },
    );
}

/// Read Discrete GPU mode from ACPI: GetGamingMiscSetting(2)
pub fn get_discrete_gpu(&self) -> bool {
    let res: Result<GmOutput64, WMIError> = self.con.exec_instance_method::<AcerGamingFunction, _>(
        &self.path,
        "GetGamingMiscSetting",
        GmInput32 { gmInput: 2 },
    );
    match res {
        Ok(out) => {
            let raw = out.gmOutput.unwrap_or(0);
            ((raw >> 8) & 0xFF) == 1
        }
        Err(_) => false,
    }
}

/// Set 30s Keyboard Backlight Idle Timeout via AcerGenericMethod::SetFunction(8)
pub fn set_backlight_timeout(&self, enable: bool) {
    let payload: u64 = 0x00080000 | (if enable { 1u64 << 16 } else { 0 });
    let _: Result<GmOutput32, _> = self.con.exec_instance_method::<AcerGenericMethod, _>(
        &self.generic_path,
        "SetFunction",
        GmInput64 { gmInput: payload },
    );
}

/// Set Power-off USB Device Charging via AcerGenericMethod::SetFunction(9)
pub fn set_usb_charging(&self, enable: bool) {
    let payload: u64 = 0x00090000 | (if enable { 1u64 << 16 } else { 0 });
    let _: Result<GmOutput32, _> = self.con.exec_instance_method::<AcerGenericMethod, _>(
        &self.generic_path,
        "SetFunction",
        GmInput64 { gmInput: payload },
    );
}
```

- [ ] **Step 2: Handle commands in `src-tauri/src/hardware_manager.rs`**

In `handle_cmd`:
```rust
WmiCmd::SetDiscreteGpu(enable) => {
    wmi.set_discrete_gpu(enable);
    state.write().discrete_gpu = enable;
}
WmiCmd::SetBacklightTimeout(enable) => {
    wmi.set_backlight_timeout(enable);
    state.write().backlight_timeout_30s = enable;
}
WmiCmd::SetUsbCharging(enable) => {
    wmi.set_usb_charging(enable);
    state.write().usb_charging = enable;
}
WmiCmd::SetTempUnit(use_f) => {
    state.write().temp_unit_f = use_f;
}
```

In `poll_hardware`:
Query `wmi.get_discrete_gpu()` and update `state.write().discrete_gpu`.

- [ ] **Step 3: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/wmi_bridge.rs src-tauri/src/hardware_manager.rs
git commit -m "feat(wmi): implement discrete GPU MUX, backlight timeout, and USB charge ACPI controls"
```

---

### Task 3: Tauri IPC Commands & Win32 System Reboot

**Files:**
- Modify: `src-tauri/src/win32_util.rs:1-30`
- Modify: `src-tauri/src/lib.rs:100-160`

**Interfaces:**
- Produces: Tauri commands: `set_discrete_gpu`, `set_backlight_timeout`, `set_usb_charging`, `set_temp_unit`, `system_reboot`.

- [ ] **Step 1: Add `reboot_system` in `src-tauri/src/win32_util.rs`**

```rust
use std::process::Command;

pub fn reboot_system() -> Result<(), String> {
    Command::new("shutdown.exe")
        .args(["/r", "/t", "3", "/c", "PredatorSense: Switching GPU Working Mode"])
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}
```

- [ ] **Step 2: Expose commands in `src-tauri/src/lib.rs`**

```rust
#[tauri::command]
fn set_discrete_gpu(enabled: bool, mgr: State<Arc<HardwareManager>>) {
    mgr.send(WmiCmd::SetDiscreteGpu(enabled));
}

#[tauri::command]
fn set_backlight_timeout(enabled: bool, mgr: State<Arc<HardwareManager>>) {
    mgr.send(WmiCmd::SetBacklightTimeout(enabled));
}

#[tauri::command]
fn set_usb_charging(enabled: bool, mgr: State<Arc<HardwareManager>>) {
    mgr.send(WmiCmd::SetUsbCharging(enabled));
}

#[tauri::command]
fn set_temp_unit(use_f: bool, mgr: State<Arc<HardwareManager>>) {
    mgr.send(WmiCmd::SetTempUnit(use_f));
}

#[tauri::command]
fn system_reboot() -> Result<(), String> {
    win32_util::reboot_system()
}
```
Register all in `generate_handler!`.

- [ ] **Step 3: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/win32_util.rs src-tauri/src/lib.rs
git commit -m "feat(ipc): expose hardware toggles and system_reboot commands"
```

---

### Task 4: Frontend UI (8-Card Settings Modal, Reboot Modal, Quick Unit Badge)

**Files:**
- Modify: `src/App.tsx:25-50` (TypeScript interface `QuickSettings`)
- Modify: `src/App.tsx:500-600` (Tachometer header °C/°F badge & conversion)
- Modify: `src/App.tsx:1900-2100` (8-Card Settings Modal & Reboot Confirmation Modal)

**Interfaces:**
- Consumes: Tauri IPC commands from Task 3.
- Produces: Full reactive UI for 8 hardware settings with MUX reboot confirmation modal and dynamic unit conversion.

- [ ] **Step 1: Update `QuickSettings` interface in `src/App.tsx`**

```typescript
export interface QuickSettings {
  win_key_locked: boolean;
  sticky_keys_enabled: boolean;
  lcd_overdrive: boolean;
  boot_sound: boolean;
  discrete_gpu: boolean;
  backlight_timeout_30s: boolean;
  usb_charging: boolean;
  temp_unit_f: boolean;
}
```

- [ ] **Step 2: Add temperature conversion helper**

```typescript
const formatTemp = (celsius: number, isF: boolean): string => {
  if (isF) {
    return `${Math.round(celsius * 1.8 + 32)}°F`;
  }
  return `${Math.round(celsius)}°C`;
};
```
Apply `formatTemp` to Tachometer gauges, CPU/GPU cards, and telemetry chips.

- [ ] **Step 3: Implement 8-Card Settings Modal**

Add 4 new cards in 2x4 grid:
- Discrete GPU Card with `REBOOT REQ` amber badge and click handler triggering `showRebootModal`.
- 30s Backlight Idle Timeout card.
- Power-off USB Device Charging card.
- Temperature Scale card (`CELSIUS (°C)` vs `FAHRENHEIT (°F)`).

- [ ] **Step 4: Implement Reboot Modal (`showRebootModal`)**

```tsx
{showRebootModal && (
  <div className="fixed inset-0 bg-black/85 backdrop-blur-md z-50 flex items-center justify-center p-4">
    <div className="bg-[#0a0f19] border-2 border-amber-500/80 rounded-2xl p-6 max-w-md w-full shadow-[0_0_50px_rgba(245,158,11,0.25)]">
      <div className="flex items-center gap-3 mb-4">
        <ShieldAlert className="w-6 h-6 text-amber-400 animate-pulse" />
        <h3 className="text-base font-bold text-amber-300 font-mono tracking-wider">
          [ MUX SWITCH: SYSTEM REBOOT REQUIRED ]
        </h3>
      </div>
      <p className="text-xs text-slate-300 font-mono leading-relaxed mb-6">
        Switching between NVIDIA Discrete GPU and Dynamic Optimus requires a system restart for BIOS display multiplexer initialization.
      </p>
      <div className="flex gap-3">
        <button
          onClick={() => invoke("system_reboot")}
          className="flex-1 py-2.5 bg-amber-500 hover:bg-amber-400 text-black font-bold font-mono text-xs rounded-lg transition-all shadow-[0_0_15px_rgba(245,158,11,0.4)]"
        >
          RESTART NOW
        </button>
        <button
          onClick={() => setShowRebootModal(false)}
          className="px-4 py-2.5 bg-slate-800 hover:bg-slate-700 text-slate-300 font-mono text-xs rounded-lg border border-slate-600 transition-all"
        >
          LATER
        </button>
      </div>
    </div>
  </div>
)}
```

- [ ] **Step 5: Add Tachometer Header `[ °C | °F ]` Quick Switch**

Add clickable toggle badge in the Tachometer header that triggers `set_temp_unit`.

- [ ] **Step 6: Verify frontend build**

Run: `pnpm build` in `predator-sense-tauri`
Expected: PASS with 0 type errors.

- [ ] **Step 7: Commit**

```bash
git add src/App.tsx
git commit -m "feat(ui): add 8-card settings grid, MUX reboot modal, and dynamic temp unit conversion"
```

---

### Task 5: End-to-End Build, Manifest Embedding & Verification

**Files:**
- Verify: `src-tauri/target/release/predator-sense-tauri.exe`

- [ ] **Step 1: Compile full release binary**

Run: `pnpm tauri build --no-bundle`

- [ ] **Step 2: Embed `requireAdministrator` manifest**

Run:
```powershell
& "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\mt.exe" -manifest "src-tauri\app.manifest" -outputresource:"src-tauri\target\release\predator-sense-tauri.exe;#1"
```

- [ ] **Step 3: Launch live executable & smoke test**

Launch `predator-sense-tauri.exe`, verify:
1. Settings modal opens with 8 cards.
2. Clicking Discrete GPU toggle brings up the MUX reboot modal.
3. Clicking `LATER` preserves staged state without freezing.
4. Clicking `[ °C | °F ]` pill badge dynamically updates all temperature gauges to Fahrenheit.
5. Fans and telemetry continue live updates.

- [ ] **Step 4: Commit and push**

```bash
git push origin main
```
