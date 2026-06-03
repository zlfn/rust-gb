//! Nested child module: its own bank group, with a banked lookup table.

use gb_bank::*;

bank::inherit!();

#[bank]
static SINE_TABLE: [u8; 64] = [
    0, 6, 12, 18, 25, 31, 37, 43, 49, 54, 60, 65, 71, 76, 81, 85,
    90, 94, 98, 102, 106, 109, 112, 115, 117, 120, 122, 123, 125, 126, 126, 127,
    127, 127, 126, 126, 125, 123, 122, 120, 117, 115, 112, 109, 106, 102, 98, 94,
    90, 85, 81, 76, 71, 65, 60, 54, 49, 43, 37, 31, 25, 18, 12, 6,
];

/// 8-bit sine approximation (0-127) for angle 0-255.
#[bank]
pub fn sin8(angle: u8) -> u8 {
    let idx = (angle & 0x3F) as usize;
    // Same-bank data, so read it directly with `.get` (no closure, no switch).
    let t = SINE_TABLE.local();
    match angle >> 6 {
        0 => t[idx],
        1 => t[63 - idx],
        2 => t[idx],
        _ => t[63 - idx],
    }
}

/// Linear interpolation: a + (b - a) * t / 256.
#[bank]
pub fn lerp8(a: u8, b: u8, t: u8) -> u8 {
    let a16 = a as u16;
    let b16 = b as u16;
    let t16 = t as u16;
    (a16 + ((b16.wrapping_sub(a16)).wrapping_mul(t16) >> 8)) as u8
}
