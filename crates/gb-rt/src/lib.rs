#![no_std]
#![feature(asm_experimental_arch)]

//! Core Game Boy (SM83) runtime.
//!
//! Depending on this crate links the startup code in `rrt0.s` (reset entry, RST
//! and interrupt vectors) and makes the linker script `gb.ld` available to the
//! ROM build pipeline. Each interrupt vector jumps to an `_on_*` handler that
//! defaults to a no-op and is overridden by defining a strong symbol; the
//! [`entry`] and [`interrupt`] attribute macros and the [`boot`] and [`builtin`]
//! modules are its Rust API.

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
