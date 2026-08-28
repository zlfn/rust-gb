//! The controller registers behind the window.
//!
//! Writes to the ROM address range; none of them can be read back.

use core::ptr::write_volatile;

const RAMG: *mut u8 = 0x0000 as *mut u8;
const RAMB: *mut u8 = 0x4000 as *mut u8;
#[cfg(any(gb_pak_rtc, all(gb_pak_mbc = "mbc1", gb_pak_sram_banks = "4")))]
const MODE: *mut u8 = 0x6000 as *mut u8;

#[cfg(all(
    gb_pak_sram,
    not(any(gb_pak_sram_banks = "4", gb_pak_sram_banks = "8", gb_pak_sram_banks = "16"))
))]
pub const BANKS: u8 = 1;
#[cfg(all(gb_pak_sram, gb_pak_sram_banks = "4"))]
pub const BANKS: u8 = 4;
#[cfg(all(gb_pak_sram, gb_pak_sram_banks = "8"))]
pub const BANKS: u8 = 8;
#[cfg(all(gb_pak_sram, gb_pak_sram_banks = "16"))]
pub const BANKS: u8 = 16;
#[cfg(not(gb_pak_sram))]
pub const BANKS: u8 = 0;

/// The rumble bit shares the bank register, which cannot be read back, so its
/// last value is kept here.
#[cfg(gb_pak_rumble)]
mod shadow {
    use super::*;
    use core::cell::UnsafeCell;

    pub(super) struct Cell(UnsafeCell<u8>);
    // The Game Boy runs one thread; an interrupt handler is the only other reader.
    unsafe impl Sync for Cell {}

    pub(super) static RAMB_VALUE: Cell = Cell(UnsafeCell::new(0));

    #[inline(always)]
    pub(super) fn get() -> u8 {
        unsafe { *RAMB_VALUE.0.get() }
    }

    #[inline(always)]
    pub(super) fn put(v: u8) {
        unsafe { *RAMB_VALUE.0.get() = v };
        unsafe { write_volatile(RAMB, v) };
    }
}

/// The bank register's non-bank bit.
#[cfg(gb_pak_rumble)]
pub const RUMBLE: u8 = 0x08;

// The motor takes the bit that bank 8 and up would need.
#[cfg(all(gb_pak_rumble, gb_pak_sram_banks = "16"))]
compile_error!(
    "rumble takes the bank register's fourth bit, so a cartridge with it reaches \
     8 SRAM banks at most. `ram_size` in header.toml asks for 16"
);

#[inline(always)]
pub fn enable() {
    unsafe { write_volatile(RAMG, 0x0A) };
}

#[inline(always)]
pub fn disable() {
    unsafe { write_volatile(RAMG, 0x00) };
}

// MBC1 spends the same two register bits on ROM banks above 512 KiB and on SRAM
// banks, so a cartridge can bank one or the other.
#[cfg(all(gb_pak_mbc = "mbc1", gb_pak_sram_banks = "4", gb_wide_bank = "mbc1"))]
compile_error!(
    "MBC1 cannot bank both ROM above 512 KiB and SRAM. In header.toml, either turn \
     off `wide_banks` or set `ram_size` to 0x02, a single 8 KiB bank"
);

/// Map `bank` into the window.
///
/// On MBC1 the bank number is the same two bits that extend the ROM bank, and
/// reaches the RAM only once banking mode 1 is set. A ROM of 512 KiB or less
/// leaves those bits unwired, which is what makes the mode safe to set.
///
/// A cartridge with one bank still writes when something else can select into
/// the window, since reading `rtc` leaves it pointing at an rtc register.
#[inline(always)]
pub fn select(bank: u8) {
    if BANKS > 1 || cfg!(any(gb_pak_rtc, gb_pak_tilt)) {
        #[cfg(all(gb_pak_mbc = "mbc1", gb_pak_sram_banks = "4"))]
        unsafe {
            write_volatile(MODE, 0x01)
        };

        #[cfg(gb_pak_rumble)]
        shadow::put((shadow::get() & RUMBLE) | bank);
        #[cfg(not(gb_pak_rumble))]
        unsafe {
            write_volatile(RAMB, bank)
        };
    }
}

/// Show something other than RAM in the window: rtc registers, sensor.
#[cfg(any(gb_pak_rtc, gb_pak_tilt))]
#[inline(always)]
pub fn select_raw(value: u8) {
    unsafe { write_volatile(RAMB, value) };
}

/// Drive the motor, keeping the bank bits.
#[cfg(gb_pak_rumble)]
#[inline(always)]
pub fn set_rumble(on: bool) {
    let kept = shadow::get() & !RUMBLE;
    shadow::put(if on { kept | RUMBLE } else { kept });
}

/// Latch the clock: the transition, not the value, is what takes the snapshot.
#[cfg(gb_pak_rtc)]
#[inline(always)]
pub fn latch() {
    unsafe { write_volatile(MODE, 0x00) };
    unsafe { write_volatile(MODE, 0x01) };
}
