#[allow(non_camel_case_types)]
pub type font_t = *const u8;

pub const FONT_256ENCODING: u8 = 0;
pub const FONT_128ENCODING: u8 = 1;
pub const FONT_NOENCODING: u8 = 2;
pub const FONT_COMPRESSED: u8 = 4;

extern "C" {
    pub static font_spect: *const u8;
    pub static font_italic: *const u8;
    pub static font_ibm: *const u8;
    pub static font_min: *const u8;
    pub static font_ibm_fixed: *const u8;

    pub fn font_init();

    #[link_name="font_load __sdcccall(0)"]
    pub fn font_load(font: font_t) -> font_t;

    #[link_name="font_set __sdcccall(0)"]
    pub fn font_set(font: font_t) -> font_t;

    #[link_name="font_color __sdcccall(0)"]
    pub fn font_color(forecolor: u8, backcolor: u8);
}
