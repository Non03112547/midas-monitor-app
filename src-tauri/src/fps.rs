use std::ffi::c_void;
use libloading::{Library, Symbol};
use windows::Win32::Graphics::Dwm::{DwmGetCompositionTimingInfo, DWM_TIMING_INFO};
use windows::Win32::Graphics::Gdi::{EnumDisplaySettingsW, DEVMODEW, ENUM_CURRENT_SETTINGS};

unsafe extern "C" fn adl_malloc(size: i32) -> *mut c_void {
    if size <= 0 {
        return std::ptr::null_mut();
    }
    let layout = match std::alloc::Layout::from_size_align(size as usize, 8) {
        Ok(l) => l,
        Err(_) => return std::ptr::null_mut(),
    };
    unsafe { std::alloc::alloc(layout) as *mut c_void }
}

struct AdlFpsContext {
    _lib: Library,
    context: *mut c_void,
    adapter_count: i32,
    get_fn: unsafe extern "C" fn(*mut c_void, i32, i32, *mut f32) -> i32,
}

unsafe impl Send for AdlFpsContext {}
unsafe impl Sync for AdlFpsContext {}

pub struct FpsMonitor {
    adl: Option<AdlFpsContext>,
}

unsafe impl Send for FpsMonitor {}
unsafe impl Sync for FpsMonitor {}

impl FpsMonitor {
    pub fn new() -> Self {
        let adl = Self::init_adl();
        Self { adl }
    }

    fn init_adl() -> Option<AdlFpsContext> {
        unsafe {
            let lib = Library::new("atiadlxx.dll")
                .or_else(|_| Library::new("atiadlxy.dll"))
                .ok()?;

            type Adl2Create = unsafe extern "C" fn(unsafe extern "C" fn(i32) -> *mut c_void, i32, *mut *mut c_void) -> i32;
            let create_fn: Symbol<Adl2Create> = lib.get(b"ADL2_Main_Control_Create\0").ok()?;

            let mut context: *mut c_void = std::ptr::null_mut();
            if create_fn(adl_malloc, 1, &mut context) != 0 || context.is_null() {
                return None;
            }

            type NumAdapters = unsafe extern "C" fn(*mut c_void, *mut i32) -> i32;
            let num_fn: Symbol<NumAdapters> = lib.get(b"ADL2_Adapter_NumberOfAdapters_Get\0").ok()?;
            let mut adapter_count = 0i32;
            let _ = num_fn(context, &mut adapter_count);

            type FrameMetricsStart = unsafe extern "C" fn(*mut c_void, i32, i32) -> i32;
            type FrameMetricsGet = unsafe extern "C" fn(*mut c_void, i32, i32, *mut f32) -> i32;
            let start_fn: Symbol<FrameMetricsStart> = lib.get(b"ADL2_Adapter_FrameMetrics_Start\0").ok()?;
            let get_fn_sym: Symbol<FrameMetricsGet> = lib.get(b"ADL2_Adapter_FrameMetrics_Get\0").ok()?;
            let get_fn = *get_fn_sym;

            // Start metrics on all adapters
            for i in 0..adapter_count {
                let _ = start_fn(context, i, 0);
            }

            Some(AdlFpsContext {
                _lib: lib,
                context,
                adapter_count,
                get_fn,
            })
        }
    }

    pub fn get_fps(&mut self) -> f64 {
        // 1. Check AMD ADL FrameMetrics (Real 3D Game / App FPS)
        if let Some(adl) = &self.adl {
            let mut highest_game_fps = 0.0f32;
            unsafe {
                for i in 0..adl.adapter_count {
                    let mut fps_val = 0.0f32;
                    if (adl.get_fn)(adl.context, i, 0, &mut fps_val) == 0 {
                        if fps_val > highest_game_fps {
                            highest_game_fps = fps_val;
                        }
                    }
                }
            }
            if highest_game_fps > 0.0 {
                return highest_game_fps as f64;
            }
        }

        // 2. Fallback to Display / Desktop Refresh Rate
        Self::get_display_refresh_rate()
    }

    fn get_display_refresh_rate() -> f64 {
        unsafe {
            // Try DWM timing info first
            let mut info: DWM_TIMING_INFO = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<DWM_TIMING_INFO>() as u32;
            if DwmGetCompositionTimingInfo(None, &mut info).is_ok() {
                let num = info.rateRefresh.uiNumerator;
                let den = info.rateRefresh.uiDenominator;
                if den > 0 {
                    let rate = num as f64 / den as f64;
                    if rate > 0.0 {
                        return rate;
                    }
                }
            }

            // Fallback to EnumDisplaySettingsW
            let mut devmode: DEVMODEW = std::mem::zeroed();
            devmode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
            if EnumDisplaySettingsW(None, ENUM_CURRENT_SETTINGS, &mut devmode).as_bool() {
                if devmode.dmDisplayFrequency > 0 {
                    return devmode.dmDisplayFrequency as f64;
                }
            }
        }

        60.0
    }
}
