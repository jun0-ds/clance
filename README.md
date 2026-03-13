# Clance

> Claude + Glance — A lightweight system monitor widget for your personal PC.

![Windows](https://img.shields.io/badge/Windows-0078D6?logo=windows&logoColor=white)
![macOS](https://img.shields.io/badge/macOS-000000?logo=apple&logoColor=white)

![Clance Screenshot](docs/screenshot.png)

Clance is a desktop widget that shows your system's vital signs at a glance:

- **CPU** usage with per-core details
- **Memory** usage
- **GPU** utilization & VRAM (NVIDIA on Windows)
- **Top 5 processes** by CPU usage

Built with [Tauri](https://tauri.app) for minimal resource footprint (~15MB RAM).

> **Note:** This is a personal desktop utility, not intended for server monitoring or enterprise use.

## Comparison with Other Widgets

| Feature | **Clance** | ResourceMonitor | Win11 Widget | Xbox Game Bar | Rainmeter |
|---------|:---:|:---:|:---:|:---:|:---:|
| CPU | ✅ | ✅ | ✅ | ✅ | ✅ |
| Memory | ✅ | ✅ | ✅ | ✅ | ✅ |
| GPU Utilization | ✅ | ✅ | ✅ | ✅ | Skin-dependent |
| GPU VRAM | ✅ | ❌ | ❌ | ❌ | Skin-dependent |
| Top Processes | ✅ | ❌ | ❌ | ❌ | Skin-dependent |
| Process Sort (CPU/MEM/GPU) | ✅ | ❌ | ❌ | ❌ | ❌ |
| Friendly Process Names | ✅ | ❌ | ❌ | ❌ | ❌ |
| Cross-platform | ✅ | ❌ | ❌ | ❌ | ❌ |
| Lightweight Setup | ✅ | ✅ | ✅ | ✅ | ❌ |

## Install

### Windows

Download the latest `.exe` or `.msi` from [GitHub Releases](https://github.com/jun0-ds/clance/releases).

### macOS (Homebrew)

```bash
brew tap jun0-ds/clance
brew install --cask clance
```

## Build from Source

```bash
# Prerequisites: Rust, Node.js, MSVC Build Tools (Windows)
cargo install tauri-cli --version "^2"
cargo tauri dev    # development
cargo tauri build  # production build
```

## License

MIT
