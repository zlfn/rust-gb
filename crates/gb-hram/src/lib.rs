#![no_std]

//! Typed handles to the Game Boy's High RAM (HRAM, `0xFF80..=0xFFFE`).
//!
//! HRAM is a 127-byte region the CPU can still reach while an OAM DMA holds the
//! bus, and the `ldh` instructions address it in a single byte. Cells declared
//! with the [`hram!`] macro are read and written with the **immediate** `ldh (n)`
//! form (2 bytes / 3 cycles): the linker assigns each cell a fixed HRAM address
//! and the low byte is baked into the instruction. That is faster and smaller
//! than the 3-byte / 4-cycle absolute `ld (nn)` used for WRAM.
//!
//! # Which kind to declare
//!
//! An access one byte wide is a single instruction, and the CPU takes interrupts
//! only between instructions, so it cannot be observed half done. A wider access
//! is several instructions and an interrupt landing in the middle leaves the
//! reader with a value that never existed. The three kinds differ in what they do
//! about that.
//!
//! | Kind | Width | Access |
//! |------|-------|--------|
//! | [`HramAtomicCell<T>`] | one byte | `get()` / `set()` |
//! | [`HramCell<T>`] | up to [`MAX_BYTES`] | `get(cs)` / `set(cs, v)` |
//! | [`HramArea<N>`] | any | raw pointers |
//!
//! [`HramCell`] takes a [`CriticalSection`] because nothing else can make a
//! multi-instruction access indivisible on this CPU. Obtaining the token is the
//! caller's business; a cell reached only from the main loop, never from an
//! interrupt handler, still needs one.
//!
//! # Examples
//!
//! ```ignore
//! #![feature(asm_experimental_arch)]
//! use gb_hram::hram;
//!
//! hram! {
//!     /// Frames elapsed, bumped by the VBlank handler.
//!     pub static FRAME: HramAtomicCell<u8>;
//!     static SCROLL: HramCell<ScrollState>;
//!     static OAM_DMA: HramArea<13>;
//! }
//!
//! fn tick() {
//!     FRAME.set(FRAME.get().wrapping_add(1));   // ldh a,(n) / ldh (n),a
//! }
//! ```
//!
//! Write `static NAME as "symbol": ...;` to export the storage under a fixed
//! symbol, to share a cell with C or assembly. The symbol is emitted with the
//! target's usual prefix.
//!
//! The accessors emit `ldh` inline asm in the calling crate, so that crate needs
//! `#![feature(asm_experimental_arch)]`.
//!
//! # Zero initialisation
//!
//! HRAM is `NOLOAD`, so nothing is loaded from ROM, and a conforming runtime is
//! required to clear `0xFF80..=0xFFFE` before `main`. A cell whose type has a
//! valid all-zero bit pattern therefore starts at `0` and may be read before it
//! is first written; any other type must be written first.

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;

pub use critical_section::CriticalSection;

/// Read and write an HRAM cell under a [`CriticalSection`].
///
/// Implemented by every handle the [`hram!`] macro generates. An
/// [`HramAtomicCell`] handle implements it too, so code holding a token can
/// reach either kind through one interface.
pub trait HramAccess {
    /// The stored type.
    type Value: Copy;

    /// Read the cell.
    ///
    /// The runtime zero-initialises HRAM (see the crate docs), so for a `Value`
    /// whose every bit pattern is valid this may be called before the first
    /// write and yields `0`; any other type must be written first.
    fn get_cs(&self, cs: CriticalSection<'_>) -> Self::Value;

    /// Write the cell.
    fn set_cs(&self, cs: CriticalSection<'_>, value: Self::Value);

    /// The cell's address, for raw access or to feed an `ldh`-based routine.
    fn as_ptr(&self) -> *mut Self::Value;
}

/// Read and write a one-byte HRAM cell without a [`CriticalSection`].
///
/// Implemented by the handles for [`HramAtomicCell`] declarations. A one-byte
/// `ldh` is a single instruction, so no token is needed to make it indivisible.
pub trait HramAtomicAccess: HramAccess {
    /// Read the cell.
    fn get(&self) -> Self::Value;

    /// Write the cell.
    fn set(&self, value: Self::Value);
}

/// Common imports for HRAM access (`use gb_hram::prelude::*`).
pub mod prelude {
    pub use crate::{HramAccess, HramAtomicAccess};
}

/// The widest [`HramCell`] value an access unrolls into straight-line `ldh`s.
/// HRAM is only 127 bytes, so cells are small; a wider type is a compile error.
pub const MAX_BYTES: usize = 8;

/// Storage for a one-byte cell, accessed without a [`CriticalSection`].
///
/// A one-byte `ldh` is a single instruction, so an interrupt handler can never
/// observe the cell mid-write. Declaring a wider type is a compile error.
///
/// Declare it with [`hram!`]; the accessors live on the handle that macro
/// generates, not on this type.
#[repr(transparent)]
pub struct HramAtomicCell<T>(UnsafeCell<MaybeUninit<T>>);

// The Game Boy is single-core, so there is no cross-thread aliasing, and a
// one-byte access cannot tear against an interrupt handler.
unsafe impl<T> Sync for HramAtomicCell<T> {}

impl<T> HramAtomicCell<T> {
    /// Reserve a cell.
    ///
    /// # Safety
    ///
    /// The cell must come to rest at an address in `0xFF80..=0xFFFE`. [`hram!`]
    /// guarantees this; a hand-placed cell needs `#[link_section = "_HRAM.*"]`.
    pub const unsafe fn uninit() -> Self {
        Self(UnsafeCell::new(MaybeUninit::uninit()))
    }

    /// The cell's address as a raw pointer.
    pub const fn ptr(&self) -> *mut T {
        self.0.get() as *mut T
    }
}

/// Storage for a cell read and written under a [`CriticalSection`].
///
/// An access wider than one byte is several `ldh` instructions, so an interrupt
/// landing between them sees the cell half updated. The token is the proof that
/// no interrupt can.
///
/// Declare it with [`hram!`]; the accessors live on the handle that macro
/// generates, not on this type.
#[repr(transparent)]
pub struct HramCell<T>(UnsafeCell<MaybeUninit<T>>);

unsafe impl<T> Sync for HramCell<T> {}

impl<T> HramCell<T> {
    /// Reserve a cell.
    ///
    /// # Safety
    ///
    /// The cell must come to rest at an address in `0xFF80..=0xFFFE`. [`hram!`]
    /// guarantees this; a hand-placed cell needs `#[link_section = "_HRAM.*"]`.
    pub const unsafe fn uninit() -> Self {
        Self(UnsafeCell::new(MaybeUninit::uninit()))
    }

    /// The cell's address as a raw pointer.
    pub const fn ptr(&self) -> *mut T {
        self.0.get() as *mut T
    }
}

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

/// A raw, fixed-size region of High RAM, addressed through pointers.
///
/// The home for a routine that must execute from HRAM (an OAM DMA trampoline) or
/// a scratch buffer. Declare it with [`hram!`]. HRAM is `NOLOAD`, so the runtime
/// zero-initialises it at startup; fill it at runtime.
#[repr(transparent)]
pub struct HramArea<const N: usize>(UnsafeCell<MaybeUninit<[u8; N]>>);

unsafe impl<const N: usize> Sync for HramArea<N> {}

impl<const N: usize> HramArea<N> {
    /// Reserve an area.
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

    /// A mutable pointer to the area as a fixed-size byte array.
    pub const fn as_array_ptr(&self) -> *mut [u8; N] {
        self.0.get() as *mut [u8; N]
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

/// Declare static HRAM cells and areas, each at a linker-assigned High RAM
/// address.
///
/// The declared type selects the kind and must be written as one of the three
/// names below, unqualified; the macro matches on it syntactically, so a path or
/// an alias does not work.
///
/// ```ignore
/// hram! {
///     pub static FRAME: HramAtomicCell<u8>;
///     static SCROLL: HramCell<ScrollState>;
///     static OAM_DMA: HramArea<13>;
///     static CURRENT_BANK as "_current_bank": HramAtomicCell<u8>;
/// }
/// ```
///
/// Each cell becomes a same-named constant carrying the accessors. See the crate
/// docs for which kind to reach for.
#[macro_export]
macro_rules! hram {
    () => {};

    (
        $(#[$attr:meta])*
        $vis:vis static $name:ident $(as $sym:literal)?: HramArea<$n:tt>;
        $($rest:tt)*
    ) => {
        $(#[$attr])*
        $(#[unsafe(export_name = $sym)])?
        #[unsafe(link_section = ::core::concat!("_HRAM.", ::core::stringify!($name)))]
        $vis static $name: $crate::HramArea<{ $n }> = unsafe { $crate::HramArea::uninit() };
        $crate::hram! { $($rest)* }
    };

    (
        $(#[$attr:meta])*
        $vis:vis static $name:ident $(as $sym:literal)?: HramAtomicCell<$ty:ty>;
        $($rest:tt)*
    ) => {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        $vis mod $name {
            use super::*;

            const _: () = assert!(
                ::core::mem::size_of::<$ty>() == 1,
                "HramAtomicCell holds one byte; use HramCell for a wider type",
            );

            $(#[unsafe(export_name = $sym)])?
            #[unsafe(link_section = ::core::concat!("_HRAM.", ::core::stringify!($name)))]
            static STORAGE: $crate::HramAtomicCell<$ty> =
                unsafe { $crate::HramAtomicCell::uninit() };

            /// The cell's handle (zero-sized): carries the immediate `ldh` accessors.
            pub struct Handle;

            impl $crate::HramAccess for Handle {
                type Value = $ty;

                #[inline]
                fn get_cs(&self, _cs: $crate::CriticalSection<'_>) -> $ty {
                    <Self as $crate::HramAtomicAccess>::get(self)
                }

                #[inline]
                fn set_cs(&self, _cs: $crate::CriticalSection<'_>, value: $ty) {
                    <Self as $crate::HramAtomicAccess>::set(self, value)
                }

                #[inline]
                fn as_ptr(&self) -> *mut $ty {
                    STORAGE.ptr()
                }
            }

            impl $crate::HramAtomicAccess for Handle {
                #[inline]
                fn get(&self) -> $ty {
                    unsafe {
                        let mut out = ::core::mem::MaybeUninit::<$ty>::uninit();
                        let dst = out.as_mut_ptr().cast::<u8>();
                        $crate::__hram_imm_load!(STORAGE, dst, $ty, 0);
                        out.assume_init()
                    }
                }

                #[inline]
                fn set(&self, value: $ty) {
                    unsafe {
                        let src = (&value as *const $ty).cast::<u8>();
                        $crate::__hram_imm_store!(STORAGE, src, $ty, 0);
                    }
                }
            }
        }

        $(#[$attr])*
        #[allow(non_upper_case_globals)]
        $vis const $name: $name::Handle = $name::Handle;

        $crate::hram! { $($rest)* }
    };

    (
        $(#[$attr:meta])*
        $vis:vis static $name:ident $(as $sym:literal)?: HramCell<$ty:ty>;
        $($rest:tt)*
    ) => {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        $vis mod $name {
            use super::*;

            const _: () = assert!(
                ::core::mem::size_of::<$ty>() <= $crate::MAX_BYTES,
                "HramCell is wider than MAX_BYTES",
            );

            $(#[unsafe(export_name = $sym)])?
            #[unsafe(link_section = ::core::concat!("_HRAM.", ::core::stringify!($name)))]
            static STORAGE: $crate::HramCell<$ty> = unsafe { $crate::HramCell::uninit() };

            /// The cell's handle (zero-sized): carries the immediate `ldh` accessors.
            pub struct Handle;

            impl $crate::HramAccess for Handle {
                type Value = $ty;

                #[inline]
                fn get_cs(&self, _cs: $crate::CriticalSection<'_>) -> $ty {
                    unsafe {
                        let mut out = ::core::mem::MaybeUninit::<$ty>::uninit();
                        let dst = out.as_mut_ptr().cast::<u8>();
                        $crate::__hram_imm_load!(STORAGE, dst, $ty, 0, 1, 2, 3, 4, 5, 6, 7);
                        out.assume_init()
                    }
                }

                #[inline]
                fn set_cs(&self, _cs: $crate::CriticalSection<'_>, value: $ty) {
                    unsafe {
                        let src = (&value as *const $ty).cast::<u8>();
                        $crate::__hram_imm_store!(STORAGE, src, $ty, 0, 1, 2, 3, 4, 5, 6, 7);
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
