//! Banked color: a struct return plus an inherent method.

use gb_bank::*;

bank::module!(3);

pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[bank]
impl Rgb {
    pub fn brightness(&self) -> u8 {
        let sum = self.r as u16 + self.g as u16 + self.b as u16;
        (sum / 3) as u8
    }
}

#[bank]
pub fn make_rgb(r: u8, g: u8, b: u8) -> Rgb {
    Rgb { r, g, b }
}
