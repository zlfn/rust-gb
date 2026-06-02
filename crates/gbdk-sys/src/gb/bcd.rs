// gb/bcd.h bindings — BCD arithmetic

pub type BCD = u32;

pub const fn bcd_hex(v: BCD) -> BCD { v }

/// Convert a decimal number to BCD encoding at compile time.
/// `make_bcd(1234)` → `0x00001234u32`
pub const fn make_bcd(mut decimal: u32) -> BCD {
    let mut result: u32 = 0;
    let mut shift = 0;
    while decimal > 0 {
        result |= (decimal % 10) << shift;
        decimal /= 10;
        shift += 4;
    }
    result
}

unsafe extern "sdcccall-0" {
    pub fn uint2bcd(i: u16, value: *mut BCD);
    pub fn bcd_add(sour: *mut BCD, value: *const BCD);
    pub fn bcd_sub(sour: *mut BCD, value: *const BCD);
    pub fn bcd2text(bcd: *const BCD, tile_offset: u8, buffer: *mut u8) -> u8;
}
