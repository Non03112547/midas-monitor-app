use std::path::PathBuf;
use std::fs;
use crate::models::AppConfig;

pub fn get_config_path() -> PathBuf {
    // 1. Check if config.json exists in current working directory
    let cwd_path = PathBuf::from("config.json");
    if cwd_path.exists() {
        return cwd_path;
    }

    // 2. Check in D:\note-dev\MD_Monitor\config.json
    let dev_path = PathBuf::from(r"D:\note-dev\MD_Monitor\config.json");
    if dev_path.exists() {
        return dev_path;
    }

    // 3. Check next to current exe
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(dir) = exe_path.parent() {
            let exe_cfg = dir.join("config.json");
            if exe_cfg.exists() {
                return exe_cfg;
            }
        }
    }

    // 4. Fallback to D:\note-dev\MD_Monitor\config.json or exe dir
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(dir) = exe_path.parent() {
            return dir.join("config.json");
        }
    }

    dev_path
}

pub fn load_config() -> AppConfig {
    let path = get_config_path();
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                return cfg;
            }
        }
    }
    AppConfig::default()
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = get_config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json_str = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, json_str).map_err(|e| format!("Failed to write config to {:?}: {}", path, e))?;
    Ok(())
}

pub fn set_autostart_registry(enable: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
        let app_name = "MD Monitor";

        if enable {
            let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
            let exe_str = exe_path.to_str().ok_or("Invalid executable path")?;
            let cmd_val = format!("\"{}\"", exe_str);

            let output = std::process::Command::new("reg")
                .args(["add", key, "/v", app_name, "/t", "REG_SZ", "/d", &cmd_val, "/f"])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map_err(|e| format!("Failed to execute reg.exe: {}", e))?;

            if !output.status.success() {
                let err = String::from_utf8_lossy(&output.stderr);
                return Err(format!("reg add failed: {}", err));
            }
        } else {
            let output = std::process::Command::new("reg")
                .args(["delete", key, "/v", app_name, "/f"])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map_err(|e| format!("Failed to execute reg.exe: {}", e))?;

            let _ = output;
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = enable;
        Ok(())
    }
}
