# Clance v0.3.0 — Detailed Mode + AI Usage Tracking

## Overview

Add a **Detailed mode** that expands the widget horizontally to show Claude Code token usage alongside existing system metrics. Redesign color palette from generic AI-gradient to Nord theme.

## Mode Switching

- Title bar gets `S` / `D` toggle button (next to opacity slider)
- **Simple mode**: Current widget (300px wide), no changes
- **Detailed mode**: Widget expands to 600px, right panel shows AI Usage
- Transition: CSS animation on width change, `window.setSize()` synced

```
Simple (300px):          Detailed (600px):
╭──────────────╮         ╭──────────────┬───────────────╮
│ Clance [S|D] │         │ Clance [S|D] │  AI Usage     │
│  CPU    48%  │         │  CPU    48%  │  Today 38.2K  │
│  MEM  12/32G │         │  MEM  12/32G │  opus   22.5K │
│  GPU    35%  │         │  GPU    35%  │  sonnet 15.7K │
│  Processes   │         │  Processes   │  7d ▁▃▅▂▇▄█  │
╰──────────────╯         │              │  Sessions     │
                         │              │  #1 clance 18K│
                         ╰──────────────┴───────────────╯
```

## AI Usage Panel (Right Side)

### Data Sources

| Data | Source | Poll Interval |
|------|--------|---------------|
| Today's tokens, model breakdown | `~/.claude/stats-cache.json` | 30s |
| Session count, message count | `~/.claude/stats-cache.json` | 30s |
| Recent session list | `~/.claude/projects/*/*.jsonl` | 1min |
| 7-day trend | JSONL files, date-aggregated | 5min |

### Tauri Commands (Rust)

**`get_ai_usage_summary`** — Parse `stats-cache.json`
```rust
struct AiUsageSummary {
    total_tokens_today: u64,      // input + output
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    models: Vec<ModelUsage>,      // per-model breakdown
    session_count: u32,
    message_count: u32,
}

struct ModelUsage {
    model: String,                // e.g. "opus", "sonnet"
    input_tokens: u64,
    output_tokens: u64,
}
```

**`get_ai_usage_history`** — Scan JSONL files
```rust
struct AiUsageHistory {
    daily_tokens: Vec<DailyTokens>,  // last 7 days
    recent_sessions: Vec<SessionInfo>, // last 5 sessions
}

struct DailyTokens {
    date: String,          // "2026-03-14"
    total_tokens: u64,
}

struct SessionInfo {
    project: String,       // directory name as project identifier
    total_tokens: u64,
    message_count: u32,
    timestamp: String,     // ISO 8601
}
```

### UI Layout (Right Panel)

```
┌─ AI Usage ──────────────┐
│                          │
│  Today         38,200    │
│  ┌──────────────────┐   │
│  │ opus       22,516 │   │
│  │ sonnet     15,684 │   │
│  └──────────────────┘   │
│  Sessions: 3  Msgs: 47  │
│                          │
│  7 Days                  │
│  ▁ ▃ ▅ ▂ ▇ ▄ █         │
│  M  T  W  T  F  S  S    │
│                          │
│  Recent                  │
│  clance-dev      18.2K   │
│  code-review     12.1K   │
│  debug-fix        7.9K   │
└──────────────────────────┘
```

- Sparkline: 7 `<span>` elements with dynamic height, no external library
- Token counts: abbreviated (K for thousands)
- Model names: shortened (claude-opus-4-6 → opus)

## Color Palette — Nord Theme

Replace all current colors with Nord palette:

| Role | Current | New (Nord) |
|------|---------|------------|
| Background | `rgba(20, 20, 30, 0.75)` | `rgba(46, 52, 64, 0.85)` `#2e3440` |
| Progress bar (normal) | `linear-gradient(#4facfe, #00f2fe)` | `#88c0d0` (solid) |
| Progress bar (warn 60%+) | `linear-gradient(#f093fb, #f5576c)` | `#ebcb8b` (amber) |
| Progress bar (crit 80%+) | `linear-gradient(#f5576c, #ff6b6b)` | `#bf616a` (muted red) |
| Text primary | `#e0e0e0` | `#eceff4` |
| Text secondary | `#888` | `#d8dee9` |
| Text muted | — | `#4c566a` |
| Accent | `#4facfe` | `#88c0d0` (frost blue) |
| Section header hover | — | `#3b4252` |
| AI panel accent | — | `#a3be8c` (green, differentiates from system) |

Key change: **All gradient bars → solid colors.** This is the single biggest change to remove the AI-generated look.

## Polling Architecture

```
System metrics (CPU/MEM/GPU/Proc):  every 2s   (unchanged)
AI summary (stats-cache.json):      every 30s  (lightweight file read)
AI history (JSONL scan):            every 5min (heavier, cached)
```

- AI polling only runs in Detailed mode
- Mode switch to Simple stops AI polling to save resources

## Files Changed

| File | Change |
|------|--------|
| `main.rs` | Add `get_ai_usage_summary`, `get_ai_usage_history` commands |
| `index.html` | S/D toggle button, right AI panel markup |
| `main.js` | Mode toggle logic, AI polling, sparkline render, resize to 600px |
| `styles.css` | Nord palette, right panel layout, flex container, transition animation |
| `capabilities/default.json` | Add fs read permission for `~/.claude/` |

## Out of Scope (v0.3.0)

- Other AI tools (Cursor, Copilot, Codex)
- Cost calculation ($)
- Settings UI (custom paths)
- API key-based tracking
- Export/share usage data
