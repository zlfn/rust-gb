//! The background: a tilemap seen through a window the size of the screen.
//!
//! The grid is 256 by 256 pixels and the screen shows 160 by 144 of it, placed by
//! [`set_scroll`] and wrapping at either edge. What fills the grid is
//! [`map`](super::map)'s business; this is where it sits.
//!
//! The scroll registers are re-read as the PPU fetches each tile, so writing them
//! part way down a frame moves the rest of it. That is how a program wobbles or
//! parallaxes a background. See <https://gbdev.io/pandocs/Scrolling.html>.

use crate::mmio::{LCDC, SCX, SCY};

use super::map::Map;

/// Where the screen sits on the grid, as `(x, y)`.
#[inline]
pub fn scroll() -> (u8, u8) {
    (SCX.read(), SCY.read())
}

/// Move the screen to `(x, y)` on the grid.
#[inline]
pub fn set_scroll(x: u8, y: u8) {
    SCX.write(x);
    SCY.write(y);
}

/// Move the screen horizontally.
///
/// Separate from [`set_scroll`] because a per-scanline effect has room for one
/// register write and not two.
#[inline]
pub fn set_scroll_x(x: u8) {
    SCX.write(x);
}

/// Move the screen vertically.
#[inline]
pub fn set_scroll_y(y: u8) {
    SCY.write(y);
}

/// Which grid the background reads, from `LCDC` bit 3.
#[inline]
pub fn map() -> Map {
    if LCDC.read().bg_tilemap_high() { Map::One } else { Map::Zero }
}

/// Point the background at a grid.
#[inline]
pub fn set_map(map: Map) {
    // Read-modify-write: the enable bit is the only reason this is unsafe.
    unsafe { LCDC.write(LCDC.read().with_bg_tilemap_high(map == Map::One)) };
}

/// `LCDC` bit 0, which means different things on the two machines.
///
/// On the original Game Boy, clearing it blanks the background and the window, leaving
/// only objects. On the Game Boy Color they keep drawing, and clearing it instead
/// gives objects priority over them.
#[inline]
pub fn enabled() -> bool {
    LCDC.read().bg_window_enable()
}

/// Set `LCDC` bit 0. See [`enabled`] for what it does on each machine.
#[inline]
pub fn set_enabled(on: bool) {
    unsafe { LCDC.write(LCDC.read().with_bg_window_enable(on)) };
}
