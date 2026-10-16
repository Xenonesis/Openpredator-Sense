# OpenPredatorSense - Batch 3: Telemetry History & Trends Spec

**Date**: 2026-10-16  
**Component**: Hardware Telemetry, Historical Ring Buffers, SVG Waveform Visualization  
**Target Repository**: `predator-sense-tauri`  

---

## 1. Objective

Implement Batch 3 of the missing official PredatorSense features:
1. **Dual Ring Buffer Engine**:
   - In-memory rolling telemetry samples for 1-hour (`1H`, 60 samples at 60s intervals) and 24-hours (`24H`, 288 samples at 300s intervals).
   - Real-time rolling 20-sample live buffer (every 1.5s) for instant response on startup.
   - Metrics tracked: CPU Temperature (°C), CPU Usage (%), GPU Temperature (°C), GPU Usage (%), RAM Used (GB), and Network I/O.
2. **Statistical Aggregations**:
   - Compute real-time `min`, `max` (peak tracker), and `avg` for each metric over the selected timeframe.
3. **High-Performance Native SVG Area Chart**:
   - Responsive vector graph with graticule lines, gradient area fills, and bloom curves.
   - Metric switcher: CPU Temp, GPU Temp, CPU Usage, GPU Usage, RAM.
   - Full integration with global temperature unit switch (°C vs °F).
4. **Clear History Modal**:
   - Tactical confirmation dialog allowing the user to reset all historical and peak recording buffers.

---

## 2. Architecture & Data Structures

```
[ Hardware Manager Polling Thread (1.5s tick) ]
       │
       ├── Push live sample to internal rolling buffers
       │     ├── Real-time buffer: max 60 samples (every tick)
       │     ├── 1H buffer: max 60 samples (every 60s)
       │     └── 24H buffer: max 288 samples (every 300s)
       │
       ▼ (Tauri IPC commands: get_telemetry_history, clear_telemetry_history)
[ Frontend React Component (Monitoring Tab) ]
       │
       ├── SVG Area Chart with Graticules & Bloom Filter
       ├── Min / Max / Avg Metric Instrument Cards
       └── Clear History Modal Confirmation
```

### 2.1 Backend Data Structures (`models.rs`)

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

---

## 3. Backend Implementation

### 3.1 Ring Buffer State (`hardware_manager.rs`)
Inside `CachedState`:
- `samples_recent: VecDeque<TelemetrySample>` (max 60)
- `samples_1h: VecDeque<TelemetrySample>` (max 60)
- `samples_24h: VecDeque<TelemetrySample>` (max 288)
- `last_sample_1h: Instant`
- `last_sample_24h: Instant`

### 3.2 Aggregation Logic
Helper `calculate_metric_stats(values: &[f32], current: f32) -> MetricStats`:
- `min`: minimum non-zero value (or 0.0)
- `max`: highest recorded value
- `avg`: arithmetic mean

### 3.3 Tauri Commands (`lib.rs`)
- `get_telemetry_history(timeframe: String) -> TelemetryHistoryResponse`
- `clear_telemetry_history()`

---

## 4. Frontend GUI Implementation (`App.tsx`)

### 4.1 Diagnostics Historical Analysis Section
- Timeframe pill buttons: `1 HOUR` | `24 HOURS`.
- Metric selector tabs with color badges:
  - `CPU TEMP` (#00e5ff)
  - `GPU TEMP` (#b026ff)
  - `CPU USAGE` (#00f59b)
  - `GPU USAGE` (#ffaa00)
  - `RAM ALLOCATION` (#38bdf8)
- 4 Instrument Cards: Current, Minimum, Peak Maximum, and Average.
- `<svg viewBox="0 0 800 200">` Area Chart:
  - Gradient definition: `grad-history-{theme}`
  - Area polygon and stroke path with `filter="url(#bloom)"`.
  - Grid lines at 25%, 50%, 75%, 100%.

### 4.2 Clear History Dialog
- `showClearHistoryModal`:
  - Tactical warning message.
  - `[ RESET BUFFERS ]` invoking `clear_telemetry_history`.
  - `[ CANCEL ]`.

---

## 5. Verification & Testing Criteria

1. **Compilation**: `cargo check` and `pnpm build` pass with 0 errors.
2. **Buffer Operations**: Samples rotate correctly in `VecDeque` without exceeding capacity caps.
3. **Unit Consistency**: When `temp_unit_f` is active, min, max, avg, and chart labels convert accurately ($F = C \times 1.8 + 32$).
4. **Buffer Clearing**: Invoking `clear_telemetry_history` resets arrays and peak stats.
