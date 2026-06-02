// gb/sgb.h bindings — Super Game Boy functions

pub const SGB_PAL_01: u8 = 0x00;
pub const SGB_PAL_23: u8 = 0x01;
pub const SGB_PAL_03: u8 = 0x02;
pub const SGB_PAL_12: u8 = 0x03;
pub const SGB_ATTR_BLK: u8 = 0x04;
pub const SGB_ATTR_LIN: u8 = 0x05;
pub const SGB_ATTR_DIV: u8 = 0x06;
pub const SGB_ATTR_CHR: u8 = 0x07;
pub const SGB_SOUND: u8 = 0x08;
pub const SGB_SOU_TRN: u8 = 0x09;
pub const SGB_PAL_SET: u8 = 0x0A;
pub const SGB_PAL_TRN: u8 = 0x0B;
pub const SGB_ATRC_EN: u8 = 0x0C;
pub const SGB_TEST_EN: u8 = 0x0D;
pub const SGB_ICON_EN: u8 = 0x0E;
pub const SGB_DATA_SND: u8 = 0x0F;
pub const SGB_DATA_TRN: u8 = 0x10;
pub const SGB_MLT_REQ: u8 = 0x11;
pub const SGB_JUMP: u8 = 0x12;
pub const SGB_CHR_TRN: u8 = 0x13;
pub const SGB_PCT_TRN: u8 = 0x14;
pub const SGB_ATTR_TRN: u8 = 0x15;
pub const SGB_ATTR_SET: u8 = 0x16;
pub const SGB_MASK_EN: u8 = 0x17;
pub const SGB_OBJ_TRN: u8 = 0x18;

unsafe extern "sdcccall-0" {
    pub fn sgb_check() -> u8;
    pub fn sgb_transfer(packet: *mut u8);
}
