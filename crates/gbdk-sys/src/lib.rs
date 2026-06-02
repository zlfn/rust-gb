#![no_std]
#![feature(abi_sdcccall_0)]
#![feature(asm_experimental_arch)]
#![allow(unsafe_op_in_unsafe_fn)]

// asm/ headers
pub mod types;

// gb/ headers
pub mod gb;

// gbdk/ headers
pub mod gbdk;

// C standard library headers
pub mod stdio;
pub mod stdlib;
pub mod string;
pub mod ctype;
pub mod rand;
pub mod time;
pub mod setjmp;

unsafe extern "C" {
    /// Initialize GBDK runtime (display, palettes, OAM DMA, font, VBlank handler).
    /// Must be called before using any GBDK functions.
    fn _gbdk_init();
}

/// Initialize the GBDK runtime.
///
/// # Safety
/// Must be called exactly once, at the start of main, before any GBDK calls.
pub unsafe fn init() {
    unsafe { _gbdk_init() }
}
