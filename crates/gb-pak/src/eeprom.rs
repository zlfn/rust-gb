//! MBC7's save storage: 128 words on a serial EEPROM.
//!
//! MBC7 carries no SRAM. In its place is a 93LC56 chip reached one bit at a
//! time through a single port, so this is a driver rather than a window: there is
//! nothing to map and no [`sram`](mod@crate::sram) scope to open.
//!
//! Words are 16 bits and addressed 0 to 127, giving 256 bytes. A write is slow:
//! the command goes out one bit at a time and the chip then takes time of its
//! own, so write a save in one pass rather than a field at a time.
//!
//! The chip starts locked. [`unlock`] allows writes and [`lock`] refuses them
//! again.

use core::ptr::{read_volatile, write_volatile};

use crate::{WINDOW, reg};

const PORT: *mut u8 = (WINDOW + 0x80) as *mut u8;

const CS: u8 = 0x80;
const CLK: u8 = 0x40;
const DI: u8 = 0x02;
const DO: u8 = 0x01;

/// Words the chip holds.
pub const WORDS: u8 = 128;

fn port(bits: u8) {
    unsafe { write_volatile(PORT, bits) };
}

/// Clock one bit out, returning the bit the chip clocked back.
fn exchange(state: u8, bit: bool) -> bool {
    let level = state | if bit { DI } else { 0 };
    port(level);
    port(level | CLK);
    let read = unsafe { read_volatile(PORT as *const u8) } & DO != 0;
    port(level);
    read
}

fn command(opcode: u8, address: u8) {
    port(0);
    port(CS);
    exchange(CS, true);
    for i in (0..2).rev() {
        exchange(CS, opcode >> i & 1 != 0);
    }
    for i in (0..8).rev() {
        exchange(CS, address >> i & 1 != 0);
    }
}

fn finish() {
    port(0);
}

/// Read one word.
pub fn read(address: u8) -> u16 {
    reg::enable();
    reg::select_raw(0x40);

    command(0b10, address);
    let mut word = 0u16;
    for _ in 0..16 {
        word = word << 1 | exchange(CS, false) as u16;
    }
    finish();

    reg::disable();
    word
}

/// Write one word. Does nothing until [`unlock`].
pub fn write(address: u8, word: u16) {
    reg::enable();
    reg::select_raw(0x40);

    command(0b01, address);
    for i in (0..16).rev() {
        exchange(CS, word >> i & 1 != 0);
    }
    finish();

    // The chip holds its data line low until the write has finished.
    port(CS);
    while unsafe { read_volatile(PORT as *const u8) } & DO == 0 {}
    finish();

    reg::disable();
}

/// Allow writes.
pub fn unlock() {
    reg::enable();
    reg::select_raw(0x40);
    command(0b00, 0b1100_0000);
    finish();
    reg::disable();
}

/// Refuse writes again.
pub fn lock() {
    reg::enable();
    reg::select_raw(0x40);
    command(0b00, 0b0000_0000);
    finish();
    reg::disable();
}
