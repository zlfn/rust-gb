#![no_std]
#![cfg_attr(target_arch = "sm83", feature(asm_experimental_arch))]

//! Typed handles to the Game Boy's High RAM (HRAM, `0xFF80..=0xFFFE`).
//!
//! HRAM is a 127-byte region the CPU can still reach while an OAM DMA holds the
//! bus, and the `ldh` instructions address it in a single byte. Cells declared
//! with the [`hram!`] macro are read and written with the **immediate** `ldh (n)`
//! form (2 bytes / 3 cycles): the linker assigns each cell a fixed HRAM address
//! and the low byte is baked into the instruction. That is faster and smaller
//! than the 3-byte / 4-cycle absolute `ld (nn)` used for WRAM.
//!
//! [`Hram<T>`] is the lower-level building block: a cell addressed through a
//! runtime pointer with the register form `ldh (c)`, for the rare case of a cell
//! whose address is not known at the access site. Prefer [`hram!`].
//!
//! Access goes through the [`HramCell`] trait; bring it into scope with
//! `use gb_hram::prelude::*`.
//!
//! # Examples
//!
//! Declare cells and areas, then read and write them. The generated accessors
//! emit `ldh` inline asm, so the crate needs `#![feature(asm_experimental_arch)]`.
//!
//! ```ignore
//! #![feature(asm_experimental_arch)]
//! use gb_hram::{hram, prelude::*};
//!
//! hram! {
//!     /// Frames elapsed, bumped by the VBlank handler.
//!     pub static FRAME: u8;
//!     static SCROLL_X: u16;
//!     static OAM_DMA: [u8; 13];   // a raw HramArea<13>
//! }
//!
//! fn tick() {
//!     FRAME.set(FRAME.get().wrapping_add(1));   // ldh a,(n) / ldh (n),a
//!     SCROLL_X.set(120);
//! }
//! ```
//!
//! Pass a cell to code that is generic over any HRAM cell; each call stays in its
//! own fast form (the macro cells immediate, [`Hram<T>`] register):
//!
//! ```ignore
//! use gb_hram::prelude::*;
//!
//! fn bump<C: HramCell<Value = u8>>(cell: &C) {
//!     cell.set(cell.get().wrapping_add(1));
//! }
//! bump(&FRAME);
//! ```
//!
//! # Zero initialisation
//!
//! A cell is **zero-initialised by the runtime**, the same way `.bss` is: HRAM is
//! `NOLOAD`, so nothing is loaded from ROM, and a conforming runtime is required
//! to clear the HRAM region (`0xFF80..=0xFFFE`) before `main`. A cell whose
//! `Value` has a valid all-zero bit pattern (an integer, say) therefore starts at
//! `0` and may be read before it is first written; a cell of any other type must
//! be written before it is read.

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

/// A typed High RAM cell: read and write `Value` with the `ldh` high-page
/// instructions. Implemented for the per-cell handles the [`hram!`] macro
/// generates (immediate `ldh (n)`) and for [`Hram<T>`] (register `ldh (c)`).
pub trait HramCell {
    /// The stored type. Must be [`Copy`] and at most [`MAX_BYTES`] wide.
    type Value: Copy;

    /// Read the cell.
    ///
    /// The runtime zero-initialises HRAM (see the crate docs), so for a `Value`
    /// whose every bit pattern is valid this may be called before the first
    /// [`set`](HramCell::set) and yields `0`; a `Value` of any other type must be
    /// set first.
    fn get(&self) -> Self::Value;

    /// Write the cell.
    fn set(&self, value: Self::Value);

    /// The cell's address, for raw access or to feed an `ldh`-based routine.
    fn as_ptr(&self) -> *mut Self::Value;
}

/// Common imports for HRAM access (`use gb_hram::prelude::*`).
pub mod prelude {
    pub use crate::HramCell;
}

/// The largest `Value` an HRAM access unrolls into straight-line `ldh`s. HRAM is
/// only 127 bytes, so cells are small; a wider type is a compile error.
pub const MAX_BYTES: usize = 8;

// ===== Hram<T>: runtime-addressed cell, register form `ldh (c)` =====

/// A typed cell addressed through a runtime pointer (`ldh (c)`).
///
/// Wraps [`UnsafeCell`] so the compiler never assumes the contents are constant
/// (an interrupt or a DMA may change them). Unlike a [`hram!`] cell, its address
/// is taken at the access site, so it cannot use the immediate `ldh (n)` form;
/// use it only when a cell must be referred to by pointer.
#[repr(transparent)]
pub struct Hram<T>(UnsafeCell<MaybeUninit<T>>);

// The Game Boy is single-core, so there is no cross-thread aliasing. Sharing a
// cell with an interrupt handler is the caller's concern (guard it with a
// critical section or atomics), just like any MMIO handle.
unsafe impl<T> Sync for Hram<T> {}

impl<T> Hram<T> {
    /// Reserve a cell. HRAM is `NOLOAD`, so its contents are not loaded from ROM;
    /// the runtime is expected to zero-initialise HRAM at startup (see the crate
    /// docs), so the cell starts at zero.
    ///
    /// # Safety
    ///
    /// The cell must come to rest at an address in `0xFF80..=0xFFFE`. The [`hram!`]
    /// macro guarantees this; a hand-placed cell needs `#[link_section = "_HRAM.*"]`.
    pub const unsafe fn uninit() -> Self {
        Self(UnsafeCell::new(MaybeUninit::uninit()))
    }

    /// The cell's address as a raw pointer.
    pub const fn ptr(&self) -> *mut T {
        self.0.get() as *mut T
    }
}

// Register-form unroll: one `ldh (c)` per byte, the low address byte held in `c`.
// The asm lives here in gb-hram, so callers of `Hram<T>` need no nightly feature.
#[cfg(target_arch = "sm83")]
macro_rules! unroll_load_c {
    ($base:ident, $dst:ident, $ty:ty, $($i:literal),*) => {$(
        if $i < size_of::<$ty>() {
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
macro_rules! unroll_store_c {
    ($base:ident, $src:ident, $ty:ty, $($i:literal),*) => {$(
        if $i < size_of::<$ty>() {
            core::arch::asm!(
                "ldh (c), a",
                in("c") $base.wrapping_add($i as u8),
                in("a") $src.add($i).read(),
                options(nostack, preserves_flags),
            );
        }
    )*};
}

impl<T: Copy> HramCell for Hram<T> {
    type Value = T;

    #[inline]
    fn get(&self) -> T {
        const { assert!(size_of::<T>() <= MAX_BYTES, "Hram<T>: T is larger than MAX_BYTES") };
        #[cfg(target_arch = "sm83")]
        unsafe {
            let base = self.ptr() as usize as u8;
            let mut out = MaybeUninit::<T>::uninit();
            let dst = out.as_mut_ptr().cast::<u8>();
            unroll_load_c!(base, dst, T, 0, 1, 2, 3, 4, 5, 6, 7);
            out.assume_init()
        }
        #[cfg(not(target_arch = "sm83"))]
        unsafe {
            self.ptr().read_volatile()
        }
    }

    #[inline]
    fn set(&self, value: T) {
        const { assert!(size_of::<T>() <= MAX_BYTES, "Hram<T>: T is larger than MAX_BYTES") };
        #[cfg(target_arch = "sm83")]
        unsafe {
            let base = self.ptr() as usize as u8;
            let src = (&value as *const T).cast::<u8>();
            unroll_store_c!(base, src, T, 0, 1, 2, 3, 4, 5, 6, 7);
        }
        #[cfg(not(target_arch = "sm83"))]
        unsafe {
            self.ptr().write_volatile(value);
        }
    }

    #[inline]
    fn as_ptr(&self) -> *mut T {
        self.0.get() as *mut T
    }
}

// ===== Immediate-form unroll, for the hram!-generated handles =====
// Emitted into the user's crate (the asm names the cell's symbol), which is why
// that crate needs `#![feature(asm_experimental_arch)]`.

/// Internal: unroll immediate `ldh a, (STORAGE+i)` loads. Not public API.
#[doc(hidden)]
#[macro_export]
macro_rules! __hram_imm_load {
    ($storage:path, $dst:ident, $ty:ty, $($i:literal),*) => {$(
        if $i < ::core::mem::size_of::<$ty>() {
            let byte: u8;
            ::core::arch::asm!(
                ::core::concat!("ldh a, ({s} + ", $i, ")"),
                s = sym $storage,
                out("a") byte,
                options(nostack, preserves_flags, readonly),
            );
            $dst.add($i).write(byte);
        }
    )*};
}

/// Internal: unroll immediate `ldh (STORAGE+i), a` stores. Not public API.
#[doc(hidden)]
#[macro_export]
macro_rules! __hram_imm_store {
    ($storage:path, $src:ident, $ty:ty, $($i:literal),*) => {$(
        if $i < ::core::mem::size_of::<$ty>() {
            ::core::arch::asm!(
                ::core::concat!("ldh ({s} + ", $i, "), a"),
                s = sym $storage,
                in("a") $src.add($i).read(),
                options(nostack, preserves_flags),
            );
        }
    )*};
}

// ===== HramArea: raw fixed-size HRAM region =====

/// A raw, fixed-size region of High RAM, addressed through pointers.
///
/// The home for a routine that must execute from HRAM (an OAM DMA trampoline) or
/// a scratch buffer. Declare it with [`hram!`] (`static NAME: [u8; N];`). HRAM is
/// `NOLOAD`, so the runtime zero-initialises it at startup (see the crate docs);
/// fill it at runtime.
#[repr(transparent)]
pub struct HramArea<const N: usize>(UnsafeCell<MaybeUninit<[u8; N]>>);

unsafe impl<const N: usize> Sync for HramArea<N> {}

impl<const N: usize> HramArea<N> {
    /// Reserve an area. HRAM is `NOLOAD`, so its bytes are not loaded from ROM; the
    /// runtime zero-initialises HRAM at startup (see the crate docs).
    ///
    /// # Safety
    ///
    /// The area must come to rest in High RAM (`0xFF80..=0xFFFE`); [`hram!`]
    /// guarantees this.
    pub const unsafe fn uninit() -> Self {
        Self(UnsafeCell::new(MaybeUninit::uninit()))
    }

    /// A pointer to the start of the area.
    pub const fn as_ptr(&self) -> *const u8 {
        self.0.get() as *const u8
    }

    /// A mutable pointer to the start of the area.
    pub const fn as_mut_ptr(&self) -> *mut u8 {
        self.0.get() as *mut u8
    }

    /// The area's length in bytes (`N`).
    pub const fn len(&self) -> usize {
        N
    }

    /// Whether the area is zero bytes.
    pub const fn is_empty(&self) -> bool {
        N == 0
    }
}

// ===== The declaration macro =====

/// Declare static HRAM cells and areas, each at a linker-assigned High RAM
/// address.
///
/// A plain type is a typed cell read/written with the immediate `ldh (n)` form
/// (bring [`HramCell`] into scope: `use gb_hram::prelude::*`). An `[u8; N]` type
/// is a raw [`HramArea`]. Cells are zero-initialised by the runtime (which must
/// clear HRAM before `main`, like `.bss`); see the crate docs.
///
/// Write `static NAME as "symbol": T;` to export the cell's storage under a fixed
/// symbol (e.g. to share a cell with C or assembly). The symbol is emitted with
/// the target's usual prefix.
///
/// The cell's accessors emit `ldh` inline asm in the calling crate, so that crate
/// needs `#![feature(asm_experimental_arch)]`.
///
/// ```ignore
/// use gb_hram::{hram, prelude::*};
/// hram! {
///     /// Frame counter, bumped by the VBlank handler.
///     pub static FRAME: u8;
///     static SCROLL_X: u16;
///     static OAM_DMA: [u8; 13];   // an HramArea<13>
/// }
/// FRAME.set(FRAME.get().wrapping_add(1));
/// ```
#[macro_export]
macro_rules! hram {
    () => {};

    // Raw area: `static NAME: [u8; N];`
    (
        $(#[$attr:meta])*
        $vis:vis static $name:ident: [u8; $n:expr];
        $($rest:tt)*
    ) => {
        $(#[$attr])*
        #[unsafe(link_section = ::core::concat!("_HRAM.", ::core::stringify!($name)))]
        $vis static $name: $crate::HramArea<{ $n }> = unsafe { $crate::HramArea::uninit() };
        $crate::hram! { $($rest)* }
    };

    // Typed cell: `static NAME: T;`
    //
    // Generates a module `NAME` holding the hidden storage and the immediate-`ldh`
    // handle, plus a same-named const `NAME` (a different namespace) that is the
    // user-facing handle value. `NAME.get()` resolves to the const; `NAME::Handle`
    // to the module's type.
    (
        $(#[$attr:meta])*
        $vis:vis static $name:ident $(as $sym:literal)?: $ty:ty;
        $($rest:tt)*
    ) => {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        $vis mod $name {
            use super::*;

            $(#[unsafe(export_name = $sym)])?
            #[unsafe(link_section = ::core::concat!("_HRAM.", ::core::stringify!($name)))]
            static STORAGE: $crate::Hram<$ty> = unsafe { $crate::Hram::uninit() };

            /// The cell's handle type (zero-sized): carries the immediate `ldh` accessors.
            pub struct Handle;

            impl $crate::HramCell for Handle {
                type Value = $ty;

                #[inline]
                fn get(&self) -> $ty {
                    const {
                        assert!(
                            ::core::mem::size_of::<$ty>() <= $crate::MAX_BYTES,
                            "hram! cell is larger than MAX_BYTES",
                        )
                    };
                    #[cfg(target_arch = "sm83")]
                    unsafe {
                        let mut out = ::core::mem::MaybeUninit::<$ty>::uninit();
                        let dst = out.as_mut_ptr().cast::<u8>();
                        $crate::__hram_imm_load!(STORAGE, dst, $ty, 0, 1, 2, 3, 4, 5, 6, 7);
                        out.assume_init()
                    }
                    #[cfg(not(target_arch = "sm83"))]
                    unsafe {
                        STORAGE.ptr().read_volatile()
                    }
                }

                #[inline]
                fn set(&self, value: $ty) {
                    const {
                        assert!(
                            ::core::mem::size_of::<$ty>() <= $crate::MAX_BYTES,
                            "hram! cell is larger than MAX_BYTES",
                        )
                    };
                    #[cfg(target_arch = "sm83")]
                    unsafe {
                        let src = (&value as *const $ty).cast::<u8>();
                        $crate::__hram_imm_store!(STORAGE, src, $ty, 0, 1, 2, 3, 4, 5, 6, 7);
                    }
                    #[cfg(not(target_arch = "sm83"))]
                    unsafe {
                        STORAGE.ptr().write_volatile(value);
                    }
                }

                #[inline]
                fn as_ptr(&self) -> *mut $ty {
                    STORAGE.ptr()
                }
            }
        }

        $(#[$attr])*
        #[allow(non_upper_case_globals)]
        $vis const $name: $name::Handle = $name::Handle;

        $crate::hram! { $($rest)* }
    };
}
