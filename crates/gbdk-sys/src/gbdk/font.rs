// gbdk/font.h bindings — font handling

pub type FontT = u16;

pub const FONT_256ENCODING: u8 = 0;
pub const FONT_128ENCODING: u8 = 1;
pub const FONT_NOENCODING: u8 = 2;
pub const FONT_COMPRESSED: u8 = 4;

unsafe extern "C" {
    pub static font_spect: [u8; 0];
    pub static font_italic: [u8; 0];
    pub static font_ibm: [u8; 0];
    pub static font_min: [u8; 0];
    pub static font_ibm_fixed: [u8; 0];

    pub fn font_init();
}

unsafe extern "sdcccall-0" {
    pub fn font_load(font: *const u8) -> FontT;
    pub fn font_set(font_handle: FontT) -> FontT;
    pub fn font_color(forecolor: u8, backcolor: u8);
}
