//! Banked arithmetic: free functions and a generic, living in their own bank.

use gb_bank::*;

bank::module!(1);

#[bank]
pub fn add(a: u8, b: u8) -> u8 {
    a.wrapping_add(b)
}

#[bank]
pub fn mul(a: u8, b: u8) -> u8 {
    a.wrapping_mul(b)
}

/// Generic clamp: every instantiation is banked independently.
#[bank]
pub fn clamp<T: PartialOrd + Copy + BankSafe>(val: T, min: T, max: T) -> T {
    if val < min {
        min
    } else if val > max {
        max
    } else {
        val
    }
}
