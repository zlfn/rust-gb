#![no_std]
#![cfg_attr(target_arch = "sm83", feature(asm_experimental_arch))]

//! Typed handles to the Game Boy's High RAM (HRAM, `0xFF80..=0xFFFE`).
//!
//! HRAM is a 127-byte region the CPU can still reach while an OAM DMA holds the
//! bus, and the `ldh` instructions address it in a single byte. An [`Hram<T>`] is
//! the HRAM analogue of an MMIO `VolAddress`: a typed handle whose *address* is
//! fixed (assigned by the linker) but whose *contents* are mutable through a
//! shared reference.
//!
//! Place a cell by emitting it into a `_HRAM.*` input section; the final link
//! must map `*(_HRAM*)` into the HRAM region (provided by `gb-rt`'s linker
//! script). Being `NOLOAD`, a cell is uninitialised at reset, so write it once at
//! runtime before reading.

use core::cell::UnsafeCell;
#[cfg(target_arch = "sm83")]
use core::mem::MaybeUninit;

/// A typed cell living in High RAM.
///
/// Wraps [`UnsafeCell`] so the compiler never assumes the contents are constant
/// (an interrupt or a DMA may change them), exactly as MMIO relies on volatile
/// access. Place one with `#[link_section = "_HRAM.<name>"]`; the linker then
/// assigns its address and it is read/written with the `ldh` high-page instructions.
#[repr(transparent)]
pub struct Hram<T>(UnsafeCell<T>);

// The Game Boy is single-core, so there is no cross-thread aliasing. Sharing a
// cell with an interrupt handler is the caller's concern (guard it with a
// critical section or atomics), just like any MMIO handle.
unsafe impl<T> Sync for Hram<T> {}

impl<T> Hram<T> {
    /// Create a cell. The value is only a placeholder: HRAM is `NOLOAD`, so
    /// nothing is written at load time and the cell holds undefined bytes until it
    /// is set at runtime.
    ///
    /// # Safety
    ///
    /// The cell must come to rest at an address in `0xFF80..=0xFFFE` (e.g. via
    /// `#[link_section = "_HRAM.*"]`, as the `hram!` macro does). [`get`](Self::get)
    /// and [`set`](Self::set) address it with only the low byte and an implicit
    /// `0xFF` high byte, so a cell placed anywhere else would read and write a
    /// different High RAM location.
    pub const unsafe fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }

    /// The cell's address. Use it for raw/volatile access of wider `T`, or to feed
    /// the address to an `ldh`-based routine.
    pub const fn as_ptr(&self) -> *mut T {
        self.0.get()
    }
}

/// The largest `T` an [`Hram`] access unrolls into straight-line `ldh`s. HRAM is
/// only 127 bytes, so cells are small; a wider `T` is a compile error.
pub const MAX_BYTES: usize = 8;

// Emit one guarded `ldh` per fixed byte index. Each `if` compares a *literal* index
// against the constant `size_of::<T>()`, so the compiler keeps the first `size_of`
// blocks and deletes the rest: the result is exactly `size_of::<T>()` straight-line
// `ldh`s, with no loop and no `generic_const_exprs`.
#[cfg(target_arch = "sm83")]
macro_rules! unroll_load {
    ($base:ident, $dst:ident, $($i:literal),*) => {$(
        if $i < size_of::<T>() {
            let byte: u8;
            core::arch::asm!(
                "ldh a, (c)",
                out("a") byte,
                in("c") $base.wrapping_add($i as u8),
                options(nostack, preserves_flags),
            );
            $dst.add($i).write(byte);
        }
    )*};
}

#[cfg(target_arch = "sm83")]
macro_rules! unroll_store {
    ($base:ident, $src:ident, $($i:literal),*) => {$(
        if $i < size_of::<T>() {
            core::arch::asm!(
                "ldh (c), a",
                in("c") $base.wrapping_add($i as u8),
                in("a") $src.add($i).read(),
                options(nostack, preserves_flags),
            );
        }
    )*};
}

impl<T: Copy> Hram<T> {
    /// Read the value with one `ldh a, (c)` per byte (the cell's low address byte in
    /// `c`, HRAM's fixed `0xFF` high byte implied). Fully unrolled: a one-byte `T` is
    /// a single `ldh`, wider `T` a straight-line run, never a loop.
    ///
    /// Sound only once the cell has been [`set`](Self::set) (or for `T` whose every
    /// bit pattern is valid): an unwritten `NOLOAD` cell holds undefined bytes.
    #[inline]
    pub fn get(&self) -> T {
        const { assert!(size_of::<T>() <= MAX_BYTES, "Hram<T>: T is larger than MAX_BYTES") };
        #[cfg(target_arch = "sm83")]
        unsafe {
            let base = self.as_ptr() as usize as u8;
            let mut out = MaybeUninit::<T>::uninit();
            let dst = out.as_mut_ptr().cast::<u8>();
            unroll_load!(base, dst, 0, 1, 2, 3, 4, 5, 6, 7);
            out.assume_init()
        }
        #[cfg(not(target_arch = "sm83"))]
        unsafe {
            self.as_ptr().read_volatile()
        }
    }

    /// Write the value with one `ldh (c), a` per byte, fully unrolled.
    #[inline]
    pub fn set(&self, value: T) {
        const { assert!(size_of::<T>() <= MAX_BYTES, "Hram<T>: T is larger than MAX_BYTES") };
        #[cfg(target_arch = "sm83")]
        unsafe {
            let base = self.as_ptr() as usize as u8;
            let src = (&value as *const T).cast::<u8>();
            unroll_store!(base, src, 0, 1, 2, 3, 4, 5, 6, 7);
        }
        #[cfg(not(target_arch = "sm83"))]
        unsafe {
            self.as_ptr().write_volatile(value);
        }
    }
}
