//! Raw C ABI bindings to the pinned psyche314/Dobby source.
//!
//! This crate builds Dobby and exposes its C API; it does not manage hook
//! ownership, suspend threads, validate function signatures, or make patching
//! atomic. Callers must uphold the native API's lifetime and quiescence rules.
//!
//! Instrument callbacks receive (address, context). They must not unwind or
//! retain the context pointer. Only read initialized registers, not the dummy
//! slots; on AArch64 q8–q31 require the full-floating-point-register-pack feature.
//! Keep the context as a raw pointer rather than copying the entire structure.
//!
//! Status 0 means success, -1 means failure before applying an entry patch,
//! and -2 means a post-write protection/cache failure. Keep execution stopped
//! after -2 and resolve the native error before resuming.
//!
//! The unimplemented DobbyImportTableReplace declaration is intentionally absent.

#![no_std]
#![allow(non_camel_case_types, non_snake_case)]

use core::ffi::{c_char, c_int, c_void};

#[cfg(target_arch = "aarch64")]
mod aarch64;
#[cfg(target_arch = "aarch64")]
pub use aarch64::*;

#[cfg(target_arch = "arm")]
mod arm;
#[cfg(target_arch = "arm")]
pub use arm::*;

#[cfg(target_arch = "x86")]
mod x86;
#[cfg(target_arch = "x86")]
pub use x86::*;

#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use x86_64::*;

pub type addr_t = usize;
pub type addr32_t = u32;
pub type addr64_t = u64;
pub type asm_func_t = *mut c_void;

pub type dobby_instrument_callback_t =
    Option<unsafe extern "C" fn(address: *mut c_void, context: *mut DobbyRegisterContext)>;
pub type dobby_alloc_near_code_callback_t =
    Option<unsafe extern "C" fn(size: u32, position: addr_t, range: usize) -> addr_t>;

unsafe extern "C" {
    /// Generate an original trampoline without changing the function entry.
    pub fn DobbyPrepare(
        address: *mut c_void,
        replacement: *mut c_void,
        original: *mut *mut c_void,
    ) -> c_int;
    /// Activate a prepared or disabled hook.
    pub fn DobbyCommit(address: *mut c_void) -> c_int;
    /// Restore the entry while retaining the trampoline.
    pub fn DobbyDisable(address: *mut c_void) -> c_int;
    /// Reapply an existing hook without rebuilding the trampoline.
    pub fn DobbyEnable(address: *mut c_void) -> c_int;
    /// Prepare and activate a hook. The original output may be null.
    pub fn DobbyHook(
        address: *mut c_void,
        replacement: *mut c_void,
        original: *mut *mut c_void,
    ) -> c_int;
    /// Restore an active hook or cancel a prepared one, then remove its metadata.
    pub fn DobbyDestroy(address: *mut c_void) -> c_int;
    /// Install an instruction callback; its arguments are (address, context).
    pub fn DobbyInstrument(address: *mut c_void, callback: dobby_instrument_callback_t) -> c_int;
    /// Patch bytes across pages and restore the observed page permissions.
    pub fn DobbyCodePatch(address: *mut c_void, buffer: *mut u8, size: u32) -> c_int;
    /// Resolve a symbol from a loaded image. Names must be NUL-terminated.
    pub fn DobbySymbolResolver(image: *const c_char, symbol: *const c_char) -> *mut c_void;
    /// Return a borrowed, process-lifetime NUL-terminated native version string.
    pub fn DobbyGetVersion() -> *const c_char;

    pub fn dobby_set_near_trampoline(enable: bool);
    pub fn dobby_register_alloc_near_code_callback(callback: dobby_alloc_near_code_callback_t);
    pub fn dobby_set_options(
        enable_near_trampoline: bool,
        callback: dobby_alloc_near_code_callback_t,
    );
}
