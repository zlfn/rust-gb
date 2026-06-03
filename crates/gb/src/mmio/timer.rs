//! Divider and timer (`DIV`, `TIMA`, `TMA`, `TAC`).

use bitfield_struct::{bitenum, bitfield};
use voladdress::{Safe, VolAddress};

/// Timer input clock: the `TAC` clock-select field (bits 1-0). The four values
/// are mutually exclusive, so this is an enum, not a set of flags.
#[bitenum(all = false)]
#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum TimerClock {
    /// 4096 Hz.
    Hz4096 = 0b00,
    /// 262144 Hz.
    Hz262144 = 0b01,
    /// 65536 Hz.
    Hz65536 = 0b10,
    /// 16384 Hz.
    #[fallback]
    Hz16384 = 0b11,
}

/// Timer control (`TAC`): the enable flag plus the 2-bit clock select.
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct TimerCtrl {
    /// Input clock that drives TIMA.
    #[bits(2)]
    pub clock: TimerClock,
    /// Run the timer (TIMA increments).
    pub enable: bool,
    #[bits(5)]
    __: u8,
}

/// Divider: reads the upper byte of the divider, any write resets it to 0.
pub const DIV: VolAddress<u8, Safe, Safe> = unsafe { VolAddress::new(0xFF04) };
/// Timer counter.
pub const TIMA: VolAddress<u8, Safe, Safe> = unsafe { VolAddress::new(0xFF05) };
/// Timer modulo (reload value on overflow).
pub const TMA: VolAddress<u8, Safe, Safe> = unsafe { VolAddress::new(0xFF06) };
/// Timer control: enable and input clock select.
pub const TAC: VolAddress<TimerCtrl, Safe, Safe> = unsafe { VolAddress::new(0xFF07) };

const _: () = {
    assert!(TimerCtrl::new().with_enable(true).into_bits() == 0b0000_0100);
    assert!(TimerCtrl::new().with_clock(TimerClock::Hz16384).into_bits() == 0b0000_0011);
};
