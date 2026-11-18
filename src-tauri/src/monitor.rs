use sysinfo::{System, Networks, Components};
use std::time::Instant;
use windows::core::w;
use windows::Win32::System::Performance::{
    PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterValue,
    PdhOpenQueryW, PDH_FMT_COUNTERVALUE, PDH_FMT_DOUBLE,
};
use crate::models::DiskTelemetry;

pub struct SysMonitor {
    sys: System,
    networks: Networks,
    components: Components,
    last_check: Instant,
    pdh_query: Option<isize>,
    pdh_counter: Option<isize>,
    pdh_is_idle: bool,
}

impl Drop for SysMonitor {
    fn drop(&mut self) {
        if let Some(query) = self.pdh_query.take() {
            unsafe {
                let _ = PdhCloseQuery(query);
            }
        }
    }
}

impl SysMonitor {
    fn init_disk_pdh() -> (Option<isize>, Option<isize>, bool) {
        unsafe {
            let mut query: isize = 0;
            if PdhOpenQueryW(None, 0, &mut query) != 0 {
                return (None, None, true);
            }

            let mut counter: isize = 0;
            let mut is_idle = true;

            if PdhAddEnglishCounterW(
                query,
                w!("\\PhysicalDisk(_Total)\\% Idle Time"),
                0,
                &mut counter,
            ) != 0
            {
                is_idle = false;
                if PdhAddEnglishCounterW(
                    query,
                    w!("\\PhysicalDisk(_Total)\\% Disk Time"),
                    0,
                    &mut counter,
                ) != 0
                {
                    let _ = PdhCloseQuery(query);
                    return (None, None, true);
                }
            }

            // Initial baseline sample required by PDH rate/percentage counters
            let _ = PdhCollectQueryData(query);
            (Some(query), Some(counter), is_idle)
        }
    }

    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let (pdh_query, pdh_counter, pdh_is_idle) = Self::init_disk_pdh();
        Self {
            sys,
            networks: Networks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
            last_check: Instant::now(),
            pdh_query,
            pdh_counter,
            pdh_is_idle,
        }
    }

    pub fn get_cpu_temp(&mut self) -> f64 {
        self.components.refresh();
        for c in &self.components {
            let label = c.label().to_lowercase();
            if label.contains("cpu") || label.contains("core") || label.contains("package") || label.contains("tctl") || label.contains("tdie") {
                let t = c.temperature();
                if t > 0.0 {
                    return t as f64;
                }
            }
        }
        for c in &self.components {
            let t = c.temperature();
            if t > 0.0 {
                return t as f64;
            }
        }
        0.0
    }

    pub fn get_cpu_usage(&mut self) -> f64 {
        self.sys.refresh_cpu_usage();
        let cpus = self.sys.cpus();
        if cpus.is_empty() {
            return 0.0;
        }
        let total_usage: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
        (total_usage / cpus.len() as f32) as f64
    }

    pub fn get_ram_usage(&mut self) -> f64 {
        self.sys.refresh_memory();
        let total = self.sys.total_memory() as f64;
        let used = self.sys.used_memory() as f64;
        if total == 0.0 {
            0.0
        } else {
            (used / total) * 100.0
        }
    }

    pub fn get_network_usage(&mut self) -> (f64, f64) {
        let elapsed = self.last_check.elapsed().as_secs_f64();
        self.last_check = Instant::now();
        
        self.networks.refresh();

        let mut rx_total = 0;
        let mut tx_total = 0;
        for (_interface_name, data) in &self.networks {
            rx_total += data.received();
            tx_total += data.transmitted();
        }

        let rx_kbps = (rx_total as f64 / 1024.0) / elapsed.max(0.1);
        let tx_kbps = (tx_total as f64 / 1024.0) / elapsed.max(0.1);

        (rx_kbps, tx_kbps)
    }

    pub fn get_disk_usage(&mut self) -> (Vec<DiskTelemetry>, String) {
        let active_percent = match (self.pdh_query, self.pdh_counter) {
            (Some(query), Some(counter)) => unsafe {
                if PdhCollectQueryData(query) == 0 {
                    let mut value = PDH_FMT_COUNTERVALUE::default();
                    if PdhGetFormattedCounterValue(counter, PDH_FMT_DOUBLE, None, &mut value) == 0 {
                        let raw = value.Anonymous.doubleValue;
                        if raw.is_finite() {
                            if self.pdh_is_idle {
                                (100.0 - raw).clamp(0.0, 100.0)
                            } else {
                                raw.clamp(0.0, 100.0)
                            }
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            },
            _ => 0.0,
        };

        let disks_vec = vec![DiskTelemetry {
            name: "PhysicalDisk(_Total)".to_string(),
            mount_point: "_Total".to_string(),
            usage_percent: active_percent,
        }];
        let summary = format!("{:.0}%", active_percent);

        (disks_vec, summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_usage() {
        let mut sm = SysMonitor::new();
        std::thread::sleep(std::time::Duration::from_millis(100));
        let (disks, summary) = sm.get_disk_usage();
        println!("Test Disks count: {}, Summary: {}", disks.len(), summary);
        assert_eq!(disks.len(), 1);
        assert!(summary.ends_with('%'));
        assert!((0.0..=100.0).contains(&disks[0].usage_percent));
    }
}
