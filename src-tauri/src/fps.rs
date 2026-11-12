use std::ffi::c_void;
use libloading::{Library, Symbol};

#[repr(C)]
struct DevModeW {
    dm_device_name: [u16; 32],
    dm_spec_version: u16,
    dm_driver_version: u16,
    dm_size: u16,
    dm_driver_extra: u16,
    dm_fields: u32,
    dm_position_x: i32,
    dm_position_y: i32,
    dm_display_orientation: u32,
    dm_display_fixed_output: u32,
    dm_color: i16,
    dm_duplex: i16,
    dm_y_resolution: i16,
    dm_tt_option: i16,
    dm_collate: i16,
    dm_form_name: [u16; 32],
    dm_log_pixels: u16,
    dm_bits_per_pel: u32,
    dm_pels_width: u32,
    dm_pels_height: u32,
    dm_display_flags: u32,
    dm_display_frequency: u32,
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

struct AdlFpsContext {
    _lib: Library,
    context: usize,
    get_fps_fn: unsafe extern "C" fn(*mut c_void, i32, i32, *mut f32) -> i32,
}

unsafe impl Send for AdlFpsContext {}
unsafe impl Send for FpsMonitor {}

pub struct FpsMonitor {
    _user32_lib: Option<Library>,
    enum_display: Option<unsafe extern "system" fn(*const u16, u32, *mut DevModeW) -> i32>,
    adl: Option<AdlFpsContext>,
}

impl FpsMonitor {
    pub fn new() -> Self {
        let (user32_lib, enum_display) = unsafe {
            match Library::new("user32.dll") {
                Ok(lib) => {
                    let func: Option<Symbol<unsafe extern "system" fn(*const u16, u32, *mut DevModeW) -> i32>> = 
                        lib.get(b"EnumDisplaySettingsW\0").ok();
                    let f = func.map(|sym| *sym);
                    (Some(lib), f)
                }
                Err(_) => (None, None),
            }
        };

        let adl = Self::init_adl();

        Self {
            _user32_lib: user32_lib,
            enum_display,
            adl,
        }
    }

    fn init_adl() -> Option<AdlFpsContext> {
        unsafe {
            let lib = Library::new("atiadlxx.dll")
                .or_else(|_| Library::new("atiadlxy.dll"))
                .ok()?;

            let create: Symbol<unsafe extern "C" fn(unsafe extern "C" fn(i32) -> *mut c_void, i32, *mut *mut c_void) -> i32> = 
                lib.get(b"ADL2_Main_Control_Create\0").ok()?;

            let mut context: *mut c_void = std::ptr::null_mut();
            if create(adl_malloc, 1, &mut context) != 0 || context.is_null() {
                return None;
            }

            let get_fps_fn = lib.get::<unsafe extern "C" fn(*mut c_void, i32, i32, *mut f32) -> i32>(b"ADL2_Adapter_FrameMetrics_Get\0")
                .ok()
                .map(|s| *s)?;

            Some(AdlFpsContext {
                _lib: lib,
                context: context as usize,
                get_fps_fn,
            })
        }
    }

    pub fn get_fps(&mut self) -> f64 {
        // 1. Check AMD 3D Game FPS first (Tier 1)
        if let Some(adl) = &self.adl {
            let mut highest_fps = 0.0f32;
            unsafe {
                for adapter_idx in 0..4 {
                    let mut fps_val = 0.0f32;
                    if (adl.get_fps_fn)(adl.context as *mut c_void, adapter_idx, 0, &mut fps_val) == 0 {
                        if fps_val > highest_fps {
                            highest_fps = fps_val;
                        }
                    }
                }
            }
            if highest_fps > 0.0 {
                return highest_fps as f64;
            }
        }

        // 2. Fallback to Display Refresh Rate (Tier 2 - Desktop Fallback)
        if let Some(enum_fn) = self.enum_display {
            unsafe {
                let mut dm = std::mem::zeroed::<DevModeW>();
                dm.dm_size = std::mem::size_of::<DevModeW>() as u16;
                // 0xFFFFFFFF = ENUM_CURRENT_SETTINGS
                if enum_fn(std::ptr::null(), 0xFFFFFFFF, &mut dm) != 0 {
                    if dm.dm_display_frequency > 0 {
                        return dm.dm_display_frequency as f64;
                    }
                }
            }
        }

        60.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fps_monitor() {
        let mut fps = FpsMonitor::new();
        let val = fps.get_fps();
        println!("Test FPS: {}", val);
        assert!(val > 0.0);
    }
}
