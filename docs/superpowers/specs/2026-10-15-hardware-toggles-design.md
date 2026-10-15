# OpenPredatorSense - Batch 1: Hardware BIOS & System Toggles Spec

**Date**: 2026-10-15  
**Component**: Hardware Settings, WMI Bridge, Cockpit GUI  
**Target Repository**: `predator-sense-tauri`  

---

## 1. Objective

Implement Batch 1 of the missing official PredatorSense features into OpenPredatorSense:
1. **Discrete GPU / MUX Switch Toggle**: Controls hardware display multiplexer between Optimus (Dynamic) and Discrete GPU Only via ACPI WMI `SetGamingMiscSetting(2)`.
2. **Reboot Confirmation Modal**: Tactical dialog warning the user of required system reboot upon changing display mode with "Restart Now" and "Restart Later" actions.
3. **30s Keyboard Backlight Idle Timeout**: Automatically cuts keyboard backlight power when no keystrokes are received for 30 seconds via ACPI `AcerGenericMethod.SetFunction(8)`.
4. **Power-off USB Device Charging**: Toggle allowing external USB devices to draw power while the notebook is in S3/S4/S5 power states via ACPI `AcerGenericMethod.SetFunction(9)`.
5. **Temperature Unit Switcher (°C / °F)**: Client-side mathematical conversion ($T_F = T_C \times 1.8 + 32$) dynamically formatting all tachometers, telemetry gauges, and monitoring cards with a quick-toggle badge on the cockpit dashboard.

---

## 2. Architecture & Data Flow

```
[ Frontend: Settings Modal / Quick Badge ]
       │
       ▼ (Tauri IPC commands: set_discrete_gpu, set_backlight_timeout, set_usb_charging, set_temp_unit)
[ Tauri Command Handlers (lib.rs) ]
       │
       ▼ (WmiCmd channel)
[ Dedicated COM STA Background Worker (hardware_manager.rs) ]
       │
       ▼ (In-process WMI execution)
[ WMI Bridge (wmi_bridge.rs) ]
       │
       ├── SetGamingMiscSetting(2 | (mode << 8))  ──> root\wmi:AcerGamingFunction (dGPU MUX)
       ├── AcerGenericMethod::SetFunction(8, val) ──> root\wmi:AcerGenericMethod (Backlight Timeout)
       └── AcerGenericMethod::SetFunction(9, val) ──> root\wmi:AcerGenericMethod (USB Charging)
```

---

## 3. Detailed Component Specifications

### 3.1 Backend Models (`models.rs`)
Extend `QuickSettings` structure:
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

### 3.2 WMI Bridge Methods (`wmi_bridge.rs`)
1. **Discrete GPU**:
   - `set_discrete_gpu(&self, enable: bool)`:
     ```rust
     let mode: u64 = if enable { 1 } else { 0 };
     let payload = 2 | (mode << 8);
     let _: Result<GmOutput32, _> = self.con.exec_instance_method::<AcerGamingFunction, _>(
         &self.path,
         "SetGamingMiscSetting",
         GmInput64 { gmInput: payload },
     );
     ```
   - `get_discrete_gpu(&self) -> bool`:
     Executes `GetGamingMiscSetting(2)`, checks `((raw >> 8) & 0xFF) == 1`.
2. **Backlight Timeout 30s**:
   - `set_backlight_timeout(&self, enable: bool)`:
     Calls `AcerGenericMethod.SetFunction` with opcode `0x00080000 | (if enable { 1 << 16 } else { 0 })`.
3. **Power-off USB Charging**:
   - `set_usb_charging(&self, enable: bool)`:
     Calls `AcerGenericMethod.SetFunction` with opcode `0x00090000 | (if enable { 1 << 16 } else { 0 })`.

### 3.3 Win32 Utilities (`win32_util.rs`)
- `reboot_system()`:
  Executes `shutdown.exe /r /t 5 /c "PredatorSense: Switching GPU Working Mode"` to allow graceful restart.

### 3.4 Frontend GUI (`App.tsx`)
1. **Settings Modal (`showSettings`)**:
   - 8-card grid layout with status indicators.
   - Discrete GPU card displays an amber warning badge `[ REBOOT REQ ]`.
   - Clicking dGPU toggle stages the change and immediately triggers `showRebootModal`.
2. **Reboot Modal (`showRebootModal`)**:
   - Tactical overlay dialog with dark backdrop blur.
   - Title: `[ MUX SWITCH: SYSTEM REBOOT REQUIRED ]`.
   - Message: "Switching display multiplexer requires a restart to re-route internal display pipeline to NVIDIA dGPU."
   - Button 1: `[ RESTART NOW ]` -> calls `invoke('system_reboot')`.
   - Button 2: `[ RESTART LATER ]` -> dismisses modal, leaving ACPI BIOS stage flag pending next boot.
3. **Tachometer Quick Unit Switch**:
   - Clickable `[ °C | °F ]` pill button on Tachometer header.
   - Toggle state converts all temperature values inline across the dashboard.

---

## 4. Verification & Testing Criteria

1. **Compilation**: `cargo check` and `pnpm build` pass with zero errors.
2. **WMI Method Integrity**: `GetGamingMiscSetting(2)` matches expected bitmask readback.
3. **Persistence**: Toggling options updates local state and reflects on next polling cycle.
4. **Reboot Flow**: Clicking "Restart Later" leaves app responsive; clicking "Restart Now" invokes scheduled restart command.
