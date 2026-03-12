# Clance Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a lightweight Tauri-based system monitor widget that shows CPU, Memory, GPU, and top 5 processes in a glassmorphism-styled always-on-top widget.

**Architecture:** Rust backend collects system metrics via `sysinfo` crate (CPU/Memory/Processes) and platform-specific GPU APIs. Frontend is vanilla HTML/CSS/JS polling the backend every 2 seconds via Tauri IPC. Single frameless transparent window with drag support.

**Tech Stack:** Tauri v2, Rust, sysinfo crate, nvml-wrapper (Windows GPU), vanilla HTML/CSS/JS

---

### Task 0: Environment Setup

**Step 1: Install Rust toolchain**

Run: `winget install Rustlang.Rustup`
Then open a NEW terminal and verify:
Run: `rustc --version && cargo --version`
Expected: rustc 1.8x.x, cargo 1.8x.x

**Step 2: Install Tauri CLI**

Run: `cargo install tauri-cli --version "^2"`
Expected: `cargo-tauri` installed

**Step 3: Commit**

No files to commit — environment only.

---

### Task 1: Scaffold Tauri Project

**Files:**
- Create: `src-tauri/` (Rust backend)
- Create: `src/` (Web frontend)
- Create: `src/index.html`, `src/styles.css`, `src/main.js`

**Step 1: Initialize Tauri project**

Run from `c:/Dev/clance`:
```bash
cargo tauri init
```

Respond to prompts:
- App name: `clance`
- Window title: `Clance`
- Frontend dev URL: `http://localhost:1420` (default)
- Frontend dist: `../src`
- Dev command: (leave empty)
- Build command: (leave empty)

**Step 2: Create minimal frontend**

Create `src/index.html`:
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Clance</title>
  <link rel="stylesheet" href="styles.css">
</head>
<body>
  <div id="app">
    <p>Clance is running</p>
  </div>
  <script src="main.js"></script>
</body>
</html>
```

Create `src/styles.css`:
```css
* { margin: 0; padding: 0; box-sizing: border-box; }
body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; }
```

Create `src/main.js`:
```js
console.log('Clance loaded');
```

**Step 3: Configure Tauri window**

Edit `src-tauri/tauri.conf.json` — set window properties:
```json
{
  "app": {
    "windows": [
      {
        "title": "Clance",
        "width": 300,
        "height": 500,
        "decorations": false,
        "transparent": true,
        "alwaysOnTop": true,
        "resizable": false,
        "skipTaskbar": true
      }
    ]
  }
}
```

**Step 4: Verify app launches**

Run: `cargo tauri dev`
Expected: A small frameless transparent window appears.

**Step 5: Commit**

```bash
git add -A
git commit -m "feat: scaffold Tauri project with frameless transparent window"
```

---

### Task 2: Rust Backend — CPU & Memory Metrics

**Files:**
- Modify: `src-tauri/Cargo.toml` (add sysinfo dependency)
- Modify: `src-tauri/src/main.rs` (add Tauri commands)

**Step 1: Add sysinfo dependency**

Edit `src-tauri/Cargo.toml`:
```toml
[dependencies]
sysinfo = "0.33"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**Step 2: Implement system info commands**

Edit `src-tauri/src/main.rs`:
```rust
use serde::Serialize;
use sysinfo::System;
use std::sync::Mutex;
use tauri::State;

#[derive(Serialize, Clone)]
struct CpuInfo {
    name: String,
    usage: f32,
    cores: Vec<f32>,
}

#[derive(Serialize, Clone)]
struct MemoryInfo {
    total_gb: f64,
    used_gb: f64,
    usage_percent: f64,
}

struct AppState {
    sys: Mutex<System>,
}

#[tauri::command]
fn get_cpu_info(state: State<AppState>) -> CpuInfo {
    let mut sys = state.sys.lock().unwrap();
    sys.refresh_cpu_all();
    let cpus = sys.cpus();
    CpuInfo {
        name: cpus.first().map(|c| c.brand().to_string()).unwrap_or_default(),
        usage: sys.global_cpu_usage(),
        cores: cpus.iter().map(|c| c.cpu_usage()).collect(),
    }
}

#[tauri::command]
fn get_memory_info(state: State<AppState>) -> MemoryInfo {
    let mut sys = state.sys.lock().unwrap();
    sys.refresh_memory();
    let total = sys.total_memory() as f64;
    let used = sys.used_memory() as f64;
    MemoryInfo {
        total_gb: total / 1_073_741_824.0,
        used_gb: used / 1_073_741_824.0,
        usage_percent: (used / total) * 100.0,
    }
}

fn main() {
    let sys = System::new_all();
    tauri::Builder::default()
        .manage(AppState { sys: Mutex::new(sys) })
        .invoke_handler(tauri::generate_handler![get_cpu_info, get_memory_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Step 3: Verify it compiles**

Run: `cargo tauri dev`
Expected: Compiles and window appears (no UI change yet).

**Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/main.rs
git commit -m "feat: add CPU and memory info Tauri commands"
```

---

### Task 3: Rust Backend — Top Processes

**Files:**
- Modify: `src-tauri/src/main.rs`

**Step 1: Add process info command**

Add to `main.rs`:
```rust
#[derive(Serialize, Clone)]
struct ProcessInfo {
    name: String,
    cpu_usage: f32,
    memory_mb: f64,
    pid: u32,
}

#[tauri::command]
fn get_top_processes(state: State<AppState>) -> Vec<ProcessInfo> {
    let mut sys = state.sys.lock().unwrap();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let mut procs: Vec<_> = sys.processes().values()
        .map(|p| ProcessInfo {
            name: p.name().to_string_lossy().to_string(),
            cpu_usage: p.cpu_usage(),
            memory_mb: p.memory() as f64 / 1_048_576.0,
            pid: p.pid().as_u32(),
        })
        .collect();
    procs.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap());
    procs.truncate(5);
    procs
}
```

Register in handler: `tauri::generate_handler![get_cpu_info, get_memory_info, get_top_processes]`

**Step 2: Verify it compiles**

Run: `cargo tauri dev`

**Step 3: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat: add top processes Tauri command"
```

---

### Task 4: Rust Backend — GPU Info (Windows)

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/main.rs`

**Step 1: Add nvml-wrapper dependency**

Edit `src-tauri/Cargo.toml`:
```toml
[target.'cfg(windows)'.dependencies]
nvml-wrapper = "0.10"
```

**Step 2: Implement GPU info command**

Add to `main.rs`:
```rust
#[derive(Serialize, Clone)]
struct GpuInfo {
    name: String,
    usage_percent: u32,
    memory_total_mb: u64,
    memory_used_mb: u64,
    temperature: Option<u32>,
    available: bool,
}

#[tauri::command]
fn get_gpu_info() -> GpuInfo {
    #[cfg(windows)]
    {
        match try_get_nvidia_gpu() {
            Some(info) => info,
            None => GpuInfo {
                name: "No GPU detected".into(),
                usage_percent: 0,
                memory_total_mb: 0,
                memory_used_mb: 0,
                temperature: None,
                available: false,
            },
        }
    }
    #[cfg(not(windows))]
    {
        GpuInfo {
            name: "GPU monitoring not supported".into(),
            usage_percent: 0,
            memory_total_mb: 0,
            memory_used_mb: 0,
            temperature: None,
            available: false,
        }
    }
}

#[cfg(windows)]
fn try_get_nvidia_gpu() -> Option<GpuInfo> {
    use nvml_wrapper::Nvml;
    let nvml = Nvml::init().ok()?;
    let device = nvml.device_by_index(0).ok()?;
    let utilization = device.utilization_rates().ok()?;
    let memory = device.memory_info().ok()?;
    let temp = device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu).ok();
    Some(GpuInfo {
        name: device.name().ok().unwrap_or_default(),
        usage_percent: utilization.gpu,
        memory_total_mb: memory.total / 1_048_576,
        memory_used_mb: memory.used / 1_048_576,
        temperature: temp,
        available: true,
    })
}
```

Register: `tauri::generate_handler![get_cpu_info, get_memory_info, get_top_processes, get_gpu_info]`

**Step 3: Verify it compiles**

Run: `cargo tauri dev`

**Step 4: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/main.rs
git commit -m "feat: add GPU info command with NVML support"
```

---

### Task 5: Frontend — Glassmorphism Widget UI

**Files:**
- Modify: `src/index.html`
- Modify: `src/styles.css`

**Step 1: Build HTML structure**

Replace `src/index.html`:
```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Clance</title>
  <link rel="stylesheet" href="styles.css">
</head>
<body>
  <div id="widget" data-tauri-drag-region>
    <div class="section" id="cpu-section">
      <div class="section-header" data-toggle="cpu">
        <span class="toggle-icon">▼</span>
        <span class="section-title">CPU</span>
        <span class="section-value" id="cpu-value">--%</span>
      </div>
      <div class="section-body" id="cpu-body">
        <div class="progress-bar"><div class="progress-fill" id="cpu-bar"></div></div>
        <div class="section-detail" id="cpu-name"></div>
      </div>
    </div>

    <div class="section" id="memory-section">
      <div class="section-header" data-toggle="memory">
        <span class="toggle-icon">▼</span>
        <span class="section-title">Memory</span>
        <span class="section-value" id="memory-value">--/--GB</span>
      </div>
      <div class="section-body" id="memory-body">
        <div class="progress-bar"><div class="progress-fill" id="memory-bar"></div></div>
      </div>
    </div>

    <div class="section" id="gpu-section">
      <div class="section-header" data-toggle="gpu">
        <span class="toggle-icon">▼</span>
        <span class="section-title">GPU</span>
        <span class="section-value" id="gpu-value">--%</span>
      </div>
      <div class="section-body" id="gpu-body">
        <div class="progress-bar"><div class="progress-fill" id="gpu-bar"></div></div>
        <div class="section-detail" id="gpu-vram"></div>
        <div class="section-detail" id="gpu-name"></div>
      </div>
    </div>

    <div class="section" id="process-section">
      <div class="section-header" data-toggle="process">
        <span class="toggle-icon">▼</span>
        <span class="section-title">Top Processes</span>
        <span class="section-value"></span>
      </div>
      <div class="section-body" id="process-body">
        <ul id="process-list"></ul>
      </div>
    </div>
  </div>
  <script src="main.js"></script>
</body>
</html>
```

**Step 2: Write glassmorphism CSS**

Replace `src/styles.css`:
```css
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  background: transparent;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  color: #e0e0e0;
  user-select: none;
  overflow: hidden;
}

#widget {
  padding: 16px;
  border-radius: 16px;
  background: rgba(20, 20, 30, 0.75);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.08);
  min-height: 100vh;
}

.section {
  margin-bottom: 12px;
}

.section:last-child {
  margin-bottom: 0;
}

.section-header {
  display: flex;
  align-items: center;
  cursor: pointer;
  padding: 6px 0;
  gap: 8px;
}

.toggle-icon {
  font-size: 10px;
  transition: transform 0.2s;
  width: 14px;
  text-align: center;
}

.section.collapsed .toggle-icon {
  transform: rotate(-90deg);
}

.section-title {
  font-size: 13px;
  font-weight: 600;
  flex: 1;
}

.section-value {
  font-size: 13px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
}

.section-body {
  padding: 6px 0 2px 22px;
  overflow: hidden;
  transition: max-height 0.2s, opacity 0.2s;
  max-height: 200px;
  opacity: 1;
}

.section.collapsed .section-body {
  max-height: 0;
  opacity: 0;
  padding: 0 0 0 22px;
}

.progress-bar {
  height: 6px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.5s ease, background-color 0.5s ease;
  background: linear-gradient(90deg, #4facfe, #00f2fe);
  width: 0%;
}

.section-detail {
  font-size: 11px;
  color: #999;
  margin-top: 4px;
}

#process-list {
  list-style: none;
  font-size: 12px;
}

#process-list li {
  display: flex;
  justify-content: space-between;
  padding: 2px 0;
  font-variant-numeric: tabular-nums;
}

#process-list li .proc-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-right: 8px;
}

#process-list li .proc-cpu {
  min-width: 50px;
  text-align: right;
  color: #4facfe;
}
```

**Step 3: Verify UI renders**

Run: `cargo tauri dev`
Expected: Glassmorphism card with sections visible.

**Step 4: Commit**

```bash
git add src/index.html src/styles.css
git commit -m "feat: glassmorphism widget UI with collapsible sections"
```

---

### Task 6: Frontend — Data Binding & Polling

**Files:**
- Modify: `src/main.js`

**Step 1: Implement data polling**

Replace `src/main.js`:
```js
const { invoke } = window.__TAURI__.core;

function formatGb(gb) {
  return gb < 10 ? gb.toFixed(1) : Math.round(gb);
}

function setBar(id, percent) {
  const el = document.getElementById(id);
  el.style.width = Math.min(100, percent) + '%';
  if (percent > 80) {
    el.style.background = 'linear-gradient(90deg, #f5576c, #ff6b6b)';
  } else if (percent > 60) {
    el.style.background = 'linear-gradient(90deg, #f093fb, #f5576c)';
  } else {
    el.style.background = 'linear-gradient(90deg, #4facfe, #00f2fe)';
  }
}

async function updateCpu() {
  const info = await invoke('get_cpu_info');
  document.getElementById('cpu-value').textContent = Math.round(info.usage) + '%';
  document.getElementById('cpu-name').textContent = info.name;
  setBar('cpu-bar', info.usage);
}

async function updateMemory() {
  const info = await invoke('get_memory_info');
  document.getElementById('memory-value').textContent =
    formatGb(info.used_gb) + '/' + formatGb(info.total_gb) + 'GB';
  setBar('memory-bar', info.usage_percent);
}

async function updateGpu() {
  const info = await invoke('get_gpu_info');
  const section = document.getElementById('gpu-section');
  if (!info.available) {
    section.style.display = 'none';
    return;
  }
  section.style.display = '';
  document.getElementById('gpu-value').textContent = info.usage_percent + '%';
  document.getElementById('gpu-name').textContent = info.name;
  const usedGb = (info.memory_used_mb / 1024).toFixed(1);
  const totalGb = (info.memory_total_mb / 1024).toFixed(1);
  document.getElementById('gpu-vram').textContent = 'VRAM: ' + usedGb + '/' + totalGb + 'GB';
  setBar('gpu-bar', (info.memory_used_mb / info.memory_total_mb) * 100);
}

async function updateProcesses() {
  const procs = await invoke('get_top_processes');
  const list = document.getElementById('process-list');
  list.innerHTML = procs.map((p, i) =>
    `<li><span class="proc-name">${i + 1}. ${p.name}</span><span class="proc-cpu">${p.cpu_usage.toFixed(1)}%</span></li>`
  ).join('');
}

async function update() {
  try {
    await Promise.all([updateCpu(), updateMemory(), updateGpu(), updateProcesses()]);
  } catch (e) {
    console.error('Update failed:', e);
  }
}

// Collapsible sections
document.querySelectorAll('.section-header').forEach(header => {
  header.addEventListener('click', (e) => {
    if (e.target.closest('[data-tauri-drag-region]') === e.target) return;
    header.closest('.section').classList.toggle('collapsed');
  });
});

// Start polling
update();
setInterval(update, 2000);
```

**Step 2: Enable Tauri IPC in config**

Ensure `src-tauri/tauri.conf.json` has the `withGlobalTauri` option set to true under `app` so that `window.__TAURI__` is available.

**Step 3: Verify live data**

Run: `cargo tauri dev`
Expected: Widget shows live CPU, Memory, GPU, and process data updating every 2 seconds.

**Step 4: Commit**

```bash
git add src/main.js src-tauri/tauri.conf.json
git commit -m "feat: data binding and 2-second polling"
```

---

### Task 7: Window Drag & Tauri Config Polish

**Files:**
- Modify: `src-tauri/tauri.conf.json`

**Step 1: Finalize Tauri config**

Ensure these settings in `tauri.conf.json`:
- `decorations: false` (no title bar)
- `transparent: true`
- `alwaysOnTop: true`
- `resizable: false`
- `skipTaskbar: true`
- `width: 300, height: 500`
- `withGlobalTauri: true`

The `data-tauri-drag-region` attribute on `#widget` div handles window dragging.

**Step 2: Verify drag works**

Run: `cargo tauri dev`
Expected: Can drag the widget by grabbing the background area. Clicking section headers toggles collapse without dragging.

**Step 3: Commit**

```bash
git add src-tauri/tauri.conf.json
git commit -m "feat: finalize window config — frameless, transparent, always-on-top"
```

---

### Task 8: README & GitHub Actions

**Files:**
- Create: `README.md`
- Create: `.github/workflows/release.yml`

**Step 1: Write README**

```markdown
# Clance

> Claude + Glance — A lightweight system monitor widget for your personal PC.

![Windows](https://img.shields.io/badge/Windows-0078D6?logo=windows&logoColor=white)
![macOS](https://img.shields.io/badge/macOS-000000?logo=apple&logoColor=white)

Clance is a desktop widget that shows your system's vital signs at a glance:

- **CPU** usage with per-core details
- **Memory** usage
- **GPU** utilization & VRAM (NVIDIA on Windows)
- **Top 5 processes** by CPU usage

Built with [Tauri](https://tauri.app) for minimal resource footprint (~15MB RAM).

## Install

Download the latest release from [GitHub Releases](https://github.com/jun0-ds/clance/releases).

## Build from Source

```bash
# Prerequisites: Rust, Node.js
cargo install tauri-cli --version "^2"
cargo tauri dev    # development
cargo tauri build  # production build
```

## License

MIT
```

**Step 2: Create GitHub Actions release workflow**

Create `.github/workflows/release.yml`:
```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - platform: windows-latest
            target: x86_64-pc-windows-msvc
          - platform: macos-latest
            target: aarch64-apple-darwin

    runs-on: ${{ matrix.platform }}

    steps:
      - uses: actions/checkout@v4

      - uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          targets: ${{ matrix.target }}

      - uses: actions/setup-node@v4
        with:
          node-version: 22

      - name: Install Tauri CLI
        run: cargo install tauri-cli --version "^2"

      - name: Build
        run: cargo tauri build --target ${{ matrix.target }}

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: clance-${{ matrix.target }}
          path: |
            src-tauri/target/${{ matrix.target }}/release/bundle/**/*

  release:
    needs: build
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/download-artifact@v4
      - uses: softprops/action-gh-release@v2
        with:
          files: clance-*/**/*
```

**Step 3: Commit**

```bash
git add README.md .github/workflows/release.yml
git commit -m "docs: add README and GitHub Actions release workflow"
```

---

### Task 9: Final Integration Test & Push

**Step 1: Full build test**

Run: `cargo tauri build`
Expected: Builds successfully, produces installer in `src-tauri/target/release/bundle/`

**Step 2: Run the built app**

Launch the built executable and verify:
- [ ] Widget appears as frameless transparent window
- [ ] CPU/Memory/GPU data updates every 2 seconds
- [ ] Sections collapse/expand on click
- [ ] Window is draggable
- [ ] Always on top works

**Step 3: Push all commits**

```bash
git push origin main
```

**Step 4: Tag first release (optional)**

```bash
git tag v0.1.0
git push origin v0.1.0
```
Expected: GitHub Actions builds and creates a release.
