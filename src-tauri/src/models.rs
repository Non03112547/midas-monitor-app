use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DiskTelemetry {
    pub name: String,
    pub mount_point: String,
    pub usage_percent: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TelemetryUpdate {
    pub cpu_usage: f64,
    pub cpu_temp: f64,
    pub ram_usage: f64,
    pub gpu_usage: f64,
    pub gpu_temp: f64,
    pub net_down_kbps: f64,
    pub net_up_kbps: f64,
    pub ping_ms: f64,
    pub disks: Vec<DiskTelemetry>,
    pub disk_summary: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct AppConfig {
    pub pos_x: i32,
    pub pos_y: i32,
    pub bg_r: u8,
    pub bg_g: u8,
    pub bg_b: u8,
    pub bg_alpha: u8,
    pub font_size: u32,
    pub text_color: String,
    pub outline_color: String,
    pub has_outline: bool,
    pub show_cpu: bool,
    pub show_gpu: bool,
    pub show_ram: bool,
    pub show_disk: bool,
    pub show_net: bool,
    pub show_ping: bool,
    pub show_fps: bool,
    pub layout_direction: String,
    pub autostart: bool,
    pub language: String,
    pub config_theme: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            pos_x: 28,
            pos_y: 19,
            bg_r: 0,
            bg_g: 0,
            bg_b: 0,
            bg_alpha: 60,
            font_size: 20,
            text_color: "#ffffff".to_string(),
            outline_color: "#000000".to_string(),
            has_outline: true,
            show_cpu: true,
            show_gpu: true,
            show_ram: true,
            show_disk: false,
            show_net: false,
            show_ping: true,
            show_fps: false,
            layout_direction: "vertical".to_string(),
            autostart: true,
            language: "th".to_string(),
            config_theme: "dark".to_string(),
        }
    }
}
