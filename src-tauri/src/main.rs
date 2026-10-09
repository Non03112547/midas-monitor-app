// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod models;
mod monitor;
mod gpu;
mod ping;

use tauri::Manager;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tokio::time::{sleep, Duration};
use tauri::Emitter;
use models::AppConfig;

#[tauri::command]
fn get_config() -> AppConfig {
    config::load_config()
}

#[tauri::command]
fn save_config(app: tauri::AppHandle, config: AppConfig) -> Result<(), String> {
    config::save_config(&config)?;
    let _ = config::set_autostart_registry(config.autostart);
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
    let _ = config::save_config(&cfg);
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

fn main() {
    tauri::Builder::default()
        .setup(|app| {
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
                let _ = config_window.set_skip_taskbar(true);
                let config_clone = config_window.clone();
                config_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = config_clone.hide();
                    }
                });
                let _ = config_window.show();
                let _ = config_window.set_focus();
            }

            // 3. System Tray Menu (matching Python monitor.pyw lines 748-766)
            let show_i = MenuItem::with_id(app, "show", "แสดงหน้าจอ (Show)", true, None::<&str>)?;
            let hide_i = MenuItem::with_id(app, "hide", "ซ่อนหน้าจอ (Hide)", true, None::<&str>)?;
            let config_i = MenuItem::with_id(app, "config", "ตั้งค่า (Config)", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let exit_i = MenuItem::with_id(app, "exit", "ปิดโปรแกรม (Exit)", true, None::<&str>)?;

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
                
                loop {
                    let cpu_usage = sys_monitor.get_cpu_usage();
                    let mut cpu_temp = sys_monitor.get_cpu_temp();
                    let ram_usage = sys_monitor.get_ram_usage();
                    let (net_down_kbps, net_up_kbps) = sys_monitor.get_network_usage();
                    let (disks, disk_summary) = sys_monitor.get_disk_usage();
                    let (gpu_usage, gpu_temp) = gpu_monitor.get_gpu_stats();
                    let ping_ms = ping::get_ping("8.8.8.8").await;

                    if cpu_temp <= 0.0 && gpu_temp > 0.0 {
                        cpu_temp = gpu_temp;
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
            set_autostart
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
