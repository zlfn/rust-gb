// gbdk/emu_debug.h bindings — emulator debug functions

use core::ffi::c_char;

unsafe extern "C" {
    pub fn EMU_printf(format: *const c_char, ...);
}

/// Trigger a BGB/Emulicious breakpoint.
#[macro_export]
macro_rules! emu_breakpoint {
    () => {
        unsafe { core::arch::asm!("ld b, b") }
    };
}
