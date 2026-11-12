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

#[repr(C)]
struct PdhFmtCounterValue {
    c_status: u32,
    _dummy: u32,
    double_value: f64,
}

#[repr(C)]
struct PdhFmtCounterValueItemW {
    sz_name: *mut u16,
    value: PdhFmtCounterValue,
}

struct UniversalGpuContext {
    _pdh_lib: Library,
    query: isize,
    counter: isize,
    collect_fn: unsafe extern "system" fn(isize) -> i32,
    get_array_fn: unsafe extern "system" fn(isize, u32, *mut u32, *mut u32, *mut PdhFmtCounterValueItemW) -> i32,
    close_fn: unsafe extern "system" fn(isize) -> i32,
}

impl Drop for UniversalGpuContext {
    fn drop(&mut self) {
        unsafe {
            (self.close_fn)(self.query);
        }
    }
}

fn to_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

impl UniversalGpuContext {
    fn new() -> Option<Self> {
        unsafe {
            let lib = Library::new("pdh.dll").ok()?;
            let open_fn: Symbol<unsafe extern "system" fn(*const u16, usize, *mut isize) -> i32> = lib.get(b"PdhOpenQueryW\0").ok()?;
            let open_fn = *open_fn;
            let add_fn: Symbol<unsafe extern "system" fn(isize, *const u16, usize, *mut isize) -> i32> = lib.get(b"PdhAddEnglishCounterW\0").ok()?;
            let add_fn = *add_fn;
            let collect_fn: Symbol<unsafe extern "system" fn(isize) -> i32> = lib.get(b"PdhCollectQueryData\0").ok()?;
            let collect_fn = *collect_fn;
            let get_array_fn: Symbol<unsafe extern "system" fn(isize, u32, *mut u32, *mut u32, *mut PdhFmtCounterValueItemW) -> i32> = lib.get(b"PdhGetFormattedCounterArrayW\0").ok()?;
            let get_array_fn = *get_array_fn;
            let close_fn: Symbol<unsafe extern "system" fn(isize) -> i32> = lib.get(b"PdhCloseQuery\0").ok()?;
            let close_fn = *close_fn;

            let mut query = 0isize;
            if open_fn(std::ptr::null(), 0, &mut query) != 0 {
                return None;
            }

            let mut counter = 0isize;
            // Query 3D engine utilization across all graphics adapters (Intel Arc / Iris Xe / UHD, NVIDIA, AMD)
            let counter_path = to_wide(r"\GPU Engine(*_engtype_3D)\Utilization Percentage");
            if add_fn(query, counter_path.as_ptr(), 0, &mut counter) != 0 {
                close_fn(query);
                return None;
            }

            // Prime the query
            let _ = collect_fn(query);

            Some(Self {
                _pdh_lib: lib,
                query,
                counter,
                collect_fn,
                get_array_fn,
                close_fn,
            })
        }
    }

    fn get_utilization(&self) -> f64 {
        unsafe {
            if (self.collect_fn)(self.query) != 0 {
                return 0.0;
            }

            let mut buffer_size = 0u32;
            let mut item_count = 0u32;

            let status = (self.get_array_fn)(
                self.counter,
                0x00000200, // PDH_FMT_DOUBLE
                &mut buffer_size,
                &mut item_count,
                std::ptr::null_mut(),
            );

            // 0x800007D2 = PDH_MORE_DATA
            if status != 0 && (status as u32) != 0x800007D2 {
                return 0.0;
            }

            if buffer_size == 0 {
                return 0.0;
            }

            let mut buffer: Vec<u8> = vec![0u8; buffer_size as usize];
            let status = (self.get_array_fn)(
                self.counter,
                0x00000200,
                &mut buffer_size,
                &mut item_count,
                buffer.as_mut_ptr() as *mut PdhFmtCounterValueItemW,
            );

            if status != 0 || item_count == 0 {
                return 0.0;
            }

            let items = buffer.as_ptr() as *const PdhFmtCounterValueItemW;
            let mut total_util = 0.0;

            for i in 0..item_count as usize {
                let item = &*items.add(i);
                if item.value.c_status == 0 && item.value.double_value > 0.0 {
                    total_util += item.value.double_value;
                }
            }

            if total_util > 100.0 {
                total_util = 100.0;
            }

            total_util
        }
    }
}

pub struct GpuMonitor {
    nvml: Option<Nvml>,
    adl: Option<AdlContext>,
    universal: Option<UniversalGpuContext>,
}

impl GpuMonitor {
    pub fn new() -> Self {
        let nvml = Nvml::init().ok();
        let adl = Self::init_adl();
        let universal = UniversalGpuContext::new();
        Self { nvml, adl, universal }
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
                return (util, temp);
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

            if util > 0.0 || temp > 0.0 {
                return (util, temp);
            }
        }

        // 3. Try Universal WDDM GPU (Intel Arc, Intel Iris Xe, Intel UHD, Snapdragon, etc.)
        if let Some(univ) = &self.universal {
            let util = univ.get_utilization();
            return (util, 0.0);
        }

        (0.0, 0.0)
    }
}
