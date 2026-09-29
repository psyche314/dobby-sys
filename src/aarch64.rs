//! Native register layout for this architecture.

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GeneralRegisters {
    pub x0: u64,
    pub x1: u64,
    pub x2: u64,
    pub x3: u64,
    pub x4: u64,
    pub x5: u64,
    pub x6: u64,
    pub x7: u64,
    pub x8: u64,
    pub x9: u64,
    pub x10: u64,
    pub x11: u64,
    pub x12: u64,
    pub x13: u64,
    pub x14: u64,
    pub x15: u64,
    pub x16: u64,
    pub x17: u64,
    pub x18: u64,
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
}

/// One 128-bit SIMD/floating-point register, viewed as bits, doubles or floats.
#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub union FPReg {
    pub q: i128,
    pub d: [f64; 2],
    pub f: [f32; 4],
}

pub const ARM64_TMP_REG_NDX_0: usize = 17;

/// A borrowed native callback frame; reserved slots are not initialized.
/// Read individual registers through the callback's raw pointer.
#[repr(C)]
pub struct DobbyRegisterContext {
    pub dummy_0: u64,
    pub sp: u64,
    pub dummy_1: u64,
    pub general: GeneralRegisters,
    pub fp: u64,
    pub lr: u64,
    pub floating: [FPReg; 32],
}
