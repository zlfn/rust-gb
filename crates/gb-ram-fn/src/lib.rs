//! Functions whose machine code can be copied into RAM (or HRAM) and run there.
//!
//! [`ram_fn`] defines a function and a handle implementing [`RamFn`]. The handle
//! places the function between two name-sorted markers, so its exact length is
//! known; [`RamFn::install`] copies the bytes into a fixed-size RAM buffer and
//! returns a callable pointer.
//!
//! The function must be **position independent**: it may reference statics and
//! other functions by absolute address (those do not move), but it must not
//! branch to its own code by an absolute target. Small functions qualify (the
//! SM83 backend uses relative `jr` for short branches); a long function whose
//! branches become absolute `jp` does not, and copying it would run the wrong
//! code; `cargo-gb` rejects such a function at build time (see below).
//! Parameters and return values are fine: they travel in registers and on the
//! stack, which copying does not affect.
//!
//! # Examples
//!
//! Define a function, run the ROM copy in place, then copy it into a RAM buffer
//! and run that. Bring [`RamFn`] into scope for the handle's methods.
//!
//! ```ignore
//! use gb_ram_fn::{RamFn, ram_fn};
//!
//! static mut COUNTER: u16 = 0;
//!
//! #[ram_fn(max = 16)]
//! fn inc() {
//!     unsafe {
//!         let c = core::ptr::read_volatile(&raw const COUNTER);
//!         core::ptr::write_volatile(&raw mut COUNTER, c.wrapping_add(1));
//!     }
//! }
//!
//! static mut BUF: [u8; 16] = [0; 16];
//!
//! fn demo() {
//!     inc.rom()();                                          // run in place, in ROM
//!     let ram_inc = unsafe { inc.install(&raw mut BUF) };
//!     ram_inc();                                            // run the RAM copy
//! }
//! ```
//!
//! Parameters and return values work; the installed pointer keeps the signature.
//! The buffer must be at least `max` bytes, checked at compile time:
//!
//! ```ignore
//! use gb_ram_fn::{RamFn, ram_fn};
//!
//! #[ram_fn(max = 8)]
//! fn add(a: u8, b: u8) -> u8 {
//!     a.wrapping_add(b)
//! }
//!
//! static mut BUF: [u8; 8] = [0; 8];
//!
//! fn demo() -> u8 {
//!     let added = unsafe { add.install(&raw mut BUF) };
//!     added(2, 3) // 5, computed from the RAM copy
//! }
//! ```
//!
//! # Build-time verification
//!
//! Two things keep [`install`](RamFn::install) safe: the function fits its
//! declared `max` (a compile-time-sized buffer then always holds it), and it is
//! position independent (the copy runs correctly at its new address). The
//! compiler cannot confirm either, so `cargo-gb` checks them over the linked ROM:
//! `END - run <= max`, and that the code holds no absolute self-references.
//!
//! These are therefore guarantees of a `cargo-gb` build, not of `#[ram_fn]`
//! itself. A build path that skips those checks does not provide them: `install`
//! may overflow its buffer, or copy code that breaks when run relocated.

#![no_std]

pub use gb_ram_fn_macros::ram_fn;

/// Shared interface for functions defined with [`ram_fn`].
///
/// Each `ram_fn` produces a zero-sized handle that implements this trait; the
/// associated [`Fn`](RamFn::Fn) type carries the function's own signature.
pub trait RamFn {
    /// Function-pointer type carrying the defined function's signature.
    type Fn;

    /// The declared maximum compiled size, in bytes (from `#[ram_fn(max = N)]`).
    const MAX: usize;

    /// Address of the function's machine code in ROM.
    fn src(&self) -> *const u8;

    /// Length of the machine code, in bytes.
    fn len(&self) -> usize;

    /// The ROM-resident copy as a callable pointer.
    fn rom(&self) -> Self::Fn;

    /// Copy the code into `dst` and return the RAM copy as a callable pointer.
    ///
    /// `dst` is a fixed-size buffer; `N >= MAX` is checked at compile time, so the
    /// buffer is large enough for any function within its declared `max`. No
    /// runtime length check is needed.
    ///
    /// # Safety
    ///
    /// `dst` must point to executable RAM that stays live and unchanged for as
    /// long as the returned pointer is called.
    ///
    /// `install` also assumes the function fits `MAX` and is position independent.
    /// Both are verified only by `cargo-gb` over the linked ROM (see the crate
    /// docs); built another way, `install` may overflow `dst` or return a pointer
    /// to code that does not run correctly when relocated.
    unsafe fn install<const N: usize>(&self, dst: *mut [u8; N]) -> Self::Fn;
}
