#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::sync::Mutex;
use sysinfo::System;
use tauri::{
    State,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
    WebviewUrl, WebviewWindowBuilder,
};

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

#[derive(Serialize, Clone)]
struct ProcessInfo {
    name: String,
    cpu_usage: f32,
    memory_mb: f64,
    gpu_memory_mb: f64,
    pid: u32,
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
        usage_percent: if total > 0.0 { (used / total) * 100.0 } else { 0.0 },
    }
}

#[cfg(windows)]
fn extract_gpu_mem_mb(mem: nvml_wrapper::enums::device::UsedGpuMemory) -> f64 {
    match mem {
        nvml_wrapper::enums::device::UsedGpuMemory::Used(bytes) => bytes as f64 / 1_048_576.0,
        nvml_wrapper::enums::device::UsedGpuMemory::Unavailable => 0.0,
    }
}

#[cfg(windows)]
fn get_gpu_process_memory() -> std::collections::HashMap<u32, f64> {
    use nvml_wrapper::Nvml;
    let mut map = std::collections::HashMap::new();
    let Ok(nvml) = Nvml::init() else { return map };
    let Ok(device) = nvml.device_by_index(0) else { return map };
    if let Ok(procs) = device.running_graphics_processes() {
        for p in procs {
            let mem_mb = extract_gpu_mem_mb(p.used_gpu_memory);
            map.insert(p.pid, mem_mb);
        }
    }
    if let Ok(procs) = device.running_compute_processes() {
        for p in procs {
            let mem_mb = extract_gpu_mem_mb(p.used_gpu_memory);
            map.entry(p.pid).and_modify(|v| *v += mem_mb).or_insert(mem_mb);
        }
    }
    map
}

fn friendly_name(raw: &str) -> String {
    let name = raw.strip_suffix(".exe").unwrap_or(raw);
    match name.to_lowercase().as_str() {
        "code" => "VS Code".into(),
        "msedgewebview2" | "msedge" => "Edge".into(),
        "chrome" => "Chrome".into(),
        "firefox" => "Firefox".into(),
        "discord" => "Discord".into(),
        "slack" => "Slack".into(),
        "spotify" => "Spotify".into(),
        "explorer" => "Explorer".into(),
        "dwm" => "Desktop Window Mgr".into(),
        "searchhost" => "Windows Search".into(),
        "runtimebroker" => "Runtime Broker".into(),
        "memory compression" => "Mem Compression".into(),
        "windowsterminal" => "Terminal".into(),
        "powershell" => "PowerShell".into(),
        "cmd" => "CMD".into(),
        "claude" => "Claude".into(),
        "teams" => "Teams".into(),
        "notion" => "Notion".into(),
        "obs64" | "obs" => "OBS".into(),
        "steam" | "steamwebhelper" => "Steam".into(),
        _ => name.to_string(),
    }
}

#[tauri::command]
fn get_top_processes(state: State<AppState>, sort_by: String) -> Vec<ProcessInfo> {
    let mut sys = state.sys.lock().unwrap();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    #[cfg(windows)]
    let gpu_mem = get_gpu_process_memory();
    #[cfg(not(windows))]
    let gpu_mem: std::collections::HashMap<u32, f64> = std::collections::HashMap::new();

    // Aggregate by friendly name
    let mut aggregated: std::collections::HashMap<String, (f32, f64, f64)> =
        std::collections::HashMap::new();
    for p in sys.processes().values() {
        let pid = p.pid().as_u32();
        let name = friendly_name(&p.name().to_string_lossy());
        let entry = aggregated.entry(name).or_insert((0.0, 0.0, 0.0));
        entry.0 += p.cpu_usage();
        entry.1 += p.memory() as f64 / 1_048_576.0;
        entry.2 += gpu_mem.get(&pid).copied().unwrap_or(0.0);
    }

    let mut procs: Vec<_> = aggregated
        .into_iter()
        .map(|(name, (cpu, mem, gpu))| ProcessInfo {
            name,
            cpu_usage: cpu,
            memory_mb: mem,
            gpu_memory_mb: gpu,
            pid: 0,
        })
        .collect();

    match sort_by.as_str() {
        "memory" => procs.sort_by(|a, b| {
            b.memory_mb.partial_cmp(&a.memory_mb).unwrap_or(std::cmp::Ordering::Equal)
        }),
        "gpu" => procs.sort_by(|a, b| {
            b.gpu_memory_mb.partial_cmp(&a.gpu_memory_mb).unwrap_or(std::cmp::Ordering::Equal)
        }),
        _ => procs.sort_by(|a, b| {
            b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal)
        }),
    }

    procs.truncate(5);
    procs
}

#[derive(Serialize, Clone)]
struct GpuInfo {
    name: String,
    usage_percent: u32,
    memory_total_mb: u64,
    memory_used_mb: u64,
    temperature: Option<u32>,
    available: bool,
}

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
    tokens: u64,
}

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
    let temp = device
        .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
        .ok();
    Some(GpuInfo {
        name: device.name().ok().unwrap_or_default(),
        usage_percent: utilization.gpu,
        memory_total_mb: memory.total / 1_048_576,
        memory_used_mb: memory.used / 1_048_576,
        temperature: temp,
        available: true,
    })
}

fn shorten_model_name(name: &str) -> String {
    if name.contains("opus") { return "opus".into(); }
    if name.contains("sonnet") { return "sonnet".into(); }
    if name.contains("haiku") { return "haiku".into(); }
    name.split('-').find(|s| !s.chars().all(|c| c.is_ascii_digit())).unwrap_or(name).to_string()
}

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
                        let short = shorten_model_name(model_name);
                        models.push(AiModelUsage { model: short, tokens: t });
                    }
                }
                break;
            }
        }
    }

    models.sort_by(|a, b| b.tokens.cmp(&a.tokens));

    AiUsageSummary {
        total_tokens_today,
        models,
        session_count,
        message_count,
        available: true,
    }
}

#[tauri::command]
fn get_ai_usage_history() -> AiUsageHistory {
    let empty = AiUsageHistory {
        daily_tokens: vec![],
        recent_sessions: vec![],
        available: false,
    };

    let Some(home) = dirs::home_dir() else { return empty };
    let claude_dir = home.join(".claude");

    // 1. Daily tokens from stats-cache.json (already aggregated)
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
    // Keep last 7 days only
    if daily_tokens.len() > 7 {
        daily_tokens = daily_tokens.split_off(daily_tokens.len() - 7);
    }

    // 2. Recent sessions from JSONL files (scan by file modification time)
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
        // Sort by most recently modified
        session_files.sort_by(|a, b| b.2.cmp(&a.2));
        session_files.truncate(5);

        for (project, path, _modified) in session_files {
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
            // Decode project name: "c--Dev" -> "c:/Dev", take last segment
            let display = project.replace("--", ":/");
            let short = display.split('/').last().unwrap_or(&display).to_string();
            recent_sessions.push(AiSessionInfo {
                project: short,
                total_tokens,
                message_count,
            });
        }
    }

    AiUsageHistory {
        daily_tokens,
        recent_sessions,
        available: true,
    }
}

fn main() {
    let sys = System::new_all();
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // When second instance launched, show existing window
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.show();
                let _ = win.set_focus();
            }
        }))
        .manage(AppState {
            sys: Mutex::new(sys),
        })
        .invoke_handler(tauri::generate_handler![
            get_cpu_info,
            get_memory_info,
            get_top_processes,
            get_gpu_info,
            get_ai_usage_summary,
            get_ai_usage_history
        ])
        .setup(|app| {
            let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let about = MenuItem::with_id(app, "about", "About Clance", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &about, &quit])?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("Clance")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "about" => {
                        if let Some(win) = app.get_webview_window("about") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        } else {
                            let _ = WebviewWindowBuilder::new(
                                app,
                                "about",
                                WebviewUrl::App("about.html".into()),
                            )
                            .title("About Clance")
                            .inner_size(280.0, 200.0)
                            .resizable(false)
                            .center()
                            .build();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        if let Some(win) = tray.app_handle().get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
