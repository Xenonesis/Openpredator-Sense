//! Win32 utility functions for Sticky Keys and low-level Windows Key locking.
#![allow(non_snake_case)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::mem::size_of;
use std::ffi::c_void;
use std::thread;
use std::process::Command;

pub static WIN_KEY_LOCKED: AtomicBool = AtomicBool::new(false);
static HOOK_INITIALIZED: AtomicBool = AtomicBool::new(false);

const SPI_GETSTICKYKEYS: u32 = 0x003A;
const SPI_SETSTICKYKEYS: u32 = 0x003B;
const SKF_STICKYKEYSON: u32 = 0x00000001;
const SPIF_UPDATEINIFILE: u32 = 0x01;
const SPIF_SENDCHANGE: u32 = 0x02;

const WH_KEYBOARD_LL: i32 = 13;
const WM_KEYDOWN: usize = 0x0100;
const WM_KEYUP: usize = 0x0101;
const WM_SYSKEYDOWN: usize = 0x0104;
const WM_SYSKEYUP: usize = 0x0105;
const VK_LWIN: u32 = 0x5B;
const VK_RWIN: u32 = 0x5C;
const VK_CONTROL: i32 = 0x11;
const VK_MENU: i32 = 0x12;
const VK_1: u32 = 0x31;
const VK_2: u32 = 0x32;
const VK_3: u32 = 0x33;
#[repr(C)]
struct STICKYKEYS {
    cbSize: u32,
    dwFlags: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct KBDLLHOOKSTRUCT {
    vkCode: u32,
    scanCode: u32,
    flags: u32,
    time: u32,
    dwExtraInfo: usize,
}

type HHOOK = *mut c_void;
type HOOKPROC = unsafe extern "system" fn(code: i32, wParam: usize, lParam: isize) -> isize;

#[link(name = "user32")]
unsafe extern "system" {
    fn SystemParametersInfoW(uiAction: u32, uiParam: u32, pvParam: *mut c_void, fWinIni: u32) -> i32;
    fn SetWindowsHookExW(idHook: i32, lpfn: HOOKPROC, hmod: *mut c_void, dwThreadId: u32) -> HHOOK;
    fn UnhookWindowsHookEx(hhk: HHOOK) -> i32;
    fn CallNextHookEx(hhk: HHOOK, nCode: i32, wParam: usize, lParam: isize) -> isize;
    fn GetMessageW(lpMsg: *mut c_void, hWnd: *mut c_void, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
    fn GetAsyncKeyState(vKey: i32) -> i16;
}
unsafe extern "system" fn keyboard_hook_proc(code: i32, wParam: usize, lParam: isize) -> isize {
    if code >= 0 {
        let is_down = wParam == WM_KEYDOWN || wParam == WM_SYSKEYDOWN;
        let is_up = wParam == WM_KEYUP || wParam == WM_SYSKEYUP;

        if is_down || is_up {
            let kbd = unsafe { *(lParam as *const KBDLLHOOKSTRUCT) };

            // Windows Key Lock
            if WIN_KEY_LOCKED.load(Ordering::Relaxed) && (kbd.vkCode == VK_LWIN || kbd.vkCode == VK_RWIN) {
                return 1;
            }

            // Global Hotkey Macro Triggers (Ctrl + Alt + 1 / 2 / 3)
            if is_down {
                let ctrl = (unsafe { GetAsyncKeyState(VK_CONTROL) } as u16 & 0x8000) != 0;
                let alt = (unsafe { GetAsyncKeyState(VK_MENU) } as u16 & 0x8000) != 0;
                if ctrl && alt {
                    match kbd.vkCode {
                        VK_1 => {
                            let _ = crate::macro_engine::play_macro("group_1".into());
                        }
                        VK_2 => {
                            let _ = crate::macro_engine::play_macro("group_2".into());
                        }
                        VK_3 => {
                            let _ = crate::macro_engine::play_macro("group_3".into());
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    unsafe { CallNextHookEx(std::ptr::null_mut(), code, wParam, lParam) }
}

/// Spawns a background message loop thread to run the low-level keyboard hook.
pub fn ensure_keyboard_hook() {
    if HOOK_INITIALIZED.swap(true, Ordering::SeqCst) {
        return;
    }

    thread::spawn(|| {
        unsafe {
            let hook = SetWindowsHookExW(WH_KEYBOARD_LL, keyboard_hook_proc, std::ptr::null_mut(), 0);
            if hook.is_null() {
                eprintln!("[win32_util] Failed to set low-level keyboard hook");
                return;
            }

            // Standard Win32 message pump for hook thread
            let mut msg: [u8; 48] = [0; 48];
            while GetMessageW(msg.as_mut_ptr() as *mut c_void, std::ptr::null_mut(), 0, 0) > 0 {}

            UnhookWindowsHookEx(hook);
        }
    });
}

pub fn set_win_key_lock(locked: bool) {
    ensure_keyboard_hook();
    WIN_KEY_LOCKED.store(locked, Ordering::SeqCst);
}

pub fn get_win_key_lock() -> bool {
    WIN_KEY_LOCKED.load(Ordering::SeqCst)
}

pub fn get_sticky_keys() -> bool {
    let mut sk = STICKYKEYS {
        cbSize: size_of::<STICKYKEYS>() as u32,
        dwFlags: 0,
    };
    unsafe {
        if SystemParametersInfoW(SPI_GETSTICKYKEYS, size_of::<STICKYKEYS>() as u32, &mut sk as *mut _ as *mut c_void, 0) != 0 {
            return (sk.dwFlags & SKF_STICKYKEYSON) != 0;
        }
    }
    false
}

pub fn set_sticky_keys(enabled: bool) {
    let mut sk = STICKYKEYS {
        cbSize: size_of::<STICKYKEYS>() as u32,
        dwFlags: 0,
    };
    unsafe {
        if SystemParametersInfoW(SPI_GETSTICKYKEYS, size_of::<STICKYKEYS>() as u32, &mut sk as *mut _ as *mut c_void, 0) != 0 {
            if enabled {
                sk.dwFlags |= SKF_STICKYKEYSON;
            } else {
                sk.dwFlags &= !SKF_STICKYKEYSON;
            }
            SystemParametersInfoW(SPI_SETSTICKYKEYS, size_of::<STICKYKEYS>() as u32, &mut sk as *mut _ as *mut c_void, SPIF_UPDATEINIFILE | SPIF_SENDCHANGE);
        }
    }
}

pub fn reboot_system() -> Result<(), String> {
    Command::new("shutdown.exe")
        .args(["/r", "/t", "3", "/c", "PredatorSense: Switching GPU Working Mode"])
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

pub fn set_sound_mode(mode: &str) -> Result<(), String> {
    // Try writing to HKLM first, fallback to HKCU if access denied (e.g. non-admin)
    let hklm_cmd = Command::new("reg.exe")
        .args([
            "add",
            r"HKLM\SOFTWARE\OEM\PredatorSense",
            "/v",
            "SoundMode",
            "/t",
            "REG_SZ",
            "/d",
            mode,
            "/f",
        ])
        .output();

    if let Ok(out) = hklm_cmd {
        if out.status.success() {
            return Ok(());
        }
    }

    let hkcu_cmd = Command::new("reg.exe")
        .args([
            "add",
            r"HKCU\SOFTWARE\OEM\PredatorSense",
            "/v",
            "SoundMode",
            "/t",
            "REG_SZ",
            "/d",
            mode,
            "/f",
        ])
        .output()
        .map_err(|e| format!("Failed to execute reg.exe: {}", e))?;

    if hkcu_cmd.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&hkcu_cmd.stderr);
        Err(format!("Failed to set SoundMode registry key: {}", err.trim()))
    }
}

pub fn get_sound_mode() -> String {
    // Check HKLM first
    if let Ok(out) = Command::new("reg.exe")
        .args(["query", r"HKLM\SOFTWARE\OEM\PredatorSense", "/v", "SoundMode"])
        .output()
    {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.contains("SoundMode") && line.contains("REG_SZ") {
                    if let Some(val) = line.split("REG_SZ").nth(1) {
                        let trimmed = val.trim();
                        if !trimmed.is_empty() {
                            return trimmed.to_string();
                        }
                    }
                }
            }
        }
    }

    // Check HKCU fallback
    if let Ok(out) = Command::new("reg.exe")
        .args(["query", r"HKCU\SOFTWARE\OEM\PredatorSense", "/v", "SoundMode"])
        .output()
    {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.contains("SoundMode") && line.contains("REG_SZ") {
                    if let Some(val) = line.split("REG_SZ").nth(1) {
                        let trimmed = val.trim();
                        if !trimmed.is_empty() {
                            return trimmed.to_string();
                        }
                    }
                }
            }
        }
    }

    "AUTOMATIC".to_string()
}
