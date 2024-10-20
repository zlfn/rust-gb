pub const SYSTEM_60HZ: u8 = 0x00;
pub const SYSTEM_50HZ: u8 = 0x01;

pub const M_DRAWING: u8 = 0x01;
pub const M_TEXT_OUT: u8 = 0x02;
pub const M_TEXT_INOUT: u8 = 0x03;
pub const M_NO_SCROLL: u8 = 0x04;
pub const M_NO_INTERP: u8 = 0x08;

extern "C" {
    #[link_name="delay"]
    pub fn delay(delay: u16);
    #[link_name="waitpad __preserves_regs(b, c, h, l)"]
    pub fn waitpad(mask: u8) -> u8;
    #[link_name="waitpadup __preserves_regs(a, b, c, d, e, h, l)"]
    pub fn waitpadup();

    pub fn mode(mode: u8);
}
