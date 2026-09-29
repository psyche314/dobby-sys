use core::{
    ffi::{c_char, c_void},
    mem::{align_of, offset_of, size_of},
    ptr,
};
use dobby_sys::*;
use std::{
    ffi::CStr,
    sync::atomic::{AtomicI32, AtomicPtr, AtomicUsize, Ordering},
};

unsafe extern "C" {
    fn fixture_target(value: i32) -> i32;
    fn fixture_layout(index: u32) -> usize;
    fn fixture_log(message: *const c_char);
}

type Target = unsafe extern "C" fn(i32) -> i32;
static ORIGINAL: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static CALLBACK_ADDRESS: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static CALLBACK_COUNT: AtomicUsize = AtomicUsize::new(0);
static CALLBACK_ARGUMENT: AtomicI32 = AtomicI32::new(0);

unsafe extern "C" fn replacement(value: i32) -> i32 {
    let original: Target = unsafe { core::mem::transmute(ORIGINAL.load(Ordering::Acquire)) };
    unsafe { original(value) + 100 }
}
unsafe extern "C" fn constant_replacement(_: i32) -> i32 {
    91
}

unsafe extern "C" fn instrument(address: *mut c_void, context: *mut DobbyRegisterContext) {
    CALLBACK_ADDRESS.store(address, Ordering::Relaxed);
    CALLBACK_COUNT.fetch_add(1, Ordering::Relaxed);
    #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
    let argument = unsafe { ptr::addr_of_mut!((*context).general.rcx) };
    #[cfg(all(target_arch = "x86_64", not(target_os = "windows")))]
    let argument = unsafe { ptr::addr_of_mut!((*context).general.rdi) };
    #[cfg(target_arch = "aarch64")]
    let argument = unsafe { ptr::addr_of_mut!((*context).general.x0) };
    #[cfg(target_arch = "arm")]
    let argument = unsafe { ptr::addr_of_mut!((*context).general.r0) };
    #[cfg(target_arch = "x86")]
    let argument = unsafe { ((*context).esp as usize + 4) as *mut u32 };
    let value = unsafe { argument.read() } as i32;
    CALLBACK_ARGUMENT.store(value, Ordering::Relaxed);
    unsafe { argument.write((value + 10) as _) };
}

fn call(value: i32) -> i32 {
    let address = std::hint::black_box(fixture_target as *const ());
    let function: Target = unsafe { core::mem::transmute(address) };
    unsafe { function(value) }
}

fn log(message: &CStr) {
    unsafe { fixture_log(message.as_ptr()) }
}

pub fn run() {
    log(c"Rust dobby-sys E2E starting");
    unsafe {
        assert_eq!(fixture_layout(0), size_of::<DobbyRegisterContext>());
        assert_eq!(fixture_layout(1), align_of::<DobbyRegisterContext>());
        assert_eq!(fixture_layout(2), offset_of!(DobbyRegisterContext, general));
        #[cfg(target_arch = "x86_64")]
        {
            assert_eq!(
                fixture_layout(3),
                offset_of!(DobbyRegisterContext, general.rax)
            );
            assert_eq!(
                fixture_layout(4),
                offset_of!(DobbyRegisterContext, general.rsp)
            );
        }
        #[cfg(target_arch = "x86")]
        {
            assert_eq!(
                fixture_layout(3),
                offset_of!(DobbyRegisterContext, general.eax)
            );
            assert_eq!(fixture_layout(4), offset_of!(DobbyRegisterContext, esp));
        }
        #[cfg(target_arch = "aarch64")]
        {
            assert_eq!(
                fixture_layout(3),
                offset_of!(DobbyRegisterContext, general.x0)
            );
            assert_eq!(fixture_layout(4), offset_of!(DobbyRegisterContext, sp));
            assert_eq!(
                fixture_layout(5),
                offset_of!(DobbyRegisterContext, floating)
            );
        }
        #[cfg(target_arch = "arm")]
        {
            assert_eq!(
                fixture_layout(3),
                offset_of!(DobbyRegisterContext, general.r0)
            );
            assert_eq!(fixture_layout(4), offset_of!(DobbyRegisterContext, sp));
        }
        assert_eq!(CStr::from_ptr(DobbyGetVersion()).to_bytes(), b"1.0.0");
    }
    log(c"PASS C header and Rust register ABI");

    let target = fixture_target as *const () as *mut c_void;
    let mut original = ptr::null_mut();
    assert_eq!(call(5), 12);
    unsafe {
        assert_eq!(
            DobbyPrepare(
                target,
                replacement as *const () as *mut c_void,
                &mut original
            ),
            0
        );
        assert!(!original.is_null());
        ORIGINAL.store(original, Ordering::Release);
        assert_eq!(call(5), 12);
        assert_ne!(
            DobbyPrepare(
                target,
                replacement as *const () as *mut c_void,
                ptr::null_mut()
            ),
            0
        );
        assert_eq!(DobbyCommit(target), 0);
        assert_eq!(call(5), 112);
        for _ in 0..20 {
            assert_eq!(DobbyDisable(target), 0);
            assert_eq!(DobbyDisable(target), 0);
            assert_eq!(call(5), 12);
            let function: Target = core::mem::transmute(original);
            assert_eq!(function(5), 12);
            assert_eq!(DobbyEnable(target), 0);
            assert_eq!(DobbyEnable(target), 0);
            assert_eq!(call(5), 112);
        }
        assert_eq!(DobbyDestroy(target), 0);
        assert_eq!(call(5), 12);
        assert_eq!(
            DobbyPrepare(
                target,
                replacement as *const () as *mut c_void,
                &mut original
            ),
            0
        );
        assert_eq!(DobbyDestroy(target), 0);
        assert_eq!(
            DobbyHook(
                target,
                constant_replacement as *const () as *mut c_void,
                ptr::null_mut()
            ),
            0
        );
        assert_eq!(call(5), 91);
        assert_eq!(DobbyDestroy(target), 0);
    }
    ORIGINAL.store(ptr::null_mut(), Ordering::Release);
    log(c"PASS Rust replacement, original trampoline, prepare/commit and enable/disable");

    CALLBACK_COUNT.store(0, Ordering::Relaxed);
    unsafe {
        assert_eq!(DobbyInstrument(target, Some(instrument)), 0);
        assert_eq!(call(5), 22);
        assert_eq!(CALLBACK_ADDRESS.load(Ordering::Relaxed), target);
        assert_eq!(CALLBACK_ARGUMENT.load(Ordering::Relaxed), 5);
        assert_eq!(CALLBACK_COUNT.load(Ordering::Relaxed), 1);
        assert_eq!(DobbyDisable(target), 0);
        assert_eq!(call(5), 12);
        assert_eq!(CALLBACK_COUNT.load(Ordering::Relaxed), 1);
        assert_eq!(DobbyEnable(target), 0);
        assert_eq!(call(5), 22);
        assert_eq!(CALLBACK_COUNT.load(Ordering::Relaxed), 2);
        assert_eq!(DobbyDestroy(target), 0);
        assert_eq!(call(5), 12);
        assert_ne!(DobbyDestroy(target), 0);
    }
    log(c"PASS native-to-Rust callback and register modification");

    unsafe {
        #[cfg(target_os = "windows")]
        let symbol = DobbySymbolResolver(c"kernel32.dll".as_ptr(), c"GetCurrentProcessId".as_ptr());
        #[cfg(not(target_os = "windows"))]
        let symbol = DobbySymbolResolver(ptr::null(), c"malloc".as_ptr());
        assert!(!symbol.is_null());
        dobby_set_near_trampoline(true);
        dobby_register_alloc_near_code_callback(None);
        dobby_set_options(true, None);
        let mut word = 123_u32;
        let mut bytes = 42_u32.to_ne_bytes();
        assert_eq!(
            DobbyCodePatch(ptr::addr_of_mut!(word).cast(), bytes.as_mut_ptr(), 4),
            0
        );
        assert_eq!(word, 42);
        assert_ne!(
            DobbyPrepare(ptr::null_mut(), ptr::null_mut(), ptr::null_mut()),
            0
        );
    }
    log(c"PASS all Dobby native E2E checks");
}

// Same launcher as the pinned native repository: no duplicate Android harness.
#[unsafe(no_mangle)]
pub extern "system" fn Java_org_psyche_dobby_E2EActivity_runTests(
    _env: *mut c_void,
    _class: *mut c_void,
) -> i32 {
    match std::panic::catch_unwind(run) {
        Ok(()) => 0,
        Err(_) => {
            log(c"FAIL Rust dobby-sys E2E");
            -1
        }
    }
}
