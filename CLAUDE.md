# CLAUDE.md

> 이 파일은 글로벌 설정(`~/.claude/CLAUDE.md`)의 지침을 기본으로 따릅니다.
> 프로젝트별 지침은 아래 섹션에 추가하세요.

## Project-Specific Guidelines

- **Project**: Clance — lightweight system monitor widget for personal PC use
- **Stack**: Tauri v2 (Rust backend + Vanilla HTML/CSS/JS frontend)
- **Platforms**: Windows, macOS
- **Design doc**: `docs/plans/2026-03-13-clance-design.md`

## Build & Run

```bash
# Prerequisites: Rust, Node.js, MSVC Build Tools (Windows)
# On Windows, MSVC environment must be set (vcvars64.bat or manual PATH/LIB/INCLUDE)

cargo tauri dev    # development (hot reload)
cargo tauri build  # production build → src-tauri/target/release/clance.exe
```

## Release Guide

Releases are automated via GitHub Actions. To publish a new version:

1. **Update version** in `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml`
2. **Commit** the version bump
3. **Tag and push**:
   ```bash
   git tag v0.x.0
   git push origin main --tags
   ```
4. GitHub Actions will automatically:
   - Build for Windows (x86_64) and macOS (aarch64)
   - Create a GitHub Release with the installers attached

## Project Structure

```
src/                  # Frontend (HTML/CSS/JS)
  index.html          # Widget layout
  styles.css          # Glassmorphism styling
  main.js             # Data binding, polling, resize logic
src-tauri/            # Rust backend
  src/main.rs         # Tauri commands: CPU, Memory, GPU, Processes
  tauri.conf.json     # App config (window, permissions)
  capabilities/       # Tauri v2 permissions
  Cargo.toml          # Rust dependencies
.github/workflows/    # CI/CD
  release.yml         # Auto-build on tag push
```

## Key Dependencies

- `sysinfo` — CPU, Memory, Process info (cross-platform)
- `nvml-wrapper` — NVIDIA GPU info (Windows only, `cfg(windows)`)
- Tauri v2 — Desktop app framework with WebView

## Conventions

- No JS frameworks — vanilla HTML/CSS/JS only
- Frontend polls backend every 2 seconds via Tauri IPC (`invoke`)
- Window auto-resizes to fit content height
- GPU features gracefully degrade when NVIDIA GPU is not available
