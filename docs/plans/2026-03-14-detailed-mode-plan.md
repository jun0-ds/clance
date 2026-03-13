# Detailed Mode + AI Usage Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Simple/Detailed mode toggle that expands the widget horizontally to show Claude Code token usage, and replace the color palette with Nord theme.

**Architecture:** Widget stays as a single Tauri window. Mode toggle changes window width (300→600px) and reveals a right-side AI panel via CSS flexbox. Rust backend reads `~/.claude/stats-cache.json` and `~/.claude/projects/*/*.jsonl` for token data. Frontend uses separate polling intervals for system (2s) and AI (30s/5min) data.

**Tech Stack:** Tauri v2, Rust (serde_json, std::fs), Vanilla HTML/CSS/JS, Nord color palette.

---

### Task 1: Nord Color Palette — CSS

**Files:**
- Modify: `src/styles.css`
- Modify: `src/main.js` (opacity slider background color)

**Step 1: Replace all colors in styles.css**

Replace these exact color values throughout `src/styles.css`:

```css
/* Body */
color: #e0e0e0;  →  color: #eceff4;

/* #widget */
background: rgba(20, 20, 30, 0.75);  →  background: rgba(46, 52, 64, 0.85);
border: 1px solid rgba(255, 255, 255, 0.08);  →  border: 1px solid rgba(216, 222, 233, 0.08);

/* #opacity-slider */
background: rgba(255, 255, 255, 0.1);  →  background: rgba(216, 222, 233, 0.1);

/* #opacity-slider::-webkit-slider-thumb */
background: #e0e0e0;  →  background: #eceff4;

/* #minimize-btn, #close-btn */
color: #e0e0e0;  →  color: #eceff4;

/* #minimize-btn:hover */
color: #4facfe;  →  color: #88c0d0;

/* #close-btn:hover */
color: #f5576c;  →  color: #bf616a;

/* .progress-bar — remove gradient, use solid */
background: linear-gradient(90deg, #4facfe, #00f2fe);  →  background: #88c0d0;

/* .sort-tab */
background: rgba(255, 255, 255, 0.06);  →  background: rgba(216, 222, 233, 0.06);
border: 1px solid rgba(255, 255, 255, 0.08);  →  border: 1px solid rgba(216, 222, 233, 0.08);
color: #999;  →  color: #d8dee9;

/* .sort-tab:hover */
background: rgba(255, 255, 255, 0.1);  →  background: rgba(216, 222, 233, 0.1);
color: #ccc;  →  color: #eceff4;

/* .sort-tab.active */
background: rgba(79, 172, 254, 0.2);  →  background: rgba(136, 192, 208, 0.2);
border-color: rgba(79, 172, 254, 0.3);  →  border-color: rgba(136, 192, 208, 0.3);
color: #4facfe;  →  color: #88c0d0;

/* .proc-value */
color: #4facfe;  →  color: #88c0d0;

/* .progress-track */
background: rgba(255, 255, 255, 0.06);  →  background: rgba(216, 222, 233, 0.06);

/* scrollbar thumb */
background: rgba(255, 255, 255, 0.1);  →  background: rgba(216, 222, 233, 0.1);
```

**Step 2: Update setBar() in main.js**

Replace the color logic in `setBar()` function:

```javascript
function setBar(id, percent) {
  const el = document.getElementById(id);
  el.style.width = Math.min(100, percent) + '%';
  if (percent > 80) {
    el.style.background = '#bf616a';
  } else if (percent > 60) {
    el.style.background = '#ebcb8b';
  } else {
    el.style.background = '#88c0d0';
  }
}
```

**Step 3: Update opacity slider handler in main.js**

Replace the rgba color in the opacity slider handler:

```javascript
opacitySlider.addEventListener('input', () => {
  const val = opacitySlider.value / 100;
  document.getElementById('widget').style.background = `rgba(46, 52, 64, ${val})`;
});
```

**Step 4: Build and verify visually**

Run: `cargo tauri dev`
Expected: Widget appears with Nord colors — muted blue bars, no gradients, warm dark background.

**Step 5: Commit**

```bash
git add src/styles.css src/main.js
git commit -m "feat: replace color palette with Nord theme"
```

---

### Task 2: Mode Toggle UI — HTML + CSS + JS

**Files:**
- Modify: `src/index.html`
- Modify: `src/styles.css`
- Modify: `src/main.js`

**Step 1: Add mode toggle button and flex container to index.html**

In `index.html`, wrap existing content in a left panel and add right panel placeholder.

Change `<div id="widget">` structure to:

```html
<div id="widget" data-tauri-drag-region>
  <div id="title-bar" data-tauri-drag-region>
    <span class="app-name">Clance</span>
    <button id="mode-toggle" title="Toggle Detailed mode">S</button>
    <input type="range" id="opacity-slider" min="20" max="100" value="75" title="Opacity">
    <button id="minimize-btn" title="Minimize to tray">&minus;</button>
    <button id="close-btn" title="Close">&times;</button>
  </div>
  <div id="panels">
    <div id="panel-system">
      <!-- ALL existing sections (cpu, memory, gpu, processes) stay here unchanged -->
    </div>
    <div id="panel-ai" class="hidden">
      <div class="ai-section-title">AI Usage</div>
      <div id="ai-content">
        <div class="ai-loading">No data</div>
      </div>
    </div>
  </div>
</div>
```

The existing CPU/Memory/GPU/Processes sections move inside `#panel-system` with zero changes.

**Step 2: Add CSS for mode toggle, panels, and right panel**

Add to `src/styles.css`:

```css
/* Mode toggle */
#mode-toggle {
  background: rgba(216, 222, 233, 0.06);
  border: 1px solid rgba(216, 222, 233, 0.08);
  border-radius: 4px;
  color: #d8dee9;
  font-size: 10px;
  font-weight: 700;
  padding: 1px 5px;
  cursor: pointer;
  margin-right: 4px;
  transition: all 0.15s;
}

#mode-toggle:hover {
  background: rgba(216, 222, 233, 0.12);
  color: #eceff4;
}

#mode-toggle.active {
  background: rgba(163, 190, 140, 0.2);
  border-color: rgba(163, 190, 140, 0.3);
  color: #a3be8c;
}

/* Panels layout */
#panels {
  display: flex;
  gap: 16px;
}

#panel-system {
  flex: 0 0 268px;
  min-width: 0;
}

#panel-ai {
  flex: 1;
  min-width: 0;
  border-left: 1px solid rgba(216, 222, 233, 0.06);
  padding-left: 16px;
}

#panel-ai.hidden {
  display: none;
}

/* AI panel styles */
.ai-section-title {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 1px;
  color: #a3be8c;
  margin-bottom: 12px;
}

.ai-loading {
  font-size: 11px;
  color: #4c566a;
}

.ai-today {
  margin-bottom: 12px;
}

.ai-today-total {
  font-size: 20px;
  font-weight: 700;
  color: #eceff4;
  font-variant-numeric: tabular-nums;
}

.ai-today-label {
  font-size: 11px;
  color: #4c566a;
  margin-bottom: 4px;
}

.ai-model-row {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  padding: 2px 0;
  color: #d8dee9;
}

.ai-model-tokens {
  font-variant-numeric: tabular-nums;
  color: #a3be8c;
}

.ai-stats {
  font-size: 11px;
  color: #4c566a;
  margin: 8px 0 12px;
}

/* Sparkline */
.sparkline {
  display: flex;
  align-items: flex-end;
  gap: 4px;
  height: 32px;
  margin-bottom: 4px;
}

.sparkline-bar {
  flex: 1;
  background: #a3be8c;
  border-radius: 2px 2px 0 0;
  min-height: 2px;
  transition: height 0.3s ease;
}

.sparkline-labels {
  display: flex;
  gap: 4px;
  font-size: 9px;
  color: #4c566a;
}

.sparkline-labels span {
  flex: 1;
  text-align: center;
}

.ai-trend-title {
  font-size: 11px;
  color: #4c566a;
  margin-bottom: 6px;
}

/* Recent sessions */
.ai-sessions-title {
  font-size: 11px;
  color: #4c566a;
  margin: 12px 0 6px;
}

.ai-session-row {
  display: flex;
  justify-content: space-between;
  font-size: 11px;
  padding: 2px 0;
  color: #d8dee9;
}

.ai-session-tokens {
  font-variant-numeric: tabular-nums;
  color: #a3be8c;
}
```

**Step 3: Add mode toggle logic in main.js**

Add after the opacity slider handler:

```javascript
// Mode toggle
const SIMPLE_WIDTH = 300;
const DETAILED_WIDTH = 600;
let detailedMode = false;
let aiSummaryInterval = null;
let aiHistoryInterval = null;

const modeToggle = document.getElementById('mode-toggle');
const panelAi = document.getElementById('panel-ai');

modeToggle.addEventListener('click', async () => {
  detailedMode = !detailedMode;
  modeToggle.textContent = detailedMode ? 'D' : 'S';
  modeToggle.classList.toggle('active', detailedMode);
  panelAi.classList.toggle('hidden', !detailedMode);

  const win = getCurrentWindow();
  const width = detailedMode ? DETAILED_WIDTH : SIMPLE_WIDTH;
  await win.setSize(new LogicalSize(width, document.getElementById('widget').scrollHeight));

  if (detailedMode) {
    startAiPolling();
  } else {
    stopAiPolling();
  }

  await resizeToContent();
});
```

**Step 4: Update WIDGET_WIDTH usage in resizeToContent**

Change `resizeToContent()` to use dynamic width:

```javascript
async function resizeToContent() {
  const widget = document.getElementById('widget');
  const height = widget.scrollHeight;
  const width = detailedMode ? DETAILED_WIDTH : SIMPLE_WIDTH;
  try {
    await getCurrentWindow().setSize(new LogicalSize(width, height));
  } catch (e) {
    // ignore resize errors
  }
}
```

**Step 5: Build and verify mode toggle**

Run: `cargo tauri dev`
Expected: Click S → switches to D, widget widens to 600px showing empty AI panel. Click D → back to S, 300px.

**Step 6: Commit**

```bash
git add src/index.html src/styles.css src/main.js
git commit -m "feat: add Simple/Detailed mode toggle with panel layout"
```

---

### Task 3: Rust Backend — AI Usage Summary Command

**Files:**
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/Cargo.toml` (add `dirs` crate for home directory)

**Step 1: Add `dirs` dependency to Cargo.toml**

Add to `[dependencies]`:

```toml
dirs = "6"
```

**Step 2: Add AI usage structs to main.rs**

Add after the existing `GpuInfo` struct:

```rust
#[derive(Serialize, Clone)]
struct AiUsageSummary {
    total_tokens_today: u64,
    models: Vec<AiModelUsage>,
    session_count: u32,
    message_count: u32,
    available: bool,
}

#[derive(Serialize, Clone)]
struct AiModelUsage {
    model: String,
    input_tokens: u64,
    output_tokens: u64,
}
```

**Step 3: Implement get_ai_usage_summary command**

Add the command function:

```rust
#[tauri::command]
fn get_ai_usage_summary() -> AiUsageSummary {
    let empty = AiUsageSummary {
        total_tokens_today: 0,
        models: vec![],
        session_count: 0,
        message_count: 0,
        available: false,
    };

    let Some(home) = dirs::home_dir() else { return empty };
    let stats_path = home.join(".claude").join("stats-cache.json");
    let Ok(content) = std::fs::read_to_string(&stats_path) else { return empty };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else { return empty };

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    // Find today's activity
    let mut session_count = 0u32;
    let mut message_count = 0u32;
    if let Some(daily) = json["dailyActivity"].as_array() {
        for day in daily {
            if day["date"].as_str() == Some(&today) {
                session_count = day["sessionCount"].as_u64().unwrap_or(0) as u32;
                message_count = day["messageCount"].as_u64().unwrap_or(0) as u32;
                break;
            }
        }
    }

    // Find today's token usage by model
    let mut models = Vec::new();
    let mut total_tokens_today = 0u64;
    if let Some(daily_tokens) = json["dailyModelTokens"].as_array() {
        for day in daily_tokens {
            if day["date"].as_str() == Some(&today) {
                if let Some(by_model) = day["tokensByModel"].as_object() {
                    for (model_name, tokens) in by_model {
                        let t = tokens.as_u64().unwrap_or(0);
                        total_tokens_today += t;
                        // Shorten model name: "claude-opus-4-6" → "opus"
                        let short = shorten_model_name(model_name);
                        models.push(AiModelUsage {
                            model: short,
                            input_tokens: t,
                            output_tokens: 0,
                        });
                    }
                }
                break;
            }
        }
    }

    // Sort by tokens descending
    models.sort_by(|a, b| b.input_tokens.cmp(&a.input_tokens));

    AiUsageSummary {
        total_tokens_today,
        models,
        session_count,
        message_count,
        available: true,
    }
}

fn shorten_model_name(name: &str) -> String {
    if name.contains("opus") { return "opus".into(); }
    if name.contains("sonnet") { return "sonnet".into(); }
    if name.contains("haiku") { return "haiku".into(); }
    // Fallback: take last meaningful segment
    name.split('-').find(|s| !s.chars().all(|c| c.is_ascii_digit())).unwrap_or(name).to_string()
}
```

**Step 4: Add `chrono` to Cargo.toml**

```toml
chrono = "0.4"
```

**Step 5: Register the command in main()**

Add `get_ai_usage_summary` to `invoke_handler`:

```rust
.invoke_handler(tauri::generate_handler![
    get_cpu_info,
    get_memory_info,
    get_top_processes,
    get_gpu_info,
    get_ai_usage_summary
])
```

**Step 6: Build and verify**

Run: `cargo tauri dev`
Open browser console, run: `window.__TAURI__.core.invoke('get_ai_usage_summary')`
Expected: Returns JSON with today's token data.

**Step 7: Commit**

```bash
git add src-tauri/src/main.rs src-tauri/Cargo.toml
git commit -m "feat: add get_ai_usage_summary Tauri command"
```

---

### Task 4: Rust Backend — AI Usage History Command

**Files:**
- Modify: `src-tauri/src/main.rs`

**Step 1: Add history structs**

```rust
#[derive(Serialize, Clone)]
struct AiUsageHistory {
    daily_tokens: Vec<DailyTokens>,
    recent_sessions: Vec<AiSessionInfo>,
    available: bool,
}

#[derive(Serialize, Clone)]
struct DailyTokens {
    date: String,
    total_tokens: u64,
}

#[derive(Serialize, Clone)]
struct AiSessionInfo {
    project: String,
    total_tokens: u64,
    message_count: u32,
    timestamp: String,
}
```

**Step 2: Implement get_ai_usage_history command**

This reads from `stats-cache.json` for daily trend (already aggregated) and scans JSONL file metadata for recent sessions:

```rust
#[tauri::command]
fn get_ai_usage_history() -> AiUsageHistory {
    let empty = AiUsageHistory {
        daily_tokens: vec![],
        recent_sessions: vec![],
        available: false,
    };

    let Some(home) = dirs::home_dir() else { return empty };
    let claude_dir = home.join(".claude");

    // 1. Daily tokens from stats-cache.json
    let mut daily_tokens = Vec::new();
    let stats_path = claude_dir.join("stats-cache.json");
    if let Ok(content) = std::fs::read_to_string(&stats_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(daily_model_tokens) = json["dailyModelTokens"].as_array() {
                for day in daily_model_tokens {
                    let date = day["date"].as_str().unwrap_or("").to_string();
                    let mut total = 0u64;
                    if let Some(by_model) = day["tokensByModel"].as_object() {
                        for (_model, tokens) in by_model {
                            total += tokens.as_u64().unwrap_or(0);
                        }
                    }
                    daily_tokens.push(DailyTokens { date, total_tokens: total });
                }
            }
        }
    }
    // Keep last 7 days
    if daily_tokens.len() > 7 {
        daily_tokens = daily_tokens.split_off(daily_tokens.len() - 7);
    }

    // 2. Recent sessions from JSONL files
    let mut recent_sessions = Vec::new();
    let projects_dir = claude_dir.join("projects");
    if let Ok(project_entries) = std::fs::read_dir(&projects_dir) {
        let mut session_files: Vec<(String, std::path::PathBuf, std::time::SystemTime)> = Vec::new();
        for project_entry in project_entries.flatten() {
            let project_name = project_entry.file_name().to_string_lossy().to_string();
            if let Ok(files) = std::fs::read_dir(project_entry.path()) {
                for file in files.flatten() {
                    let path = file.path();
                    if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
                        if let Ok(meta) = file.metadata() {
                            if let Ok(modified) = meta.modified() {
                                session_files.push((project_name.clone(), path, modified));
                            }
                        }
                    }
                }
            }
        }
        // Sort by modified time, most recent first
        session_files.sort_by(|a, b| b.2.cmp(&a.2));
        session_files.truncate(5);

        for (project, path, modified) in session_files {
            let mut total_tokens = 0u64;
            let mut message_count = 0u32;
            if let Ok(content) = std::fs::read_to_string(&path) {
                for line in content.lines() {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                        if val["type"].as_str() == Some("assistant") {
                            message_count += 1;
                            if let Some(usage) = val["message"]["usage"].as_object() {
                                total_tokens += usage.get("input_tokens")
                                    .and_then(|v| v.as_u64()).unwrap_or(0);
                                total_tokens += usage.get("output_tokens")
                                    .and_then(|v| v.as_u64()).unwrap_or(0);
                            }
                        }
                    }
                }
            }
            // Decode project name (e.g., "c--Dev" → "c:/Dev")
            let display_project = project.replace("--", ":/");
            let timestamp = modified
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs().to_string())
                .unwrap_or_default();

            recent_sessions.push(AiSessionInfo {
                project: display_project,
                total_tokens,
                message_count,
                timestamp,
            });
        }
    }

    AiUsageHistory {
        daily_tokens,
        recent_sessions,
        available: true,
    }
}
```

**Step 3: Register the command**

```rust
.invoke_handler(tauri::generate_handler![
    get_cpu_info,
    get_memory_info,
    get_top_processes,
    get_gpu_info,
    get_ai_usage_summary,
    get_ai_usage_history
])
```

**Step 4: Build and verify**

Run: `cargo tauri dev`
Console: `window.__TAURI__.core.invoke('get_ai_usage_history')`
Expected: Returns JSON with daily tokens array and recent sessions.

**Step 5: Commit**

```bash
git add src-tauri/src/main.rs
git commit -m "feat: add get_ai_usage_history Tauri command"
```

---

### Task 5: Frontend — AI Panel Rendering + Polling

**Files:**
- Modify: `src/main.js`
- Modify: `src/index.html`

**Step 1: Add AI panel HTML structure in index.html**

Replace the placeholder `<div id="ai-content">` with:

```html
<div id="ai-content">
  <div class="ai-today">
    <div class="ai-today-label">Today</div>
    <div class="ai-today-total" id="ai-total">--</div>
    <div id="ai-models"></div>
  </div>
  <div class="ai-stats" id="ai-stats">Sessions: -- Msgs: --</div>
  <div class="ai-trend-title">7 Days</div>
  <div class="sparkline" id="ai-sparkline"></div>
  <div class="sparkline-labels" id="ai-sparkline-labels"></div>
  <div class="ai-sessions-title">Recent</div>
  <div id="ai-sessions"></div>
</div>
```

**Step 2: Add AI update functions in main.js**

```javascript
function formatTokens(n) {
  if (n >= 1000000) return (n / 1000000).toFixed(1) + 'M';
  if (n >= 1000) return (n / 1000).toFixed(1) + 'K';
  return n.toString();
}

async function updateAiSummary() {
  try {
    const data = await invoke('get_ai_usage_summary');
    if (!data.available) return;

    document.getElementById('ai-total').textContent = formatTokens(data.total_tokens_today);
    document.getElementById('ai-models').innerHTML = data.models
      .map(m => `<div class="ai-model-row"><span>${m.model}</span><span class="ai-model-tokens">${formatTokens(m.input_tokens)}</span></div>`)
      .join('');
    document.getElementById('ai-stats').textContent =
      `Sessions: ${data.session_count}  Msgs: ${data.message_count}`;
  } catch (e) {
    console.error('AI summary update failed:', e);
  }
}

async function updateAiHistory() {
  try {
    const data = await invoke('get_ai_usage_history');
    if (!data.available) return;

    // Sparkline
    const maxTokens = Math.max(...data.daily_tokens.map(d => d.total_tokens), 1);
    const sparkline = document.getElementById('ai-sparkline');
    sparkline.innerHTML = data.daily_tokens
      .map(d => {
        const h = Math.max(2, (d.total_tokens / maxTokens) * 32);
        return `<div class="sparkline-bar" style="height:${h}px"></div>`;
      })
      .join('');

    // Day labels
    const labels = document.getElementById('ai-sparkline-labels');
    const dayNames = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
    labels.innerHTML = data.daily_tokens
      .map(d => {
        const day = new Date(d.date + 'T00:00:00').getDay();
        return `<span>${dayNames[day]}</span>`;
      })
      .join('');

    // Recent sessions
    document.getElementById('ai-sessions').innerHTML = data.recent_sessions
      .map(s => {
        const name = s.project.split('/').pop() || s.project;
        return `<div class="ai-session-row"><span>${name}</span><span class="ai-session-tokens">${formatTokens(s.total_tokens)}</span></div>`;
      })
      .join('');
  } catch (e) {
    console.error('AI history update failed:', e);
  }
}

function startAiPolling() {
  updateAiSummary();
  updateAiHistory();
  aiSummaryInterval = setInterval(updateAiSummary, 30000);
  aiHistoryInterval = setInterval(updateAiHistory, 300000);
}

function stopAiPolling() {
  if (aiSummaryInterval) { clearInterval(aiSummaryInterval); aiSummaryInterval = null; }
  if (aiHistoryInterval) { clearInterval(aiHistoryInterval); aiHistoryInterval = null; }
}
```

**Step 3: Build and verify end-to-end**

Run: `cargo tauri dev`
1. Click S → D: Widget expands, AI panel shows today's token count, model breakdown, sparkline, recent sessions.
2. Click D → S: Widget shrinks, AI panel hidden, polling stops.

**Step 4: Commit**

```bash
git add src/index.html src/main.js
git commit -m "feat: wire up AI usage panel with data polling"
```

---

### Task 6: Polish & Edge Cases

**Files:**
- Modify: `src/main.js`
- Modify: `src/styles.css`

**Step 1: Handle edge snapping for wider widget**

In `snapToEdge()`, the width calculation already uses `size.width / scale` which is dynamic. No change needed — verify this works at 600px.

**Step 2: Handle no Claude Code installed**

The `get_ai_usage_summary` and `get_ai_usage_history` commands already return `available: false` if `~/.claude/` doesn't exist. In the frontend, show a message:

Add to `updateAiSummary()` after the `if (!data.available)` check:

```javascript
if (!data.available) {
  document.getElementById('ai-content').innerHTML =
    '<div class="ai-loading">Claude Code not detected</div>';
  return;
}
```

**Step 3: Persist mode preference**

Use `localStorage` to remember the user's mode choice:

```javascript
// On startup, restore mode
const savedMode = localStorage.getItem('clance-mode');
if (savedMode === 'detailed') {
  modeToggle.click();
}

// In toggle handler, save preference
localStorage.setItem('clance-mode', detailedMode ? 'detailed' : 'simple');
```

**Step 4: Build and full test**

Run: `cargo tauri dev`
Test: Toggle modes, check persistence across restart, verify edge snap at both widths, check with no Claude data.

**Step 5: Commit**

```bash
git add src/main.js src/styles.css
git commit -m "feat: handle edge cases and persist mode preference"
```

---

### Task 7: Version Bump & Final Build

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/Cargo.toml`

**Step 1: Bump version**

```bash
./scripts/bump-version.sh 0.3.0
```

**Step 2: Full release build**

```bash
cargo tauri build
```

Expected: Builds successfully for current platform.

**Step 3: Commit and tag**

```bash
git add -A
git commit -m "v0.3.0: detailed mode with AI usage tracking, Nord palette"
git tag v0.3.0
```
