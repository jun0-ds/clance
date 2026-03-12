# Clance — System Monitor Widget Design

## Overview

**Clance** (Claude + Glance) is a lightweight desktop system monitor widget for **personal PC use**.
Built with Tauri (Rust + Web frontend), it displays CPU, Memory, GPU usage and top processes at a glance.

> This is a personal desktop utility, not intended for server monitoring or enterprise use.

## Target

- **Platforms**: Windows, macOS
- **Distribution**: GitHub Releases

## Architecture

```
┌─────────────────────────────┐
│  Frontend (HTML/CSS/JS)     │  ← Widget UI, glassmorphism
│  Polls backend every 2s     │
├─────────────────────────────┤
│  Tauri IPC (invoke)         │
├─────────────────────────────┤
│  Rust Backend               │  ← System info collection
│  - CPU usage                │
│  - Memory usage/capacity    │
│  - GPU usage/VRAM (NVML)    │
│  - Top 5 processes by CPU   │
└─────────────────────────────┘
```

## UI Design

- **Single card** (~300x500px), always-on-top, frameless, draggable
- **Glassmorphism**: translucent dark background + blur + rounded corners
- Each section (CPU, Memory, GPU, Processes) is **collapsible**

```
╭─────────────────────────╮
│  ▼ CPU          48%     │
│  ████████░░░░░░░░░░     │
│                         │
│  ▼ Memory    12/32GB    │
│  ██████░░░░░░░░░░░░     │
│                         │
│  ▼ GPU          35%     │
│  ██████░░░░░░░░░░░░     │
│  VRAM: 4.2/8GB          │
│                         │
│  ▼ Top Processes        │
│  1. chrome     12.3%    │
│  2. vscode      8.1%    │
│  3. rust-ana    5.2%    │
│  4. discord     3.8%    │
│  5. explorer    2.1%    │
╰─────────────────────────╯
```

## Tech Details

| Component | Implementation |
|-----------|---------------|
| CPU/Memory/Processes | Rust `sysinfo` crate (cross-platform) |
| GPU (Windows) | NVML (NVIDIA) / WMI (fallback) |
| GPU (macOS) | IOKit / Metal API |
| Translucency/Blur | Tauri window effects / OS native API |
| Frontend | Vanilla HTML/CSS/JS (no framework) |
| Update interval | 2 seconds |
| Build/Deploy | GitHub Actions → GitHub Releases |

## Out of Scope (v1)

- Network / Disk monitoring
- Settings UI
- System tray icon
- Graphs / Charts
- Multi-monitor support
