//! Interrupt flags (`IF`, `IE`).

use bitflags::bitflags;
use voladdress::{Safe, Unsafe, VolAddress};

bitflags! {
    /// Interrupt sources. `IE` and `IF` share this type (same bit layout).
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Interrupts: u8 {
        /// VBlank: the PPU entered VBlank.
        const VBLANK = 1 << 0;
        /// LCD STAT: a configured PPU mode or LYC=LY condition.
        const LCD_STAT = 1 << 1;
        /// Timer: TIMA overflowed.
        const TIMER = 1 << 2;
        /// Serial transfer completed.
        const SERIAL = 1 << 3;
        /// Joypad: a selected input line went low.
        const JOYPAD = 1 << 4;
    }
}

/// Interrupt flag: pending interrupt requests.
///
/// # Safety
///
/// Writing can request an interrupt; with no handler installed the CPU runs
/// whatever sits at the vector.
pub const IF: VolAddress<Interrupts, Safe, Unsafe> = unsafe { VolAddress::new(0xFF0F) };
/// Interrupt enable: which interrupts may fire.
///
/// # Safety
///
/// Enabling an interrupt with no handler installed runs whatever sits at its
/// vector.
pub const IE: VolAddress<Interrupts, Safe, Unsafe> = unsafe { VolAddress::new(0xFFFF) };

const _: () = {
    assert!(Interrupts::VBLANK.bits() == 0b0000_0001);
    assert!(Interrupts::JOYPAD.bits() == 0b0001_0000);
};
