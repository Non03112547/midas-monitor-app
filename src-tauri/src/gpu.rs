use nvml_wrapper::Nvml;
use libloading::{Library, Symbol};
use std::ffi::c_void;

#[repr(C)]
struct ADLPMActivity {
    i_size: i32,
    i_engine_clock: i32,
    i_memory_clock: i32,
    i_vddc: i32,
    i_activity_percent: i32,
    i_current_performance_level: i32,
    i_current_bus_speed: i32,
    i_current_bus_lanes: i32,
    i_maximum_bus_lanes: i32,
    i_reserved: i32,
}

#[repr(C)]
struct ADLTemperature {
    i_size: i32,
    i_temperature: i32,
}

unsafe extern "C" fn adl_malloc(size: i32) -> *mut c_void {
    if size <= 0 {
        return std::ptr::null_mut();
    }
    let layout = match std::alloc::Layout::from_size_align(size as usize, 8) {
        Ok(l) => l,
        Err(_) => return std::ptr::null_mut(),
    };
    std::alloc::alloc(layout) as *mut c_void
}

struct AdlContext {
    _lib: Library,
    get_activity: unsafe extern "C" fn(i32, *mut ADLPMActivity) -> i32,
    get_temp_odn: Option<unsafe extern "C" fn(*mut c_void, i32, i32, *mut i32) -> i32>,
    get_temp_od6: Option<unsafe extern "C" fn(i32, *mut i32) -> i32>,
    get_temp_od5: Option<unsafe extern "C" fn(i32, i32, *mut ADLTemperature) -> i32>,
}

pub struct GpuMonitor {
    nvml: Option<Nvml>,
    adl: Option<AdlContext>,
}

impl GpuMonitor {
    pub fn new() -> Self {
        let nvml = Nvml::init().ok();
        let adl = Self::init_adl();
        Self { nvml, adl }
    }

    fn init_adl() -> Option<AdlContext> {
        unsafe {
            let lib = Library::new("atiadlxx.dll")
                .or_else(|_| Library::new("atiadlxy.dll"))
                .ok()?;

            let create: Symbol<unsafe extern "C" fn(unsafe extern "C" fn(i32) -> *mut c_void, i32) -> i32> = 
                lib.get(b"ADL_Main_Control_Create\0").ok()?;

            if create(adl_malloc, 1) != 0 {
                return None;
            }

            let get_activity_sym: Symbol<unsafe extern "C" fn(i32, *mut ADLPMActivity) -> i32> =
                lib.get(b"ADL_Overdrive5_CurrentActivity_Get\0").ok()?;
            let get_activity = *get_activity_sym;

            let get_temp_odn = lib.get::<unsafe extern "C" fn(*mut c_void, i32, i32, *mut i32) -> i32>(b"ADL2_OverdriveN_Temperature_Get\0")
                .ok()
                .map(|s| *s);

            let get_temp_od6 = lib.get::<unsafe extern "C" fn(i32, *mut i32) -> i32>(b"ADL_Overdrive6_Temperature_Get\0")
                .ok()
                .map(|s| *s);

            let get_temp_od5 = lib.get::<unsafe extern "C" fn(i32, i32, *mut ADLTemperature) -> i32>(b"ADL_Overdrive5_Temperature_Get\0")
                .ok()
                .map(|s| *s);

            Some(AdlContext {
                _lib: lib,
                get_activity,
                get_temp_odn,
                get_temp_od6,
                get_temp_od5,
            })
        }
    }

    pub fn get_gpu_stats(&mut self) -> (f64, f64) {
        // 1. Try Nvidia NVML
        if let Some(nvml) = &self.nvml {
            if let Ok(device) = nvml.device_by_index(0) {
                let util = device.utilization_rates().map(|u| u.gpu as f64).unwrap_or(0.0);
                let temp = device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                    .map(|t| t as f64)
                    .unwrap_or(0.0);
                if util > 0.0 || temp > 0.0 {
                    return (util, temp);
                }
            }
        }

        // 2. Try AMD ADL
        if let Some(adl) = &self.adl {
            let mut util = 0.0;
            let mut temp = 0.0;

            unsafe {
                let mut act = std::mem::zeroed::<ADLPMActivity>();
                act.i_size = std::mem::size_of::<ADLPMActivity>() as i32;
                if (adl.get_activity)(0, &mut act) == 0 {
                    util = act.i_activity_percent as f64;
                }

                // Check OverdriveN first (modern AMD RDNA / APU)
                if let Some(odn) = adl.get_temp_odn {
                    let mut t = 0i32;
                    if odn(std::ptr::null_mut(), 0, 1, &mut t) == 0 && t > 0 {
                        let t_val = t as f64;
                        if t_val > 1000.0 {
                            temp = t_val / 1000.0;
                        } else {
                            temp = t_val;
                        }
                    }
                }

                // Fallback to Overdrive6
                if temp == 0.0 {
                    if let Some(od6) = adl.get_temp_od6 {
                        let mut t = 0i32;
                        if od6(0, &mut t) == 0 && t > 0 {
                            let t_val = t as f64;
                            if t_val > 1000.0 {
                                temp = t_val / 1000.0;
                            } else {
                                temp = t_val;
                            }
                        }
                    }
                }

                // Fallback to Overdrive5
                if temp == 0.0 {
                    if let Some(od5) = adl.get_temp_od5 {
                        let mut t = std::mem::zeroed::<ADLTemperature>();
                        t.i_size = std::mem::size_of::<ADLTemperature>() as i32;
                        if od5(0, 0, &mut t) == 0 && t.i_temperature > 0 {
                            let t_val = t.i_temperature as f64;
                            if t_val > 1000.0 {
                                temp = t_val / 1000.0;
                            } else {
                                temp = t_val;
                            }
                        }
                    }
                }
            }

            return (util, temp);
        }

        (0.0, 0.0)
    }
}
