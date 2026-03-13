#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use std::sync::Mutex;
use sysinfo::System;
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

fn main() {
    let sys = System::new_all();
    tauri::Builder::default()
        .manage(AppState {
            sys: Mutex::new(sys),
        })
        .invoke_handler(tauri::generate_handler![
            get_cpu_info,
            get_memory_info,
            get_top_processes,
            get_gpu_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
