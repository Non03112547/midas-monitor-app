// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod models;
mod monitor;
mod gpu;
mod ping;
mod power;
mod fps;
mod history;

use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tokio::time::{sleep, Duration};
use tauri::Emitter;
use models::AppConfig;

struct HistoryState(Arc<Mutex<history::SessionTracker>>);
struct DailyHistoryState(Arc<Mutex<history::DailyTracker>>);
struct HistoryWindowLock(tokio::sync::Mutex<()>);

#[tauri::command]
fn get_session_history(state: tauri::State<'_, HistoryState>) -> Vec<history::SessionRecord> {
    match state.0.lock() {
        Ok(tracker) => tracker.get_all_sessions(),
        Err(_) => Vec::new(),
    }
}

#[tauri::command]
fn clear_session_history(state: tauri::State<'_, HistoryState>) -> Result<(), String> {
    let mut tracker = state
        .0
        .lock()
        .map_err(|_| "Failed to lock session history".to_string())?;
    tracker.clear_past_sessions()
}

#[tauri::command]
fn get_daily_history(state: tauri::State<'_, DailyHistoryState>) -> history::DailyHistoryData {
    match state.0.lock() {
        Ok(tracker) => tracker.get_daily_history(),
        Err(_) => history::DailyHistoryData::default(),
    }
}

#[tauri::command]
fn clear_daily_history(state: tauri::State<'_, DailyHistoryState>) -> Result<(), String> {
    let mut tracker = state
        .0
        .lock()
        .map_err(|_| "Failed to lock daily history".to_string())?;
    tracker.clear()
}

#[tauri::command]
async fn open_history_window(
    app: tauri::AppHandle,
    lock: tauri::State<'_, HistoryWindowLock>,
) -> Result<(), String> {
    let _guard = lock.0.lock().await;

    if let Some(win) = app.get_webview_window("history") {
        let _ = win.unminimize();
        let _ = win.show();
        let _ = win.set_focus();
        let _ = win.emit("history-refresh", ());
        return Ok(());
    }

    let win = tauri::WebviewWindowBuilder::new(
        &app,
        "history",
        tauri::WebviewUrl::App("history.html".into()),
    )
    .title("ประวัติการใช้งาน - MD Monitor")
    .inner_size(760.0, 620.0)
    .min_inner_size(560.0, 440.0)
    .resizable(true)
    .always_on_top(true)
    .skip_taskbar(false)
    .center()
    .build()
    .map_err(|e| format!("Failed to open history window: {}", e))?;

    let _ = win.show();
    let _ = win.set_focus();
    Ok(())
}

#[tauri::command]
fn get_config() -> AppConfig {
    config::load_config()
}

#[tauri::command]
fn save_config(app: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    config::save_config(&config)?;
    // autostart registry is updated via set_autostart command
    let _ = app.emit("config-changed", &config);
    Ok(())
}

#[tauri::command]
fn reset_config(app: tauri::AppHandle) -> Result<AppConfig, String> {
    let def = AppConfig::default();
    config::save_config(&def)?;
    let _ = config::set_autostart_registry(def.autostart);
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: def.pos_x,
            y: def.pos_y,
        }));
    }
    let _ = app.emit("config-changed", &def);
    Ok(def)
}

#[tauri::command]
fn set_ignore_cursor(window: tauri::WebviewWindow, ignore: bool) -> Result<(), String> {
    window.set_ignore_cursor_events(ignore).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_window_position(window: tauri::WebviewWindow, x: i32, y: i32) -> Result<(), String> {
    window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn set_main_position(app: tauri::AppHandle, x: i32, y: i32) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
    }
    let mut cfg = config::load_config();
    cfg.pos_x = x;
    cfg.pos_y = y;
    // position is persisted via save_config on mouseup/preset click
    let _ = app.emit("config-changed", &cfg);
    Ok(())
}

#[tauri::command]
fn get_screen_resolution(window: tauri::WebviewWindow) -> Result<(u32, u32), String> {
    if let Some(monitor) = window.primary_monitor().map_err(|e| e.to_string())? {
        let size = monitor.size();
        Ok((size.width, size.height))
    } else {
        Ok((1920, 1080))
    }
}

#[tauri::command]
fn get_main_rect(app: tauri::AppHandle) -> Result<(i32, i32, u32, u32), String> {
    if let Some(win) = app.get_webview_window("main") {
        let pos = win.outer_position().unwrap_or(tauri::PhysicalPosition { x: 0, y: 0 });
        let size = win.outer_size().unwrap_or(tauri::PhysicalSize { width: 300, height: 400 });
        Ok((pos.x, pos.y, size.width, size.height))
    } else {
        Ok((0, 0, 300, 400))
    }
}

#[tauri::command]
fn get_window_position(window: tauri::WebviewWindow) -> Result<(i32, i32), String> {
    let pos = window.outer_position().map_err(|e| e.to_string())?;
    Ok((pos.x, pos.y))
}

#[tauri::command]
fn set_window_size(window: tauri::WebviewWindow, width: u32, height: u32) -> Result<(), String> {
    window.set_size(tauri::Size::Logical(tauri::LogicalSize {
        width: width as f64,
        height: height as f64,
    })).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_autostart(enable: bool) -> Result<(), String> {
    config::set_autostart_registry(enable)
}

#[tauri::command]
fn close_config_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("config") {
        win.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn ensure_single_instance_or_exit() -> std::net::TcpListener {
    match std::net::TcpListener::bind("127.0.0.1:45678") {
        Ok(listener) => {
            let _ = listener.set_nonblocking(true);
            listener
        }
        Err(_) => {
            // Already running! Signal the primary instance to show windows
            if let Ok(mut stream) = std::net::TcpStream::connect("127.0.0.1:45678") {
                use std::io::Write;
                let _ = stream.write_all(b"show\n");
            }
            std::process::exit(0);
        }
    }
}

fn main() {
    let single_instance_listener = ensure_single_instance_or_exit();
    let history_tracker = Arc::new(Mutex::new(history::SessionTracker::new()));
    let history_for_loop = Arc::clone(&history_tracker);
    let daily_tracker = Arc::new(Mutex::new(history::DailyTracker::new()));
    let daily_for_loop = Arc::clone(&daily_tracker);

    let app = tauri::Builder::default()
        .manage(HistoryState(Arc::clone(&history_tracker)))
        .manage(DailyHistoryState(Arc::clone(&daily_tracker)))
        .manage(HistoryWindowLock(tokio::sync::Mutex::new(())))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(move |app| {
            let app_handle_for_ipc = app.handle().clone();

            // Background task to handle second-instance activation requests via TCP
            tauri::async_runtime::spawn(async move {
                let listener = single_instance_listener;
                loop {
                    match listener.accept() {
                        Ok((_, _)) => {
                            // Secondary instance launched: bring config and main windows to front
                            if let Some(win) = app_handle_for_ipc.get_webview_window("config") {
                                let _ = win.show();
                                let _ = win.unminimize();
                                let _ = win.set_focus();
                            }
                            if let Some(main_win) = app_handle_for_ipc.get_webview_window("main") {
                                let _ = main_win.show();
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            sleep(Duration::from_millis(250)).await;
                        }
                        Err(_) => {
                            sleep(Duration::from_millis(500)).await;
                        }
                    }
                }
            });

            // 1. Setup 'main' OSD window
            if let Some(main_window) = app.get_webview_window("main") {
                let initial_config = config::load_config();
                let _ = main_window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                    x: initial_config.pos_x,
                    y: initial_config.pos_y,
                }));
                let _ = main_window.set_ignore_cursor_events(true);
                let _ = main_window.set_skip_taskbar(true);
                let _ = main_window.set_shadow(false);
                let _ = main_window.show();
            }

            // 2. Setup 'config' window
            if let Some(config_window) = app.get_webview_window("config") {
                let _ = config_window.set_skip_taskbar(false);
                let config_clone = config_window.clone();
                config_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = config_clone.hide();
                    }
                });
                let _ = config_window.show();
                let _ = config_window.unminimize();
                let _ = config_window.set_focus();
            }

            // 3. System Tray Menu
            let show_i = MenuItem::with_id(app, "show", "แสดงหน้าจอ", true, None::<&str>)?;
            let hide_i = MenuItem::with_id(app, "hide", "ซ่อนหน้าจอ", true, None::<&str>)?;
            let config_i = MenuItem::with_id(app, "config", "ตั้งค่า", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let exit_i = MenuItem::with_id(app, "exit", "ปิดโปรแกรม", true, None::<&str>)?;

            let menu = Menu::with_items(app, &[&show_i, &hide_i, &config_i, &sep, &exit_i])?;

            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                        }
                    }
                    "hide" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.hide();
                        }
                    }
                    "config" => {
                        if let Some(win) = app.get_webview_window("config") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "exit" => {
                        if let Ok(tracker) = app.state::<HistoryState>().0.lock() {
                            let _ = tracker.flush();
                        }
                        if let Ok(daily) = app.state::<DailyHistoryState>().0.lock() {
                            let _ = daily.flush();
                        }
                        std::process::exit(0);
                    }
                    _ => {}
                });

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }

            let _tray = tray_builder.build(app)?;

            // Background polling task
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut sys_monitor = monitor::SysMonitor::new();
                let mut gpu_monitor = gpu::GpuMonitor::new();
                let mut power_monitor = power::PowerMonitor::new();
                let mut fps_monitor = fps::FpsMonitor::new();
                let mut tick_count: u64 = 0;

                loop {
                    let cpu_usage = sys_monitor.get_cpu_usage();
                    let mut cpu_temp = sys_monitor.get_cpu_temp();
                    let ram_usage = sys_monitor.get_ram_usage();
                    let (net_down_kbps, net_up_kbps) = sys_monitor.get_network_usage();
                    let (disks, disk_summary) = sys_monitor.get_disk_usage();
                    let disk_usage = disks.first().map(|d| d.usage_percent).unwrap_or(0.0);
                    let (gpu_usage, mut gpu_temp) = gpu_monitor.get_gpu_stats();
                    let (power_w, energy_wh) = power_monitor.get_power_stats();
                    let fps = fps_monitor.get_fps();
                    let ping_ms = ping::get_ping("8.8.8.8").await;

                    if cpu_temp <= 0.0 && gpu_temp > 0.0 {
                        cpu_temp = gpu_temp;
                    } else if gpu_temp <= 0.0 && cpu_temp > 0.0 {
                        // For Intel iGPU / APUs where GPU shares the processor silicon package
                        gpu_temp = cpu_temp;
                    }

                    tick_count += 1;
                    let sample = history::SessionSample {
                        cpu_usage,
                        cpu_temp,
                        gpu_usage,
                        gpu_temp,
                        ram_usage,
                        disk_usage,
                        power_w,
                        energy_wh,
                    };
                    if let Ok(mut tracker) = history_for_loop.lock() {
                        tracker.record_sample(sample);
                        if tick_count == 1 || tick_count % 10 == 0 {
                            let _ = tracker.flush();
                        }
                    }
                    if let Ok(mut daily) = daily_for_loop.lock() {
                        daily.record_sample(sample);
                        if tick_count == 1 || tick_count % 30 == 0 {
                            let _ = daily.flush();
                        }
                    }

                    let payload = models::TelemetryUpdate {
                        cpu_usage,
                        cpu_temp,
                        ram_usage,
                        gpu_usage,
                        gpu_temp,
                        net_down_kbps,
                        net_up_kbps,
                        ping_ms,
                        disks,
                        disk_summary,
                        power_w,
                        energy_wh,
                        fps,
                    };

                    let _ = app_handle.emit("telemetry-update", payload);

                    sleep(Duration::from_secs(1)).await;
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            reset_config,
            set_ignore_cursor,
            set_window_position,
            set_main_position,
            get_screen_resolution,
            get_main_rect,
            get_window_position,
            set_window_size,
            set_autostart,
            close_config_window,
            get_session_history,
            clear_session_history,
            get_daily_history,
            clear_daily_history,
            open_history_window
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            if let Ok(tracker) = app_handle.state::<HistoryState>().0.lock() {
                let _ = tracker.flush();
            }
        }
    });
}
