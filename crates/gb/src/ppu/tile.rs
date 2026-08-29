//! Tile data: the 384 slots at `0x8000`, sixteen bytes each.
//!
//! # Slots and indices
//!
//! A tilemap byte is eight bits, so it names one of 256 tiles, but there are 384.
//! `LCDC` bit 4 chooses which window of 256 it names, and the two windows overlap
//! in the middle:
//!
//! | Block | Address | Slot | [`Base8000`](Addressing::Base8000) | [`Base8800`](Addressing::Base8800) |
//! |---|---|---|---|---|
//! | 0 | `8000-87FF` | 0-127 | index 0-127 | out of reach |
//! | 1 | `8800-8FFF` | 128-255 | index 128-255 | index 128-255 |
//! | 2 | `9000-97FF` | 256-383 | out of reach | index 0-127 |
//!
//! Objects ignore the bit and always read as `Base8000`, so putting the
//! background on `Base8800` gives each a private block and leaves block 1 shared.
//! That is how all 384 become reachable at once.
//!
//! Writing here addresses a slot, which means the same tile whichever mode is
//! set; [`index`] converts one to the byte a map needs. See
//! <https://gbdev.io/pandocs/Tile_Data.html>.

use crate::mmio::{LCDC, Tile, VRAM_TILES};

use super::Access;

/// Tile slots in one VRAM bank.
pub const SLOT_COUNT: u16 = 384;

/// Which 256 slots a tilemap byte names, from `LCDC` bit 4.
///
/// The `$8000` and `$8800` methods, named as Pan Docs and the wider Game Boy
/// development community name them. `Base8800` is a misnomer kept for that
/// familiarity: its base pointer is `0x9000`, and the index is read as signed,
/// which is what puts its second half back down in block 1.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Addressing {
    /// Base `0x8000`, index unsigned: blocks 0 and 1. Objects always use this.
    Base8000,
    /// Base `0x9000`, index signed: blocks 2 and 1.
    Base8800,
}

/// The slot a tilemap byte of `index` names under `mode`.
pub const fn slot(mode: Addressing, index: u8) -> u16 {
    match mode {
        Addressing::Base8000 => index as u16,
        // Signed: 0..=127 land in block 2, and 128..=255 read as -128..=-1 and
        // fall back into block 1.
        Addressing::Base8800 if index < 128 => 256 + index as u16,
        Addressing::Base8800 => index as u16,
    }
}

/// The tilemap byte that names `slot` under `mode`, if `mode` reaches it.
///
/// `None` for the block the mode leaves out, and for a slot past [`SLOT_COUNT`].
pub const fn index(mode: Addressing, slot: u16) -> Option<u8> {
    match mode {
        Addressing::Base8000 if slot < 256 => Some(slot as u8),
        Addressing::Base8800 if slot >= 256 && slot < SLOT_COUNT => Some((slot - 256) as u8),
        Addressing::Base8800 if slot >= 128 && slot < 256 => Some(slot as u8),
        _ => None,
    }
}

/// Which window `LCDC` currently selects for the background and window layers.
#[inline]
pub fn addressing() -> Addressing {
    if LCDC.read().tiledata_8000() {
        Addressing::Base8000
    } else {
        Addressing::Base8800
    }
}

/// Select the window for the background and window layers.
#[inline]
pub fn set_addressing(mode: Addressing) {
    let on = mode == Addressing::Base8000;
    // Read-modify-write: the enable bit is the only reason this is unsafe.
    unsafe { LCDC.write(LCDC.read().with_tiledata_8000(on)) };
}

/// Write one tile into `slot`.
///
/// # Panics
///
/// If `slot` is [`SLOT_COUNT`] or beyond, which would otherwise land in a
/// tilemap. The check folds away when `slot` is a constant.
#[inline]
pub fn write(access: Access<'_>, slot: u16, data: &Tile) {
    let dst = VRAM_TILES.index(slot as usize).as_usize() as *mut u8;
    unsafe { access.write(dst, data) };
}

/// Write `data` into consecutive slots from `first`.
///
/// # Panics
///
/// If the run would reach [`SLOT_COUNT`].
pub fn write_all(access: Access<'_>, first: u16, data: &[Tile]) {
    assert!(
        (first as usize)
            .checked_add(data.len())
            .is_some_and(|end| end <= SLOT_COUNT as usize)
    );
    if data.is_empty() {
        return;
    }
    // Slots are contiguous, so the whole run is one copy.
    let dst = VRAM_TILES.index(first as usize).as_usize() as *mut u8;
    unsafe { access.write(dst, data.as_flattened()) };
}
