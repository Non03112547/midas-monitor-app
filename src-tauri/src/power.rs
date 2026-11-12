use std::time::Instant;
use libloading::{Library, Symbol};
use std::ffi::c_void;

#[repr(C)]
struct PdhFmtCounterValue {
    c_status: u32,
    _dummy: u32,
    double_value: f64,
}

#[repr(C)]
struct SystemBatteryState {
    ac_on_line: u8,
    battery_present: u8,
    charging: u8,
    discharging: u8,
    spare1: [u8; 4],
    max_capacity: u32,
    remaining_capacity: u32,
    rate: i32, // in mW, negative when discharging
    estimated_time: u32,
    default_alert1: u32,
    default_alert2: u32,
}

pub struct PowerMonitor {
    _pdh_lib: Option<Library>,
    _powrprof_lib: Option<Library>,
    pdh_collect: Option<unsafe extern "system" fn(isize) -> i32>,
    pdh_get_value: Option<unsafe extern "system" fn(isize, u32, *mut u32, *mut PdhFmtCounterValue) -> i32>,
    pdh_close: Option<unsafe extern "system" fn(isize) -> i32>,
    call_nt_power: Option<unsafe extern "system" fn(i32, *const c_void, u32, *mut c_void, u32) -> i32>,
    query_handle: isize,
    counter_handle: isize,
    last_sample_time: Instant,
    accumulated_energy_wh: f64,
}

fn to_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

impl PowerMonitor {
    pub fn new() -> Self {
        let mut pdh_collect = None;
        let mut pdh_get_value = None;
        let mut pdh_close = None;
        let mut query_handle = 0isize;
        let mut counter_handle = 0isize;

        let pdh_lib = unsafe {
            match Library::new("pdh.dll") {
                Ok(lib) => {
                    let open: Option<Symbol<unsafe extern "system" fn(*const u16, usize, *mut isize) -> i32>> = lib.get(b"PdhOpenQueryW\0").ok();
                    let add: Option<Symbol<unsafe extern "system" fn(isize, *const u16, usize, *mut isize) -> i32>> = lib.get(b"PdhAddEnglishCounterW\0").ok();
                    let collect: Option<Symbol<unsafe extern "system" fn(isize) -> i32>> = lib.get(b"PdhCollectQueryData\0").ok();
                    let get_val: Option<Symbol<unsafe extern "system" fn(isize, u32, *mut u32, *mut PdhFmtCounterValue) -> i32>> = lib.get(b"PdhGetFormattedCounterValue\0").ok();
                    let close: Option<Symbol<unsafe extern "system" fn(isize) -> i32>> = lib.get(b"PdhCloseQuery\0").ok();

                    if let (Some(open_fn), Some(add_fn), Some(collect_fn), Some(get_val_fn), Some(close_fn)) = (open, add, collect, get_val, close) {
                        let mut q = 0isize;
                        if open_fn(std::ptr::null(), 0, &mut q) == 0 {
                            let mut c = 0isize;
                            // Try hardware RAPL pkg power first
                            let paths = [
                                r"\Energy Meter(rapl_package0_pkg)\Power",
                                r"\Energy Meter(_total)\Power",
                                r"\Power Meter(power meter (0))\Power",
                            ];
                            let mut added = false;
                            for p in &paths {
                                let wpath = to_wide(p);
                                if add_fn(q, wpath.as_ptr(), 0, &mut c) == 0 {
                                    added = true;
                                    break;
                                }
                            }

                            if added {
                                query_handle = q;
                                counter_handle = c;
                                pdh_collect = Some(*collect_fn);
                                pdh_get_value = Some(*get_val_fn);
                                pdh_close = Some(*close_fn);
                                // Initial collection to prime PDH calculation
                                let _ = (*collect_fn)(q);
                            } else {
                                let _ = close_fn(q);
                            }
                        }
                    }
                    Some(lib)
                }
                Err(_) => None,
            }
        };

        let (powrprof_lib, call_nt_power) = unsafe {
            match Library::new("powrprof.dll") {
                Ok(lib) => {
                    let func: Option<Symbol<unsafe extern "system" fn(i32, *const c_void, u32, *mut c_void, u32) -> i32>> = 
                        lib.get(b"CallNtPowerInformation\0").ok();
                    let f = func.map(|sym| *sym);
                    (Some(lib), f)
                }
                Err(_) => (None, None),
            }
        };

        Self {
            _pdh_lib: pdh_lib,
            _powrprof_lib: powrprof_lib,
            pdh_collect,
            pdh_get_value,
            pdh_close,
            call_nt_power,
            query_handle,
            counter_handle,
            last_sample_time: Instant::now(),
            accumulated_energy_wh: 0.0,
        }
    }

    pub fn get_power_stats(&mut self) -> (f64, f64) {
        let elapsed = self.last_sample_time.elapsed().as_secs_f64();
        self.last_sample_time = Instant::now();

        let mut power_w = 0.0;

        // 1. Try PDH hardware energy meter
        if let (Some(collect_fn), Some(get_val_fn)) = (self.pdh_collect, self.pdh_get_value) {
            unsafe {
                if collect_fn(self.query_handle) == 0 {
                    let mut val = PdhFmtCounterValue { c_status: 0, _dummy: 0, double_value: 0.0 };
                    // 0x00000200 = PDH_FMT_DOUBLE
                    if get_val_fn(self.counter_handle, 0x00000200, std::ptr::null_mut(), &mut val) == 0 {
                        if val.c_status == 0 && val.double_value > 0.0 {
                            power_w = val.double_value / 1000.0;
                        }
                    }
                }
            }
        }

        // 2. Fallback to battery discharge rate if discharging
        if power_w <= 0.0 {
            if let Some(call_fn) = self.call_nt_power {
                unsafe {
                    let mut state = std::mem::zeroed::<SystemBatteryState>();
                    // 5 = SystemBatteryState
                    let ret = call_fn(
                        5,
                        std::ptr::null(),
                        0,
                        &mut state as *mut _ as *mut c_void,
                        std::mem::size_of::<SystemBatteryState>() as u32,
                    );
                    if ret == 0 {
                        if state.discharging != 0 && state.rate != 0 {
                            let mw = (state.rate as f64).abs();
                            power_w = mw / 1000.0;
                        }
                    }
                }
            }
        }

        // Integrate energy consumption (Watt-hours) for current round
        if elapsed > 0.0 && power_w > 0.0 {
            let delta_wh = power_w * (elapsed / 3600.0);
            self.accumulated_energy_wh += delta_wh;
        }

        (power_w, self.accumulated_energy_wh)
    }
}

impl Drop for PowerMonitor {
    fn drop(&mut self) {
        if let Some(close_fn) = self.pdh_close {
            if self.query_handle != 0 {
                unsafe {
                    let _ = close_fn(self.query_handle);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_stats() {
        let mut pm = PowerMonitor::new();
        let (w, wh) = pm.get_power_stats();
        println!("Test Power: {} W, Energy: {} Wh", w, wh);
    }
}
