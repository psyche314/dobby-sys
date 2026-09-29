//! Native register layout for this architecture.

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct GeneralRegisters {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    pub ebp: u32,
    pub esp: u32,
    pub edi: u32,
    pub esi: u32,
}

/// A borrowed native callback frame; reserved slots are not initialized.
/// Read individual registers through the callback's raw pointer.
#[repr(C)]
pub struct DobbyRegisterContext {
    pub dummy_0: u32,
    pub esp: u32,
    pub dummy_1: u32,
    pub flags: u32,
    pub general: GeneralRegisters,
}
