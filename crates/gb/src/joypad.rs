//! This module reads the joypad's eight buttons.
//!
//! [`read`] returns a [`Buttons`], a bit per button and set where held. The
//! hardware is the other way round, clearing the bit of a button that is down.
//! See <https://gbdev.io/pandocs/Joypad_Input.html>.
//!
//! [`Pad`] is the usual way in. It keeps the previous reading, so a program can
//! ask what changed rather than only what is held.
//!
//! ```ignore
//! let mut pad = Pad::new();
//! loop {
//!     ppu::wait_vblank();
//!     pad.poll();
//!
//!     if pad.just_pressed.a() {
//!         jump();
//!     }
//!     x = x.wrapping_add_signed(pad.pressed.x() as i8);
//! }
//! ```
//!
//! # Reading from a handler
//!
//! [`read`] selects a row, reads it, then selects the other and reads that. A
//! handler reading the joypad in between finishes with both rows off, so the
//! half of the interrupted read that has not happened yet comes back as nothing
//! pressed. Nothing reports it, so a program should poll in one place.

use bitfield_struct::bitfield;

use crate::mmio::{JOYP, Joypad};

/// The eight buttons, set where held.
///
/// The d-pad occupies the low nibble and the action buttons the high one, which
/// is the layout GBDK's `J_` constants use.
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct Buttons {
    /// Right on the d-pad.
    pub right: bool,
    /// Left on the d-pad.
    pub left: bool,
    /// Up on the d-pad.
    pub up: bool,
    /// Down on the d-pad.
    pub down: bool,
    /// The A button.
    pub a: bool,
    /// The B button.
    pub b: bool,
    /// The Select button.
    pub select: bool,
    /// The Start button.
    pub start: bool,
}

/// Where the d-pad points left or right, as a number to add to a coordinate.
///
/// ```ignore
/// x = x.wrapping_add_signed(pad.pressed.x() as i8 * SPEED);
/// ```
#[repr(i8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DPadX {
    /// Left, negative because screen coordinates grow to the right.
    Left = -1,
    /// Neither, or both at once.
    Neutral = 0,
    /// Right.
    Right = 1,
}

/// Where the d-pad points up or down, as a number to add to a coordinate.
#[repr(i8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DPadY {
    /// Up, negative because screen coordinates grow downwards.
    Up = -1,
    /// Neither, or both at once.
    Neutral = 0,
    /// Down.
    Down = 1,
}

impl Buttons {
    /// Where the d-pad points left or right.
    #[inline]
    pub const fn x(self) -> DPadX {
        match (self.left(), self.right()) {
            (true, false) => DPadX::Left,
            (false, true) => DPadX::Right,
            _ => DPadX::Neutral,
        }
    }

    /// Where the d-pad points up or down.
    #[inline]
    pub const fn y(self) -> DPadY {
        match (self.up(), self.down()) {
            (true, false) => DPadY::Up,
            (false, true) => DPadY::Down,
            _ => DPadY::Neutral,
        }
    }
}

// Active low, so clearing a select bit is what turns that row on.
const DPAD: Joypad = Joypad::new().with_buttons(true);
const BUTTONS: Joypad = Joypad::new().with_dpad(true);
const NEITHER: Joypad = Joypad::new().with_dpad(true).with_buttons(true);

/// Read the buttons.
pub fn read() -> Buttons {
    // A select line takes time to settle, so a row is read more than once and
    // only the last read counts. The counts are GBDK's. The second row gets more
    // because both select lines move for it where only one moves for the first.
    JOYP.write(DPAD);
    JOYP.read();
    let dpad = JOYP.read().into_bits() & 0x0F;

    JOYP.write(BUTTONS);
    JOYP.read();
    JOYP.read();
    JOYP.read();
    JOYP.read();
    JOYP.read();
    let buttons = JOYP.read().into_bits() & 0x0F;

    // With both rows off no input line can fall, so a press between calls
    // cannot reach the joypad interrupt.
    JOYP.write(NEITHER);

    Buttons::from_bits(!((buttons << 4) | dpad))
}

/// The buttons, and what changed at the last [`poll`](Self::poll).
pub struct Pad {
    /// Held now.
    pub pressed: Buttons,
    /// Went down at the last poll.
    pub just_pressed: Buttons,
    /// Came up at the last poll.
    pub just_released: Buttons,
}

impl Pad {
    /// A pad with nothing held.
    ///
    /// A button already down when the first poll runs counts as newly pressed.
    pub const fn new() -> Self {
        Pad {
            pressed: Buttons::new(),
            just_pressed: Buttons::new(),
            just_released: Buttons::new(),
        }
    }

    /// Read the buttons and work out what changed.
    #[inline]
    pub fn poll(&mut self) {
        let now = read().into_bits();
        let was = self.pressed.into_bits();
        self.just_pressed = Buttons::from_bits(now & !was);
        self.just_released = Buttons::from_bits(!now & was);
        self.pressed = Buttons::from_bits(now);
    }
}

impl Default for Pad {
    fn default() -> Self {
        Self::new()
    }
}

const _: () = {
    assert!(DPAD.into_bits() == 0b0010_0000);
    assert!(BUTTONS.into_bits() == 0b0001_0000);
    assert!(NEITHER.into_bits() == 0b0011_0000);
    assert!(Buttons::new().with_right(true).into_bits() == 0b0000_0001);
    assert!(Buttons::new().with_a(true).into_bits() == 0b0001_0000);
    assert!(Buttons::new().with_start(true).into_bits() == 0b1000_0000);
};
