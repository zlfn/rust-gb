//! Serial transfer (`SB`, `SC`).

use bitfield_struct::bitfield;
use voladdress::{Safe, VolAddress};

/// Serial transfer control (`SC`).
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7   | `transfer_enable` | R/W | Transfer requested / in progress. |
/// | 6-2 | —                 |     | Unused. |
/// | 1   | `clock_speed`     | R/W | Fast clock (CGB only). |
/// | 0   | `clock_select`    | R/W | Use the internal clock (this Game Boy drives the transfer). |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct SerialCtrl {
    /// Use the internal clock (this Game Boy drives the transfer).
    pub clock_select: bool,
    /// Fast clock (CGB only).
    pub clock_speed: bool,
    #[bits(5)]
    __: u8,
    /// Transfer requested / in progress.
    pub transfer_enable: bool,
}

/// Serial transfer data (the byte shifted in/out).
pub const SB: VolAddress<u8, Safe, Safe> = unsafe { VolAddress::new(0xFF01) };
/// Serial transfer control.
pub const SC: VolAddress<SerialCtrl, Safe, Safe> = unsafe { VolAddress::new(0xFF02) };

const _: () = {
    assert!(SerialCtrl::new().with_transfer_enable(true).into_bits() == 0b1000_0000);
    assert!(SerialCtrl::new().with_clock_select(true).into_bits() == 0b0000_0001);
};
