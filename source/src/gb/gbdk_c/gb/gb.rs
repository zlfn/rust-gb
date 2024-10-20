pub const SYSTEM_60HZ: u8 = 0x00;
pub const SYSTEM_50HZ: u8 = 0x01;

extern "C" {
    #[link_name="delay"]
    pub fn delay(delay: u16);
    #[link_name="waitpad __preserves_regs(b, c, h, l)"]
    pub fn waitpad(mask: u8) -> u8;
    #[link_name="waitpadup __preserves_regs(a, b, c, d, e, h, l)"]
    pub fn waitpadup();
}
