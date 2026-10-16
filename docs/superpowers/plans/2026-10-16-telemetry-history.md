# Batch 3: Telemetry History & Trends Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement in-memory 1H / 24H rolling telemetry ring buffers, Min/Max/Avg statistical aggregation, SVG waveform area charting, and Clear History dialog in OpenPredatorSense.

**Architecture:** Extend `hardware_manager.rs` background worker with timestamped `VecDeque` ring buffers. Compute real-time min/max/average stats per metric. Expose Tauri IPC commands. Build native SVG area chart with graticules, gradient fills, and unit conversion in React.

**Tech Stack:** Rust 2024, Tauri v2, React 19, TypeScript, Tailwind CSS v4, Lucide Icons.

**Spec:** `docs/superpowers/specs/2026-10-16-telemetry-history-design.md`

## Global Constraints

- Standalone executable on Windows 10/11 x64 with embedded `requireAdministrator` manifest.
- Pure in-memory ring buffers; zero disk writes during periodic sampling.
- Temperature values must convert dynamically according to `temp_unit_f` setting.
- Maximum buffer sizes: 60 samples (1H) and 288 samples (24H).

---

### Task 1: Backend Data Models & Ring Buffer State

**Files:**
- Modify: `src-tauri/src/models.rs:100-130`
- Modify: `src-tauri/src/hardware_manager.rs:40-100`

**Interfaces:**
- Produces: `TelemetrySample`, `MetricStats`, `TelemetryHistoryResponse`.
- Produces: Ring buffers in `CachedState`.

- [ ] **Step 1: Add structs in `src-tauri/src/models.rs`**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySample {
    pub timestamp: u64,
    pub cpu_temp: f32,
    pub cpu_usage: f32,
    pub gpu_temp: f32,
    pub gpu_usage: f32,
    pub ram_used_gb: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricStats {
    pub min: f32,
    pub max: f32,
    pub avg: f32,
    pub current: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryHistoryResponse {
    pub samples: Vec<TelemetrySample>,
    pub cpu_temp: MetricStats,
    pub cpu_usage: MetricStats,
    pub gpu_temp: MetricStats,
    pub gpu_usage: MetricStats,
    pub ram: MetricStats,
}
```

- [ ] **Step 2: Add ring buffers to `CachedState` in `src-tauri/src/hardware_manager.rs`**

```rust
use std::collections::VecDeque;
use std::time::Instant;

pub samples_recent: VecDeque<TelemetrySample>,
pub samples_1h: VecDeque<TelemetrySample>,
pub samples_24h: VecDeque<TelemetrySample>,
pub last_sample_1h: Instant,
pub last_sample_24h: Instant,
```

- [ ] **Step 3: Sample data in background polling loop**

Push current sample to `samples_recent` (cap 60), to `samples_1h` every 60s (cap 60), and to `samples_24h` every 300s (cap 288).

- [ ] **Step 4: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/models.rs src-tauri/src/hardware_manager.rs
git commit -m "feat(telemetry): add ring buffer models and background sample collection"
```

---

### Task 2: History Aggregations & Tauri Commands

**Files:**
- Modify: `src-tauri/src/hardware_manager.rs:350-450`
- Modify: `src-tauri/src/lib.rs:140-180`

**Interfaces:**
- Produces: `HardwareManager::get_telemetry_history`, `HardwareManager::clear_telemetry_history`.
- Produces: Tauri commands `get_telemetry_history`, `clear_telemetry_history`.

- [ ] **Step 1: Implement aggregation logic in `src-tauri/src/hardware_manager.rs`**

Helper:
```rust
fn calc_metric(vals: &[f32], cur: f32) -> MetricStats {
    if vals.is_empty() {
        return MetricStats { min: cur, max: cur, avg: cur, current: cur };
    }
    let min = vals.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = vals.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let avg = vals.iter().sum::<f32>() / (vals.len() as f32);
    MetricStats { min, max, avg, current: cur }
}
```

Implement `get_telemetry_history(timeframe: &str)` and `clear_telemetry_history()`.

- [ ] **Step 2: Expose commands in `src-tauri/src/lib.rs`**

```rust
#[tauri::command]
fn get_telemetry_history(timeframe: String, mgr: State<Arc<HardwareManager>>) -> TelemetryHistoryResponse {
    mgr.get_telemetry_history(&timeframe)
}

#[tauri::command]
fn clear_telemetry_history(mgr: State<Arc<HardwareManager>>) {
    mgr.clear_telemetry_history();
}
```
Register in `tauri::generate_handler!`.

- [ ] **Step 3: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/hardware_manager.rs src-tauri/src/lib.rs
git commit -m "feat(telemetry): implement history aggregation and Tauri IPC commands"
```

---

### Task 3: Frontend SVG Area Chart Component

**Files:**
- Modify: `src/App.tsx:500-600`

**Interfaces:**
- Produces: `<TelemetryAreaChart>` SVG component with bloom curves, gradient fills, and grid lines.

- [ ] **Step 1: Build `TelemetryAreaChart` component in `src/App.tsx`**

Takes `samples: number[]`, `minVal`, `maxVal`, `themeColor`, `unit`.
Renders calibrated SVG viewBox with grid lines and area polygon.

- [ ] **Step 2: Verify build**

Run: `pnpm build`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/App.tsx
git commit -m "feat(ui): add native SVG TelemetryAreaChart component"
```

---

### Task 4: Diagnostics Tab Integration & Clear History Modal

**Files:**
- Modify: `src/App.tsx:1800-2000`
- Modify: `src/App.tsx:2500-2600`

**Interfaces:**
- Consumes: `get_telemetry_history`, `clear_telemetry_history`.
- Produces: Timeframe switcher (`1H`/`24H`), metric filter tabs, min/max/avg cards, and Clear History dialog.

- [ ] **Step 1: Add history state in `App.tsx`**

`timeframe: "1h" | "24h"`, `selectedMetric: "cpu_temp" | "gpu_temp" | "cpu_usage" | "gpu_usage" | "ram"`, `historyData: TelemetryHistoryResponse | null`.

- [ ] **Step 2: Integrate into Diagnostics (Monitoring) Tab**

Render Timeframe pill, Metric tabs, 4 Readout cards (`CURRENT`, `MIN`, `MAX`, `AVG`), and `TelemetryAreaChart`.

- [ ] **Step 3: Add Clear History Modal**

`showClearHistoryModal` confirmation dialog.

- [ ] **Step 4: Verify build**

Run: `pnpm build`
Expected: PASS with 0 errors.

- [ ] **Step 5: Commit**

```bash
git add src/App.tsx
git commit -m "feat(ui): integrate telemetry history analytics and clear history modal"
```

---

### Task 5: End-to-End Build, Manifest Embedding & Verification

**Files:**
- Verify: `src-tauri/target/release/predator-sense-tauri.exe`

- [ ] **Step 1: Compile release binary**

Run: `pnpm tauri build --no-bundle`

- [ ] **Step 2: Embed `requireAdministrator` manifest**

Run: `mt.exe`

- [ ] **Step 3: Push to GitHub**

Run: `git push origin main`
