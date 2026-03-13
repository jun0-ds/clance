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

#[tauri::command]
fn get_top_processes(state: State<AppState>) -> Vec<ProcessInfo> {
    let mut sys = state.sys.lock().unwrap();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let mut procs: Vec<_> = sys
        .processes()
        .values()
        .map(|p| ProcessInfo {
            name: p.name().to_string_lossy().to_string(),
            cpu_usage: p.cpu_usage(),
            memory_mb: p.memory() as f64 / 1_048_576.0,
            pid: p.pid().as_u32(),
        })
        .collect();
    procs.sort_by(|a, b| {
        b.cpu_usage
            .partial_cmp(&a.cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    procs.truncate(5);
    procs
}

fn main() {
    let sys = System::new_all();
    tauri::Builder::default()
        .manage(AppState {
            sys: Mutex::new(sys),
        })
        .invoke_handler(tauri::generate_handler![get_cpu_info, get_memory_info, get_top_processes])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
