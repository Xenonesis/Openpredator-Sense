# ⚡ OpenPredatorSense

**OpenPredatorSense** is a lightweight, open-source, high-performance replacement for Acer's official PredatorSense software (v3 / v4). Built in **Rust** using **Tauri 2**, **React 19**, and **Tailwind CSS v4**, it communicates directly with Acer's hardware ACPI BIOS via WMI without requiring OEM background telemetry services (`PSSvc.exe`, `PSAgent.exe`, `PSAdminAgent.exe`).

[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange?logo=rust)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-v2.0-24C8D8?logo=tauri)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-19.1-61DAFB?logo=react)](https://react.dev/)
[![TailwindCSS](https://img.shields.io/badge/Tailwind_CSS-v4.0-38BDF8?logo=tailwindcss)](https://tailwindcss.com/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## 🚀 Key Advantages Over Official PredatorSense

| Metric / Capability | Official Acer PredatorSense v3/v4 | OpenPredatorSense (This Project) |
| :--- | :--- | :--- |
| **Executable Size** | `~95 MB` (UWP + MSI bundle) | **`~5.1 MB`** (Standalone portable binary) |
| **Memory Footprint** | `~150 MB - 250 MB` RAM | **`~35 MB - 42 MB`** RAM |
| **Startup Time** | `~4.5s - 8.0s` (Multi-service initialization) | **`< 0.25s`** (Instantaneous cold launch) |
| **Background Services** | Requires 4+ background services (`PSSvc.exe`, etc.) | **Zero background services** (Pure in-process COM STA thread) |
| **UAC Bypass Check** | Restricts Named Pipe IPC to `C:\Program Files\WindowsApps` | **Embedded `requireAdministrator` manifest** (Direct kernel ACPI WMI access) |
| **Window Frame** | Bulky, non-resizable | **Frameless tactical cockpit**, full Aero snap, double-click maximize/restore |

---

## 🎮 Features

### 1. Flight Operation Modes (Command Deck)
- **`Quiet`**: 25W acoustic whisper profile for productivity & silent battery operation.
- **`Default`**: Balanced daily thermal curve with automatic fan throttling.
- **`Extreme`**: GPU core boost with dynamic aggressive fan curves.
- **`Turbo`**: Maximum power limits, +100MHz GPU Core / +200MHz VRAM overclock, and immediate **`7,100+ RPM` dual fan roar**.
- **Mode Lighting Pulse**: Switching modes automatically pulses the keyboard in the mode's signature hue:
  - Quiet: `#FFFFFF` (Ice White)
  - Default: `#00AEC7` (Electric Cyan)
  - Extreme: `#FF8800` (Amber Orange)
  - Turbo: `#FF1100` (Crimson Fire)
- **"Introduction of Modes" Modal**: Authentic replica explaining power states and cooling curves.

### 2. AeroBlade™ 3D Calibrated Tachometers & Fan Control
- **Dual Speedometer Gauges**: Calibrated `0 to 7,500 RPM` dial instruments with needle sweep, live tabular RPM readouts, and turbine blades spinning proportionally to physical RPM.
- **Auto Mode**: Direct ACPI curve synchronization (`SetGamingFanBehavior(0x00410009)`).
- **Max Mode**: Engages dual max cooling at physical limit (`SetGamingFanBehavior(0x00820009)`).
- **Custom Mode**: Independent linear potentiometer sliders for CPU (Index 1) and GPU (Index 4) with 30% to 100% duty cycle tuning and stepper buttons (`-5%`, `+5%`).
- **Acer CoolBoost™ Overdrive**: ACPI circuit breaker toggle raising fan duty cycle by ~10% over BIOS curve (`SetFunction(0x00010007)`).

### 3. Pulsar RGB Keyboard (4-Zone Chiclet Deck)
- **Interactive Chiclet Visualizer**: 4 distinct physical sectors:
  - Sector 1: `WASD + Left Modifiers (Tab, Caps, Shift, Ctrl, Q/W/E/R)`
  - Sector 2: `Central Alphas (T/Y/U/G/H/J)`
  - Sector 3: `Right Alphas + Navigation + Arrow Keys + Enter`
  - Sector 4: `10-Key Numpad + Dedicated PredatorSense Key`
- **Static Illumination**: Direct `SetGamingRgbKb` bitmasked controls (`0x01`, `0x02`, `0x04`, `0x08`).
- **7 Dynamic ACPI Effects**: Static, Breathing, Wave, Zoom, Shift, Neon Cycle, Rainbow Flow via fixed 16-byte ACPI array (`SetGamingKBBacklight`).
- **Velocity Radar & Direction Vectors**: 1x, 2x, 3x speeds + East, West, South, North wave propagation.
- **Brightness Calibration**: Discrete 0%, 25%, 50%, 75%, 100% stops + smooth slider.

### 4. Telemetry Oscilloscope & Diagnostics
- **CPU Computation Core**: Real-time package utilization %, ACPI package temp (°C), and live frequency (MHz).
- **Discrete GPU Engine**: Live NVAPI core clock (MHz), VRAM clock (MHz), ACPI core temp (°C), and Optimus idle indicator with informative tooltip.
- **Physical Memory Pool**: Allocated vs Total DDR5 RAM.
- **Multi-Channel Network Oscilloscope**: Real-time rolling waveform graph for Ethernet and Wi-Fi Download & Upload rates.

### 5. Autonomic Game Sync Engine
- Scans Steam libraries (`libraryfolders.vdf`, `appmanifest_*.acf`) and Epic Games manifests for installed titles.
- Custom game executable registration.
- Background process watcher automatically triggers configured thermal and lighting profiles when a game launches, and reverts back when closed.

### 6. Tactical Utility Launchpad
- 1-Click executable launcher detecting installed gaming tools:
  - Acer Care Center
  - Killer Control Center
  - NVIDIA App / GeForce Experience
  - Steam
  - Discord
  - OBS Studio
  - XSplit Gamecaster

### 7. Cockpit Hardware Toggles (BIOS & Windows)
- **Windows Key Gaming Lock**: Blocks Windows key via ACPI Profile & Low-Level `WH_KEYBOARD_LL` Hook.
- **Sticky Keys Suppression**: Suppresses 5x Shift key prompt via `SystemParametersInfoW(SPI_SETSTICKYKEYS)`.
- **LCD 3ms Panel Overdrive**: Toggles panel response time boost via ACPI (`SetGamingProfile(16 | (1 << 48))`).
- **BIOS Boot Roar Chime**: Toggles power-on post audio chime (`SetGamingMiscSetting(6 | (1 << 8))`).

---

## 🛠️ Reverse-Engineered ACPI BIOS WMI Architecture

OpenPredatorSense communicates directly with `root\wmi:AcerGamingFunction` and `root\wmi:AcerGenericMethod`:

| ACPI WMI Method | Parameters / Formula | Hardware Function |
| :--- | :--- | :--- |
| `GetGamingSysInfo(gmInput)` | `0x0101` (CPU Temp), `0x0201` (CPU RPM)<br>`0x0301` (GPU Temp), `0x0601` (GPU RPM) | Real hardware sensor telemetry (`(output >> 8) & 0xFFFF`) |
| `SetGamingFanBehavior(gmInput)` | `0x00410009` (Auto)<br>`0x00820009` (Max Roar)<br>`0x00C30009` (Custom Manual) | Dual-fan firmware cooling curve state |
| `SetGamingFanSpeed(gmInput)` | `fan_id \| ((pct) << 16)`<br>*(CPU: ID 1, GPU: ID 4 on 12th Gen)* | Manual fan duty cycle percentage (30% - 100%) |
| `SetGamingMiscSetting(gmInput)` | `1 \| (mode_id << 8)`<br>*(Quiet: 0, Default: 1, Extreme: 2, Turbo: 3)* | ACPI System Performance / Overclock Profile |
| `SetGamingRgbKb(gmInput)` | `zone_mask \| (r << 8) \| (g << 16) \| (b << 24)`<br>*(Masks: Z1=0x01, Z2=0x02, Z3=0x04, Z4=0x08)* | Static 4-Zone keyboard RGB colorization |
| `SetGamingKBBacklight(gmInput)` | Fixed 16-Byte Array (`UInt8Array[16]`) | Dynamic FX mode, speed, direction, brightness |
| `SetGamingProfile(gmInput)` | `2 \| (lock << 24)` (Win Key Lock)<br>`16 \| (enable << 48)` (LCD 3ms Overdrive) | System & display hardware profile flags |
| `AcerGenericMethod::SetFunction`| `7 \| (enable << 16)` | Acer CoolBoost™ duty cycle ceiling boost |

---

## 📦 Building from Source

### Prerequisites
1. **Rust**: 1.80+ (2024 edition) with MSVC toolchain (`rustup default stable-x86_64-pc-windows-msvc`).
2. **Node.js / pnpm**: Node.js 18+ and `pnpm` (`npm install -g pnpm`).
3. **Windows 10 / 11 x64**.
4. **Windows SDK**: `mt.exe` (Manifest Tool) to embed the UAC elevation manifest.

### Build Steps

```bash
# 1. Clone repository
git clone https://github.com/Xenonesis/Openpredator-Sense.git
cd Openpredator-Sense

# 2. Install frontend dependencies
pnpm install

# 3. Build React production bundle
pnpm build

# 4. Compile optimized Rust release binary
pnpm tauri build --no-bundle

# 5. Embed requireAdministrator UAC manifest
& "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\mt.exe" -manifest "src-tauri\app.manifest" -outputresource:"src-tauri\target\release\predator-sense-tauri.exe;#1"
```

The resulting standalone executable is located at:
```
src-tauri/target/release/predator-sense-tauri.exe
```

---

## 🛡️ License

MIT License. Designed and built for Acer Predator laptop users seeking maximum gaming performance, clean aesthetics, and zero background bloat.
