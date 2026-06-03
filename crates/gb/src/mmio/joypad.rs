//! Joypad input (`JOYP`).

use bitfield_struct::bitfield;
use voladdress::{Safe, VolAddress};

/// Joypad register (`JOYP`) value.
///
/// Active low: a cleared bit means the group is selected, or the input pressed.
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
    /// Select the d-pad.
    pub select_dpad: bool,
    /// Select the action buttons (A/B/Select/Start).
    pub select_buttons: bool,
    #[bits(2)]
    __: u8,
}

/// Joypad: write selects the button or d-pad group, read returns its state.
pub const JOYP: VolAddress<Joypad, Safe, Safe> = unsafe { VolAddress::new(0xFF00) };

const _: () = {
    assert!(Joypad::new().with_select_buttons(true).into_bits() == 0b0010_0000);
    assert!(Joypad::new().with_p10(true).into_bits() == 0b0000_0001);
};
