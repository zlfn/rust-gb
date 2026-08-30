//! Joypad input (`JOYP`).

use bitfield_struct::bitfield;
use voladdress::{Safe, VolAddress};

/// Joypad register (`JOYP`). Active low: a cleared bit means selected, or the input
/// pressed.
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7-6 | —         |     | Unused. |
/// | 5   | `buttons` | R/W | Selects the action buttons (A/B/Select/Start). |
/// | 4   | `dpad`    | R/W | Selects the d-pad. |
/// | 3   | `p13`     | RO  | Input line P13: Down (d-pad) or Start (buttons). |
/// | 2   | `p12`     | RO  | Input line P12: Up (d-pad) or Select (buttons). |
/// | 1   | `p11`     | RO  | Input line P11: Left (d-pad) or B (buttons). |
/// | 0   | `p10`     | RO  | Input line P10: Right (d-pad) or A (buttons). |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct Joypad {
    /// Input line P10: Right (d-pad) or A (buttons).
    pub p10: bool,
    /// Input line P11: Left (d-pad) or B (buttons).
    pub p11: bool,
    /// Input line P12: Up (d-pad) or Select (buttons).
    pub p12: bool,
    /// Input line P13: Down (d-pad) or Start (buttons).
    pub p13: bool,
    /// Selects the d-pad.
    pub dpad: bool,
    /// Selects the action buttons (A/B/Select/Start).
    pub buttons: bool,
    #[bits(2)]
    __: u8,
}

/// Joypad: write selects the button or d-pad group, read returns its state.
pub const JOYP: VolAddress<Joypad, Safe, Safe> = unsafe { VolAddress::new(0xFF00) };

const _: () = {
    assert!(Joypad::new().with_buttons(true).into_bits() == 0b0010_0000);
    assert!(Joypad::new().with_dpad(true).into_bits() == 0b0001_0000);
    assert!(Joypad::new().with_p10(true).into_bits() == 0b0000_0001);
};
