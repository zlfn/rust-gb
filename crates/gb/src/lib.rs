//! `gb`: Game Boy hardware abstractions.
//!
//! # Feature flags
#![doc = document_features::document_features!()]
#![no_std]
#![feature(asm_experimental_arch)]
#![feature(linkage)]
#![feature(negative_impls)]
#![feature(abi_z80_interrupt)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod mmio;
pub mod interrupt;
pub mod joypad;
pub mod ppu;

// Also at the root, so the attribute reads as `#[gb::bank]` rather than
// `#[gb::bank::bank]`. A macro and a module may share a name.
#[cfg(feature = "bank")]
#[cfg_attr(docsrs, doc(cfg(feature = "bank")))]
pub use gb_bank_macros::bank;
#[cfg(feature = "pak")]
#[cfg_attr(docsrs, doc(cfg(feature = "pak")))]
pub use gb_pak_macros::sram;
pub use gb_hram::hram;
pub use gb_ram_fn::ram_fn;

#[cfg(feature = "bank")]
#[cfg_attr(docsrs, doc(cfg(feature = "bank")))]
#[doc(inline)]
pub use gb_bank as bank;
#[cfg(feature = "pak")]
#[cfg_attr(docsrs, doc(cfg(feature = "pak")))]
#[doc(inline)]
pub use gb_pak as pak;
#[doc(inline)]
pub use gb_hram as hram;
#[doc(inline)]
pub use gb_ram_fn as ram_fn;
#[doc(inline)]
pub use gb_rt as rt;

// Where the `gb-bank` and `gb-ram-fn` macros look when the invoking crate depends
// on this crate instead of on them.
#[cfg(feature = "bank")]
#[doc(hidden)]
pub use gb_bank as __bank;
#[cfg(feature = "pak")]
#[doc(hidden)]
pub use gb_pak as __pak;
#[doc(hidden)]
pub use gb_ram_fn as __ram_fn;

/// Whether the machine has the Game Boy Color's hardware.
///
/// From the value the boot ROM leaves in `A`, so it costs one load. A Game Boy
/// Advance answers yes: it runs the same hardware.
///
/// A cartridge built for both machines needs this wherever a Color feature has
/// nothing to fall back on.
#[inline]
pub fn is_cgb() -> bool {
    rt::boot::a() == 0x11
}

/// Whether the machine is a Game Boy Advance running a Game Boy Color cartridge.
///
/// Programs usually ask in order to lighten their palettes, the Advance's screen
/// being the darker of the two.
#[inline]
pub fn is_gba() -> bool {
    is_cgb() && rt::boot::b() & 1 != 0
}
