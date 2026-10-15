//! Lightweight dynamic loader for NVAPI (NVIDIA API) to read real GPU & VRAM clocks.

use std::ffi::{c_void, CString};
use std::mem::size_of;

type NvAPIQueryInterfaceFn = unsafe extern "C" fn(offset: u32) -> *mut c_void;
type NvAPIInitializeFn = unsafe extern "C" fn() -> i32;
type NvAPIEnumPhysicalGPUsFn = unsafe extern "C" fn(handles: *mut *mut c_void, count: *mut i32) -> i32;
type NvAPIGetFullNameFn = unsafe extern "C" fn(handle: *mut c_void, name: *mut u8) -> i32;
type NvAPIGetAllClockFrequenciesFn = unsafe extern "C" fn(handle: *mut c_void, freqs: *mut NvGpuClockFrequencies) -> i32;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NvClockDomain {
    pub is_present: u32,
    pub frequency: u32,
}

#[repr(C)]
pub struct NvGpuClockFrequencies {
    pub version: u32,
    pub clock_type: u32,
    pub domain: [NvClockDomain; 32],
}

pub struct NvApi {
    gpu_handle: *mut c_void,
    get_all_clocks: Option<NvAPIGetAllClockFrequenciesFn>,
    pub gpu_name: String,
}

unsafe impl Send for NvApi {}
unsafe impl Sync for NvApi {}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryA(lpLibFileName: *const u8) -> *mut c_void;
    fn GetProcAddress(hModule: *mut c_void, lpProcName: *const u8) -> *mut c_void;
}

impl NvApi {
    pub fn init() -> Option<Self> {
        unsafe {
            let lib_name = CString::new("nvapi64.dll").ok()?;
            let hmod = LoadLibraryA(lib_name.as_ptr() as *const u8);
            if hmod.is_null() {
                return None;
            }

            let proc_name = CString::new("nvapi_QueryInterface").ok()?;
            let query_addr = GetProcAddress(hmod, proc_name.as_ptr() as *const u8);
            if query_addr.is_null() {
                return None;
            }

            let query_fn: NvAPIQueryInterfaceFn = std::mem::transmute(query_addr);

            // NvAPI_Initialize = 0x0150E828
            let init_addr = query_fn(0x0150E828);
            if init_addr.is_null() {
                return None;
            }
            let init_fn: NvAPIInitializeFn = std::mem::transmute(init_addr);
            if init_fn() != 0 {
                return None;
            }

            // NvAPI_EnumPhysicalGPUs = 0xE5AC21F1
            let enum_addr = query_fn(0xE5AC21F1);
            if enum_addr.is_null() {
                return None;
            }
            let enum_fn: NvAPIEnumPhysicalGPUsFn = std::mem::transmute(enum_addr);
            let mut handles: [*mut c_void; 64] = [std::ptr::null_mut(); 64];
            let mut count: i32 = 0;
            if enum_fn(handles.as_mut_ptr(), &mut count) != 0 || count == 0 {
                return None;
            }

            let gpu_handle = handles[0];

            // NvAPI_GPU_GetFullName = 0xCEEE8E9F
            let mut gpu_name = "NVIDIA GeForce GPU".to_string();
            let name_addr = query_fn(0xCEEE8E9F);
            if !name_addr.is_null() {
                let name_fn: NvAPIGetFullNameFn = std::mem::transmute(name_addr);
                let mut buf = [0u8; 64];
                if name_fn(gpu_handle, buf.as_mut_ptr()) == 0 {
                    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
                    if let Ok(s) = std::str::from_utf8(&buf[..len]) {
                        if !s.trim().is_empty() {
                            gpu_name = s.trim().to_string();
                        }
                    }
                }
            }

            // NvAPI_GPU_GetAllClockFrequencies = 0xDCB616C3
            let clocks_addr = query_fn(0xDCB616C3);
            let get_all_clocks = if !clocks_addr.is_null() {
                Some(std::mem::transmute::<*mut c_void, NvAPIGetAllClockFrequenciesFn>(clocks_addr))
            } else {
                None
            };

            Some(Self {
                gpu_handle,
                get_all_clocks,
                gpu_name,
            })
        }
    }

    /// Read (Core Clock MHz, VRAM Clock MHz)
    pub fn get_clocks(&self) -> Option<(u32, u32)> {
        let get_clocks_fn = self.get_all_clocks?;
        unsafe {
            let mut freqs = NvGpuClockFrequencies {
                version: (size_of::<NvGpuClockFrequencies>() as u32) | 0x20000,
                clock_type: 0,
                domain: [NvClockDomain { is_present: 0, frequency: 0 }; 32],
            };
            if get_clocks_fn(self.gpu_handle, &mut freqs) == 0 {
                let core_mhz = freqs.domain[0].frequency / 1000;
                let vram_mhz = freqs.domain[8].frequency / 1000;
                Some((core_mhz, vram_mhz))
            } else {
                None
            }
        }
    }
}
