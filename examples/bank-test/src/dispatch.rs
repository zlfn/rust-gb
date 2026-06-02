//! Banked callbacks for heterogeneous dispatch. The resident loop collects these
//! with `far!(..).erase()` into one `[DynFar<fn(u8) -> u8>; N]` table and calls
//! them by runtime index.

use gb_bank::*;

bank::module!();

#[bank]
pub fn add(x: u8) -> u8 {
    x.wrapping_add(0x11)
}

#[bank]
pub fn xor(x: u8) -> u8 {
    x ^ 0x55
}

#[bank]
pub fn mul(x: u8) -> u8 {
    x.wrapping_mul(3)
}
