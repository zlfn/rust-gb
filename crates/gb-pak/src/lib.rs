#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! The Game Pak's own hardware.
//!
//! The memory bank controller answers writes to the ROM address range as register
//! writes, and one 8 KiB window at `0xA000..=0xBFFF` shows whatever those
//! registers last selected. For example SRAM, where a game saves its data.
//!
//! # Sharing the window
//!
//! Every peripheral here shares that window, and selecting one deselects the
//! rest, so they do not nest:
//!
//! ```ignore
//! // Wrong: the second read comes from the rtc.
//! FILE.open(cs, |f| {
//!     let a = f.gold.get();
//!     let t = rtc::time(cs);   // selects rtc registers
//!     let b = f.gold.get();    // reads the rtc, not the save
//! });
//! ```
//!
//! An interrupt handler nests the same way without the code showing it, and one
//! racing the interrupted code for the same bytes is a data race. Every operation
//! takes a [`CriticalSection`] to rule that out, from `gb::interrupt::free` or
//! `critical_section::with`. It does not rule out the nesting above.
//!
//! # What a cartridge has
//!
//! `cargo-gb` derives the capabilities below from `cartridge_type` and `ram_size`
//! in `header.toml`, so a program that reaches for hardware the cartridge lacks
//! fails to compile rather than writing to a chip that is not there.
//!
//! MBC1 spends the same two register bits on ROM banks above 512 KiB and on SRAM
//! banks, so `wide_banks` and more than one SRAM bank cannot both be set.

#![doc = "| Module | Needs | Present on |"]
#![doc = "|---|---|---|"]
#![doc = "| [`sram`](mod@crate::sram) | `ram_size` above zero | MBC1, MBC3, MBC5 |"]
#![cfg_attr(gb_pak_rtc, doc = "| [`rtc`] | `+TIMER` | MBC3 |")]
#![cfg_attr(gb_pak_rumble, doc = "| [`rumble`] | `+RUMBLE` | MBC5 |")]
#![cfg_attr(gb_pak_tilt, doc = "| [`tilt`], [`eeprom`] | MBC7 | MBC7 |")]

/// The shared window. Everything but the motor is read through it.
pub(crate) const WINDOW: usize = 0xA000;
pub(crate) const WINDOW_LEN: usize = 0x2000;

pub use critical_section::CriticalSection;

pub(crate) mod reg;

pub mod sram;

pub use gb_pak_macros::sram;

#[cfg(gb_pak_rtc)]
#[cfg_attr(docsrs, doc(cfg(gb_pak_rtc)))]
pub mod rtc;

#[cfg(gb_pak_rumble)]
#[cfg_attr(docsrs, doc(cfg(gb_pak_rumble)))]
pub mod rumble;

#[cfg(gb_pak_tilt)]
#[cfg_attr(docsrs, doc(cfg(gb_pak_tilt)))]
pub mod tilt;

#[cfg(gb_pak_tilt)]
#[cfg_attr(docsrs, doc(cfg(gb_pak_tilt)))]
pub mod eeprom;
