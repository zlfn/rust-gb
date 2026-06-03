//! Memory-mapped hardware registers and memory regions as typed [`voladdress`]
//! handles. Every access is volatile, with the read/write permission in the type.
//!
//! The Game Boy has no faulting memory, so an accessor is `unsafe` only when it
//! can make the CPU execute arbitrary bytes (OAM DMA started outside HRAM, or
//! enabling an interrupt with no handler) or physically damage the panel
//! (turning the LCD off outside VBlank). Plain VRAM, OAM, and register access is
//! safe: at worst it draws wrong pixels, which is not undefined behavior.
//!
//! Each handle is `VolAddress<T, R, W>` (or `VolBlock`/`VolGrid2d`): `R` and `W`
//! are the read and write permissions, each `Safe`, `Unsafe`, or `()` (no such
//! method). Addresses follow the Pan Docs / GBDK `hardware.h` map.
//!
//! Registers and their value types are re-exported flat (`mmio::LCDC`). The
//! CGB-only registers live in [`cgb`], gated by the `cgb` feature.

mod audio;
mod interrupt;
mod joypad;
mod ppu;
mod serial;
mod timer;

pub use audio::*;
pub use interrupt::*;
pub use joypad::*;
pub use ppu::*;
pub use serial::*;
pub use timer::*;

#[cfg(feature = "cgb")]
pub mod cgb;
