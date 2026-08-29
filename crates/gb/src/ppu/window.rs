//! The window: a second tilemap laid over the background.
//!
//! It has no scroll of its own. Wherever it is placed, it draws from its grid's
//! top left corner, so moving what it shows means rewriting the grid. That fixity
//! is what makes it a status bar or a text box while the background scrolls
//! underneath.
//!
//! # Position
//!
//! The hardware register is the screen column *plus seven*, so `WX` of 7 is the
//! left edge. [`set_position`] takes the screen column and adds it, which also
//! puts the off-screen values out of reach: `WX` below 7 starts the window left
//! of the screen and is where the hardware behaves least predictably. Reach for
//! [`mmio::WX`](crate::mmio::WX) directly if that is wanted.
//!
//! # Writing mid-frame
//!
//! `WX`, `WY` and the enable bit are least glitchy written during VBlank, or
//! during HBlank where they must change mid-frame, which is what the [`Access`]
//! here is for. [`set_map`] is not among them: it takes effect from the next
//! tile fetched and has nothing to go wrong.
//!
//! Where the window has to come and go within a frame, [`hide`] is the way and
//! the enable bit is not. The PPU turns the window on for a frame when `WY`
//! first matches `LY`, and on the Game Boy Color clearing the enable bit undoes
//! that: the window then stays away until `WY` matches again, which for the rest
//! of the frame it cannot. See <https://gbdev.io/pandocs/Window.html>.

use crate::mmio::{LCDC, WX, WY};

use super::map::Map;
use super::{Access, wait_blank};

/// What `WX` is offset by: `WX` of 7 is screen column 0.
pub const X_OFFSET: u8 = 7;

/// Where the window's top left corner sits on screen, as `(x, y)`.
#[inline]
pub fn position() -> (u8, u8) {
    (WX.read().saturating_sub(X_OFFSET), WY.read())
}

/// Put the window's top left corner at `(x, y)` on screen.
///
/// An `x` of 160 or more, or a `y` of 144 or more, leaves it off screen; [`hide`]
/// is the way to say that on purpose.
#[inline]
pub fn set_position(access: Access<'_>, x: u8, y: u8) {
    if matches!(access, Access::Polled) {
        wait_blank();
    }
    WX.write(x.saturating_add(X_OFFSET));
    WY.write(y);
}

/// Take the window off screen without disturbing the enable bit.
#[inline]
pub fn hide(access: Access<'_>) {
    if matches!(access, Access::Polled) {
        wait_blank();
    }
    // One past the last visible `WX`, and so also past the monochrome bug at 166
    // where the window spans the screen shifted down a line.
    WX.write(167);
}

/// Whether the PPU draws the window at all, from `LCDC` bit 5.
#[inline]
pub fn enabled() -> bool {
    LCDC.read().window_enable()
}

/// Set `LCDC` bit 5.
///
/// For taking the window away and bringing it back within a frame, use [`hide`]
/// and [`set_position`]: on the Game Boy Color, clearing this bit also clears the
/// condition that lets the window appear at all, and it will not come back that
/// frame.
#[inline]
pub fn set_enabled(access: Access<'_>, on: bool) {
    if matches!(access, Access::Polled) {
        wait_blank();
    }
    // Read-modify-write: the LCD enable bit is the only reason this is unsafe.
    unsafe { LCDC.write(LCDC.read().with_window_enable(on)) };
}

/// Which grid the window reads, from `LCDC` bit 6.
#[inline]
pub fn map() -> Map {
    if LCDC.read().window_tilemap_high() { Map::One } else { Map::Zero }
}

/// Point the window at a grid.
#[inline]
pub fn set_map(map: Map) {
    unsafe { LCDC.write(LCDC.read().with_window_tilemap_high(map == Map::One)) };
}
