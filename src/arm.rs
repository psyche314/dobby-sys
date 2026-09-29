//! Native register layout for this architecture.

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GeneralRegisters {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    pub r8: u32,
    pub r9: u32,
    pub r10: u32,
    pub r11: u32,
    pub r12: u32,
}

/// A borrowed native callback frame; reserved slots are not initialized.
/// Read individual registers through the callback's raw pointer.
#[repr(C)]
pub struct DobbyRegisterContext {
    pub dummy_0: u32,
    pub dummy_1: u32,
    pub dummy_2: u32,
    pub sp: u32,
    pub general: GeneralRegisters,
    pub lr: u32,
}
