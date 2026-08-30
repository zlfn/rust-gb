#![no_std]
#![feature(asm_experimental_arch)]

//! Core Game Boy (SM83) runtime.
//!
//! Depending on this crate links the startup code in `rrt0.s` (reset entry, RST
//! and interrupt vectors) and makes the linker script `gb.ld` available to the
//! ROM build pipeline. Each interrupt vector jumps to an `_on_*` handler that
//! defaults to a no-op and is overridden by defining a strong symbol.

// The startup is assembled by the compiler itself (no host assembler is invoked),
// landing in this crate's object. Nothing references `_reset` from Rust, so the
// startup would be dropped when the staticlib is built; the ROM pipeline instead
// links this crate's rlib directly, where the linker's ENTRY(_reset) pulls it in.
core::arch::global_asm!(include_str!("rrt0.s"));

pub mod boot;
pub mod builtin;

/// Proof that interrupts are disabled, re-exported from `critical-section`.
///
/// A handler marked with [`macro@interrupt`] can take one as a parameter.
pub use critical_section::CriticalSection;

/// Attribute marking the program entry point. See [`macro@entry`].
pub use gb_rt_macros::entry;

/// Attribute installing an interrupt handler at its vector. See [`macro@interrupt`].
pub use gb_rt_macros::interrupt;

/// Whether this program was built for CGB double speed mode.
///
/// The `cgb-double-speed` feature sets it, and the startup switches before
/// [`entry`](macro@crate::entry) hands over. Reading it is how the rest of the
/// crates work out what a clock counts at.
pub const DOUBLE_SPEED: bool = cfg!(feature = "cgb-double-speed");

/// Switch into CGB double speed mode, once, before the program runs.
///
/// [`entry`](macro@crate::entry) puts a call at the top of the program, so
/// there is nothing to remember and nowhere else this belongs.
///
/// # Safety
///
/// Executes `stop`. That is only sound at the very start, before interrupts are
/// on and before anything has been drawn: the CPU pauses for 2050 M-cycles with
/// video memory locking frozen, which shows as a black or object-less frame.
#[doc(hidden)]
#[inline]
pub unsafe fn __enter_double_speed() {
    // A cartridge built for this is a Game Boy Color one, but an original Game
    // Boy will still run it, and there `stop` is a machine that never wakes.
    if !DOUBLE_SPEED || crate::boot::a() != 0x11 {
        return;
    }
    unsafe {
        core::arch::asm!(
            // `stop` wakes on a joypad line falling, so the lines are taken out
            // of the picture first: no interrupt may fire, and no row selected.
            "xor a",
            "ldh ($ff), a",     // IE
            "ld a, $30",
            "ldh ($00), a",     // JOYP, both rows off
            "ld a, $01",
            "ldh ($4d), a",     // KEY1, switch armed
            // `stop`. The assembler has no mnemonic for it, and the byte after
            // is the one the CPU skips.
            ".byte 0x10, 0x00",
            out("a") _,
            options(nostack),
        );
    }
}
