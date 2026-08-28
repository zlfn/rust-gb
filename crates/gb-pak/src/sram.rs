//! The cartridge's own RAM, usually kept alive by a battery.
//!
//! SRAM is only readable and writable while the controller has it switched on, so
//! every access happens inside [`Sram::open`], which switches it on for the
//! closure and off again after. Leaving it off otherwise protects the contents
//! while the console powers down or the cartridge is pulled.
//!
//! # Declaring
//!
//! ```ignore
//! use core::cell::Cell;
//! use gb_pak::{CriticalSection, sram};
//! use zerocopy::FromBytes;
//!
//! #[repr(C)]
//! #[derive(FromBytes)]
//! struct Save {
//!     magic:   Cell<[u8; 4]>,
//!     version: Cell<u8>,
//!     hp:      Cell<u8>,
//!     gold:    Cell<u16>,
//! }
//!
//! /// The save file.
//! #[sram(0)]
//! static FILE: Save;
//!
//! fn hurt(cs: CriticalSection<'_>) {
//!     FILE.open(cs, |f| f.hp.set(f.hp.get().saturating_sub(1)));
//! }
//! ```
//!
//! A static has no initializer: the bytes are the cartridge's already. A bank
//! holds one of them, at its start (`0xA000`), and the attribute says which bank.
//!
//! Turned down at compile time: a missing bank number, a bank the cartridge does
//! not have, any bank when it has no SRAM, a value larger than one 8 KiB bank,
//! and an initializer.
//!
//! # Editing in place
//!
//! The closure is handed a shared reference, so every field is written as a
//! [`Cell`](core::cell::Cell), the way a peripheral crate writes a register
//! block. A field that is not one can never be written.
//!
//! ```ignore
//! unsafe { gb::interrupt::free(|cs| FILE.open(cs, |f| f.gold.set(f.gold.get() + 10))) };
//! ```
//!
//! # Trusting the contents
//!
//! A cartridge that has never been written holds noise, so the value is bounded
//! on [`FromBytes`]: no byte pattern may be invalid for it, which rules out
//! `bool`, `char`, enums, and references.
//!
//! `repr(C)` is recommended: a `repr(Rust)` layout is unspecified and may differ
//! between toolchain versions, so a save an earlier build wrote could be read
//! back wrong.
//!
//! That bounds the type, not the contents. The bytes are still whatever survived:
//! noise on a new cartridge, decay on a failing battery, another build's layout
//! after an update. Check a magic number, a schema version, and a checksum before
//! believing a save.

use zerocopy::FromBytes;

use crate::{CriticalSection, WINDOW, WINDOW_LEN, reg};

/// A value at the base of SRAM bank `BANK`.
///
/// Declared by [`#[sram]`](macro@crate::sram).
pub struct Sram<T: FromBytes, const BANK: u8> {
    _value: core::marker::PhantomData<T>,
}

// The handle holds no `T`; the bytes are the cartridge's and are reached only
// inside `open`.
unsafe impl<T: FromBytes, const BANK: u8> Sync for Sram<T, BANK> {}

impl<T: FromBytes, const BANK: u8> Sram<T, BANK> {
    /// # Safety
    ///
    /// The cartridge must have bank `BANK`. The attribute checks that against
    /// `header.toml`, which is why this module is reachable on a cartridge with
    /// no SRAM at all: the check belongs at the declaration, not at the module.
    #[doc(hidden)]
    pub const unsafe fn declare() -> Self {
        Sram { _value: core::marker::PhantomData }
    }

    /// Switch the RAM on, run `f`, switch it off.
    ///
    /// The [`CriticalSection`] keeps an interrupt handler from reaching the same
    /// bytes while `f` runs.
    ///
    /// The window holds one thing at a time, so `f` must not open another scope
    /// or read another peripheral: the rest of `f` would look at whatever that
    /// left selected.
    #[inline]
    pub fn open<R>(&self, _cs: CriticalSection<'_>, f: impl FnOnce(&T) -> R) -> R {
        reg::select(BANK);
        reg::enable();
        let r = f(unsafe { &*(WINDOW as *const T) });
        reg::disable();
        r
    }
}

/// Bytes one bank holds.
pub const BANK_LEN: usize = WINDOW_LEN;

/// SRAM banks this cartridge has, from `ram_size` in `header.toml`.
pub const BANKS: u8 = reg::BANKS;
