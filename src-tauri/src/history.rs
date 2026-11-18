use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const MAX_HISTORY_SESSIONS: usize = 100;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SessionRecord {
    pub id: String,
    pub start_timestamp: u64,
    pub end_timestamp: u64,
    pub duration_sec: u64,
    pub sample_count: u64,
    pub avg_cpu_usage: f64,
    pub avg_cpu_temp: f64,
    pub avg_gpu_usage: f64,
    pub avg_gpu_temp: f64,
    pub avg_ram_usage: f64,
    pub avg_disk_usage: f64,
    pub avg_power_w: f64,
    pub total_energy_wh: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct SessionSample {
    pub cpu_usage: f64,
    pub cpu_temp: f64,
    pub gpu_usage: f64,
    pub gpu_temp: f64,
    pub ram_usage: f64,
    pub disk_usage: f64,
    pub power_w: f64,
    pub energy_wh: f64,
}

pub fn current_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn get_history_path() -> PathBuf {
    let config_path = crate::config::get_config_path();
    let dir = config_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    dir.join("history.json")
}

pub fn load_history_from_path(path: &Path) -> Vec<SessionRecord> {
    if !path.exists() {
        return Vec::new();
    }
    match fs::read_to_string(path) {
        Ok(content) => {
            let mut records = serde_json::from_str::<Vec<SessionRecord>>(&content).unwrap_or_default();
            records.sort_by(|a, b| b.start_timestamp.cmp(&a.start_timestamp));
            records.truncate(MAX_HISTORY_SESSIONS);
            records
        }
        Err(_) => Vec::new(),
    }
}

pub fn save_history_to_path(path: &Path, records: &[SessionRecord]) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create history dir: {}", e))?;
    }
    let json = serde_json::to_string_pretty(records).map_err(|e| e.to_string())?;
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, &json).map_err(|e| format!("Failed to write temp history {:?}: {}", tmp_path, e))?;
    if let Err(rename_err) = fs::rename(&tmp_path, path) {
        // Fallback to direct write if rename fails on Windows due to file lock
        let _ = fs::remove_file(&tmp_path);
        fs::write(path, &json).map_err(|e| {
            format!(
                "Failed to write history {:?} (rename err: {}, write err: {})",
                path, rename_err, e
            )
        })?;
    }
    Ok(())
}

pub struct SessionTracker {
    current: SessionRecord,
    temp_cpu_samples: u64,
    temp_gpu_samples: u64,
    past_sessions: Vec<SessionRecord>,
    history_path: PathBuf,
}

impl SessionTracker {
    pub fn new() -> Self {
        Self::with_path(get_history_path())
    }

    pub fn with_path(history_path: PathBuf) -> Self {
        let now = current_unix_secs();
        Self::with_path_and_start(history_path, now)
    }

    pub fn with_path_and_start(history_path: PathBuf, start_ts: u64) -> Self {
        let past_sessions = load_history_from_path(&history_path);
        let current = SessionRecord {
            id: start_ts.to_string(),
            start_timestamp: start_ts,
            end_timestamp: start_ts,
            duration_sec: 0,
            sample_count: 0,
            avg_cpu_usage: 0.0,
            avg_cpu_temp: 0.0,
            avg_gpu_usage: 0.0,
            avg_gpu_temp: 0.0,
            avg_ram_usage: 0.0,
            avg_disk_usage: 0.0,
            avg_power_w: 0.0,
            total_energy_wh: 0.0,
        };
        Self {
            current,
            temp_cpu_samples: 0,
            temp_gpu_samples: 0,
            past_sessions,
            history_path,
        }
    }

    pub fn record_sample(&mut self, sample: SessionSample) {
        self.record_sample_at(sample, current_unix_secs());
    }

    pub fn record_sample_at(&mut self, sample: SessionSample, now_ts: u64) {
        self.current.sample_count += 1;
        let n = self.current.sample_count as f64;

        self.current.avg_cpu_usage += (sample.cpu_usage - self.current.avg_cpu_usage) / n;
        self.current.avg_gpu_usage += (sample.gpu_usage - self.current.avg_gpu_usage) / n;
        self.current.avg_ram_usage += (sample.ram_usage - self.current.avg_ram_usage) / n;
        self.current.avg_disk_usage += (sample.disk_usage - self.current.avg_disk_usage) / n;
        self.current.avg_power_w += (sample.power_w - self.current.avg_power_w) / n;

        if sample.cpu_temp > 0.0 {
            self.temp_cpu_samples += 1;
            let tn = self.temp_cpu_samples as f64;
            self.current.avg_cpu_temp += (sample.cpu_temp - self.current.avg_cpu_temp) / tn;
        }

        if sample.gpu_temp > 0.0 {
            self.temp_gpu_samples += 1;
            let tn = self.temp_gpu_samples as f64;
            self.current.avg_gpu_temp += (sample.gpu_temp - self.current.avg_gpu_temp) / tn;
        }

        self.current.end_timestamp = now_ts.max(self.current.start_timestamp);
        self.current.duration_sec = self
            .current
            .end_timestamp
            .saturating_sub(self.current.start_timestamp);
        self.current.total_energy_wh = sample.energy_wh;
    }

    pub fn get_all_sessions(&self) -> Vec<SessionRecord> {
        let mut all = Vec::with_capacity(1 + self.past_sessions.len());
        all.push(self.current.clone());
        all.extend(self.past_sessions.iter().cloned());
        all.truncate(MAX_HISTORY_SESSIONS);
        all
    }

    pub fn flush(&self) -> Result<(), String> {
        let all = self.get_all_sessions();
        save_history_to_path(&self.history_path, &all)
    }

    pub fn clear_past_sessions(&mut self) -> Result<(), String> {
        self.past_sessions.clear();
        self.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_history_file(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir()
            .join(format!("md_monitor_test_{}_{}", name, nonce))
            .join("history.json")
    }

    #[test]
    fn test_running_averages_and_temp_filtering() {
        let path = temp_history_file("avg");
        let mut tracker = SessionTracker::with_path_and_start(path.clone(), 1000);

        // Sample 1: cpu_temp = 0.0 (should be excluded from avg_cpu_temp), gpu_temp = 50.0
        tracker.record_sample_at(
            SessionSample {
                cpu_usage: 20.0,
                cpu_temp: 0.0,
                gpu_usage: 10.0,
                gpu_temp: 50.0,
                ram_usage: 40.0,
                disk_usage: 5.0,
                power_w: 60.0,
                energy_wh: 0.016,
            },
            1001,
        );

        // Sample 2: cpu_temp = 60.0, gpu_temp = 0.0 (should be excluded from avg_gpu_temp)
        tracker.record_sample_at(
            SessionSample {
                cpu_usage: 40.0,
                cpu_temp: 60.0,
                gpu_usage: 30.0,
                gpu_temp: 0.0,
                ram_usage: 60.0,
                disk_usage: 15.0,
                power_w: 100.0,
                energy_wh: 0.044,
            },
            1005,
        );

        // Sample 3: cpu_temp = 70.0, gpu_temp = 70.0
        tracker.record_sample_at(
            SessionSample {
                cpu_usage: 60.0,
                cpu_temp: 70.0,
                gpu_usage: 50.0,
                gpu_temp: 70.0,
                ram_usage: 50.0,
                disk_usage: 10.0,
                power_w: 80.0,
                energy_wh: 0.066,
            },
            1010,
        );

        let sessions = tracker.get_all_sessions();
        assert_eq!(sessions.len(), 1);
        let cur = &sessions[0];
        assert_eq!(cur.id, "1000");
        assert_eq!(cur.start_timestamp, 1000);
        assert_eq!(cur.end_timestamp, 1010);
        assert_eq!(cur.duration_sec, 10);
        assert_eq!(cur.sample_count, 3);
        assert!((cur.avg_cpu_usage - 40.0).abs() < 1e-9);
        assert!((cur.avg_cpu_temp - 65.0).abs() < 1e-9);
        assert!((cur.avg_gpu_usage - 30.0).abs() < 1e-9);
        assert!((cur.avg_gpu_temp - 60.0).abs() < 1e-9);
        assert!((cur.avg_ram_usage - 50.0).abs() < 1e-9);
        assert!((cur.avg_disk_usage - 10.0).abs() < 1e-9);
        assert!((cur.avg_power_w - 80.0).abs() < 1e-9);
        assert!((cur.total_energy_wh - 0.066).abs() < 1e-9);
    }

    #[test]
    fn test_save_load_cap_and_clear() {
        let path = temp_history_file("persist");

        // Pre-populate 105 past sessions
        let mut initial = Vec::new();
        for i in 0..105u64 {
            initial.push(SessionRecord {
                id: format!("{}", 100 + i),
                start_timestamp: 100 + i,
                end_timestamp: 200 + i,
                duration_sec: 100,
                sample_count: 10,
                avg_cpu_usage: 25.0,
                avg_cpu_temp: 55.0,
                avg_gpu_usage: 15.0,
                avg_gpu_temp: 48.0,
                avg_ram_usage: 50.0,
                avg_disk_usage: 5.0,
                avg_power_w: 75.0,
                total_energy_wh: 2.5,
            });
        }
        save_history_to_path(&path, &initial).expect("save initial");

        // Start a new tracker at timestamp 5000
        let mut tracker = SessionTracker::with_path_and_start(path.clone(), 5000);
        tracker.record_sample_at(
            SessionSample {
                cpu_usage: 10.0,
                cpu_temp: 45.0,
                gpu_usage: 5.0,
                gpu_temp: 40.0,
                ram_usage: 30.0,
                disk_usage: 2.0,
                power_w: 50.0,
                energy_wh: 1.25,
            },
            5010,
        );
        tracker.flush().expect("flush");

        // Reload and verify cap = 100, newest first
        let loaded = load_history_from_path(&path);
        assert_eq!(loaded.len(), MAX_HISTORY_SESSIONS);
        assert_eq!(loaded[0].id, "5000");
        assert_eq!(loaded[1].id, "204"); // 100 + 104

        // Clear past sessions, current session must remain
        tracker.clear_past_sessions().expect("clear");
        let after_clear = load_history_from_path(&path);
        assert_eq!(after_clear.len(), 1);
        assert_eq!(after_clear[0].id, "5000");

        let _ = fs::remove_dir_all(path.parent().unwrap());
    }
}
