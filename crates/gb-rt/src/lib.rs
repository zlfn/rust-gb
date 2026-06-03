#![no_std]
#![feature(asm_experimental_arch)]

//! Core Game Boy (SM83) runtime.
//!
//! Depending on this crate links the startup code in `rrt0.s` (reset entry, RST
//! and interrupt vectors with weak `_on_*` handlers) and makes the linker script
//! `gb.ld` available to the ROM build pipeline. It exposes no Rust API of its own.

// The startup is assembled by the compiler itself (no host assembler is invoked),
// landing in this crate's object. Nothing references `_reset` from Rust, so the
// startup would be dropped when the staticlib is built; the ROM pipeline instead
// links this crate's rlib directly, where the linker's ENTRY(_reset) pulls it in.
core::arch::global_asm!(include_str!("rrt0.s"));

/// Attribute marking the program entry point. See [`macro@entry`].
pub use gb_rt_macros::entry;
