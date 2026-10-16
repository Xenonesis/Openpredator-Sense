# Batch 4: Macro Key Engine & Sound Profiles Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement 3-Group Macro Key Engine (recording, JSON storage, Win32 `SendInput` replay) and Acer TrueHarmony / DTS Sound Profiles in OpenPredatorSense.

**Architecture:** Create `macro_engine.rs` managing `%APPDATA%\OpenPredatorSense\macros.json` and detached thread keystroke playback via `SendInput`. Extend `win32_util.rs` with Sound Mode registry synchronization. Build dedicated KEY MACROS tab, real-time keystroke recording modal, and Sound Mode selector card in React.

**Tech Stack:** Rust 2024, Tauri v2, Windows Win32 `SendInput` API, React 19, TypeScript, Tailwind CSS v4, Lucide Icons.

**Spec:** `docs/superpowers/specs/2026-10-16-macro-engine-audio-design.md`

## Global Constraints

- Standalone executable on Windows 10/11 x64 with embedded `requireAdministrator` manifest.
- Macro playback must run on detached background threads to avoid blocking Tauri IPC or GUI responsiveness.
- All persistent configurations stored under `%APPDATA%\OpenPredatorSense\`.

---

### Task 1: Backend Data Models & Macro Engine Module

**Files:**
- Modify: `src-tauri/src/models.rs:130-170`
- Create: `src-tauri/src/macro_engine.rs`
- Modify: `src-tauri/src/win32_util.rs:30-60`

**Interfaces:**
- Produces: `MacroKeyAction`, `MacroGroup`.
- Produces: `macro_engine::get_macro_groups`, `macro_engine::save_macro_group`, `macro_engine::play_macro`.
- Produces: `win32_util::set_sound_mode`, `win32_util::get_sound_mode`.

- [ ] **Step 1: Add structs in `src-tauri/src/models.rs`**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroKeyAction {
    pub key: String,
    pub key_code: u16,
    pub is_down: bool,
    pub delay_ms: u32,
    pub extra_payload: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroGroup {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub trigger_hotkey: String,
    pub color_hex: String,
    pub actions: Vec<MacroKeyAction>,
    pub enabled: bool,
}
```

- [ ] **Step 2: Create `src-tauri/src/macro_engine.rs`**

Implement `%APPDATA%\OpenPredatorSense\macros.json` file storage with default P1, P2, P3 groups.
Implement `play_macro(group_id: &str)` spawning a thread executing `SendInput` with inter-key delays.

- [ ] **Step 3: Add Sound Mode registry helper in `src-tauri/src/win32_util.rs`**

- [ ] **Step 4: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/models.rs src-tauri/src/macro_engine.rs src-tauri/src/win32_util.rs
git commit -m "feat(macro): implement macro data structures, JSON storage, and Win32 replay engine"
```

---

### Task 2: Tauri IPC Handlers Registration

**Files:**
- Modify: `src-tauri/src/lib.rs:150-200`

**Interfaces:**
- Produces: Tauri commands: `get_macro_groups`, `save_macro_group`, `play_macro`, `set_sound_mode`, `get_sound_mode`.

- [ ] **Step 1: Declare `pub mod macro_engine;` in `src-tauri/src/lib.rs`**
- [ ] **Step 2: Expose commands in `src-tauri/src/lib.rs`**
- [ ] **Step 3: Register all in `tauri::generate_handler!`**
- [ ] **Step 4: Verify compilation**

Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(ipc): expose macro and sound mode Tauri commands"
```

---

### Task 3: Frontend Key Macros Tab & Recorder Modal

**Files:**
- Modify: `src/App.tsx`

**Interfaces:**
- Consumes: `get_macro_groups`, `save_macro_group`, `play_macro`.
- Produces: `KEY MACROS` sidebar tab, 3 Group cards (P1/P2/P3), and `showMacroRecordModal`.

- [ ] **Step 1: Add `Keyboard` icon to `lucide-react` imports and `TABS`**
- [ ] **Step 2: Add Macro state in `App.tsx`**
- [ ] **Step 3: Build `activeTab === "macros"` page view**
- [ ] **Step 4: Build `showMacroRecordModal` with real-time key logger**
- [ ] **Step 5: Verify build**

Run: `pnpm build`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/App.tsx
git commit -m "feat(ui): add KEY MACROS tab and real-time keystroke recorder modal"
```

---

### Task 4: Frontend Sound Mode Deck Integration

**Files:**
- Modify: `src/App.tsx` (Home tab)

**Interfaces:**
- Consumes: `set_sound_mode`, `get_sound_mode`.
- Produces: 6-preset Sound Mode selector card on Command Deck.

- [ ] **Step 1: Add sound mode state and 6 preset buttons**
- [ ] **Step 2: Wire 1-click apply handler**
- [ ] **Step 3: Verify build**

Run: `pnpm build`
Expected: PASS with 0 errors.

- [ ] **Step 4: Commit**

```bash
git add src/App.tsx
git commit -m "feat(ui): add Acer TrueHarmony sound mode selector to Command Deck"
```

---

### Task 5: End-to-End Build, Manifest Embedding & Verification

**Files:**
- Verify: `src-tauri/target/release/predator-sense-tauri.exe`

- [ ] **Step 1: Compile release binary (`pnpm tauri build --no-bundle`)**
- [ ] **Step 2: Embed `requireAdministrator` manifest (`mt.exe`)**
- [ ] **Step 3: Push to GitHub (`git push origin main`)**
