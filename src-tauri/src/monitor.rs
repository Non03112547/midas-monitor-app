use sysinfo::{System, Networks, Disks, Components};
use std::time::Instant;
use crate::models::DiskTelemetry;

pub struct SysMonitor {
    sys: System,
    networks: Networks,
    disks: Disks,
    components: Components,
    last_check: Instant,
}

impl SysMonitor {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            sys,
            networks: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
            components: Components::new_with_refreshed_list(),
            last_check: Instant::now(),
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
        self.disks.refresh();

        let mut disks_vec = Vec::new();
        let mut summary_parts = Vec::new();

        let mut cd_total: u64 = 0;
        let mut cd_available: u64 = 0;
        let mut cd_found = false;
        let mut cd_labels = Vec::new();

        for disk in &self.disks {
            let total = disk.total_space();
            let available = disk.available_space();
            if total > 0 {
                let used = total.saturating_sub(available);
                let usage_percent = (used as f64 / total as f64) * 100.0;
                let mount = disk.mount_point().to_string_lossy().to_string();
                let name = disk.name().to_string_lossy().to_string();
                
                // Format drive letter (e.g., "C:" from "C:\\" or "C:")
                let drive_label = mount.trim_end_matches(['\\', '/']).to_uppercase();
                let display_name = if drive_label.is_empty() { name.clone() } else { drive_label.clone() };

                disks_vec.push(DiskTelemetry {
                    name,
                    mount_point: mount,
                    usage_percent,
                });

                if drive_label.starts_with("C:") || drive_label.starts_with("D:") {
                    cd_total += total;
                    cd_available += available;
                    cd_found = true;
                    if !cd_labels.contains(&drive_label) {
                        cd_labels.push(drive_label);
                    }
                } else {
                    summary_parts.push(format!("{} {:.0}%", display_name, usage_percent));
                }
            }
        }

        if cd_found && cd_total > 0 {
            let cd_used = cd_total.saturating_sub(cd_available);
            let cd_percent = (cd_used as f64 / cd_total as f64) * 100.0;
            summary_parts.insert(0, format!("{:.0}%", cd_percent));
        }

        let summary = if summary_parts.is_empty() {
            "N/A".to_string()
        } else {
            summary_parts.join(" | ")
        };

        (disks_vec, summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_usage() {
        let mut sm = SysMonitor::new();
        let (disks, summary) = sm.get_disk_usage();
        println!("Test Disks count: {}, Summary: {}", disks.len(), summary);
        assert!(!summary.is_empty());
    }
}
