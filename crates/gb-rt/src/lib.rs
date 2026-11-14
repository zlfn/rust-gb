#![no_std]
#![feature(asm_experimental_arch)]

//! # Run-time support for Game Boy (SM83)
//!
//! This crate provides startup code that LLVM-Z80 and other crates in Rust-GB expect.
//!
//! Depending on this crate links the startup code in `rrt0.s`
//! and makes the linker script `gb.ld` available to the ROM build pipeline.

core::arch::global_asm!(include_str!("rrt0.s"));

pub mod boot;
pub mod builtin;

/// Critical section token, re-exported from `critical-section`.
///
/// A handler marked with [`macro@interrupt`] can take this as a parameter.
pub use critical_section::CriticalSection;

/// Attribute marking the program entry point. See [`macro@entry`].
pub use gb_rt_macros::entry;

/// Attribute installing an interrupt handler at its vector. See [`macro@interrupt`].
pub use gb_rt_macros::interrupt;

/// Whether this program was built for CGB double speed mode.
///
/// The `cgb-double-speed` feature sets it, and the startup switches before
/// [`entry`](macro@crate::entry) hands over.
///
/// Other crates can read this value to determine the current Game Boy's clock speed.
pub const DOUBLE_SPEED: bool = cfg!(feature = "cgb-double-speed");

/// Switch into CGB double speed mode.
///
/// This should be called once, before the program runs.
/// [`macro@entry`] runs this automatically depending on the feature flag.
#[doc(hidden)]
#[inline]
pub fn __enter_double_speed() {
    // If the code below runs on the original Game Boy, it will freeze.
    // TODO: Prevent this at compile time in cargo-gb?
    if !DOUBLE_SPEED || crate::boot::BOOT_A != 0x11 {
        return;
    }
    unsafe {
        core::arch::asm!(
            // The following procedure follows Pan Docs's speed switching process.
            // https://gbdev.io/pandocs/CGB_Registers.html#ff4d--key1spd-cgb-mode-only-prepare-speed-switch
            "xor a",
            "ldh ($ff), a",     // IE
            "ld a, $30",
            "ldh ($00), a",     // JOYP
            "ld a, $01",
            "ldh ($4d), a",     // KEY1
            // TODO: Replace this if `stop` mnemonic is supported in LLVM-Z80.
            ".byte 0x10, 0x00", // `stop`
            out("a") _,
            options(nostack),
        );
    }
}
