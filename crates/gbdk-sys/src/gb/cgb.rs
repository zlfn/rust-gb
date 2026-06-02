// gb/cgb.h bindings — Color Game Boy functions

pub type PaletteColor = u16;

pub const fn rgb(r: u8, g: u8, b: u8) -> PaletteColor {
    ((b as u16 & 0x1f) << 10) | ((g as u16 & 0x1f) << 5) | (r as u16 & 0x1f)
}

pub const fn rgb8(r: u8, g: u8, b: u8) -> PaletteColor {
    rgb(r >> 3, g >> 3, b >> 3)
}

pub const fn rgbhtml(rgb24: u32) -> PaletteColor {
    rgb8((rgb24 >> 16) as u8, (rgb24 >> 8) as u8, rgb24 as u8)
}

pub const RGB_RED: PaletteColor = rgb(31, 0, 0);
pub const RGB_DARKRED: PaletteColor = rgb(15, 0, 0);
pub const RGB_GREEN: PaletteColor = rgb(0, 31, 0);
pub const RGB_DARKGREEN: PaletteColor = rgb(0, 15, 0);
pub const RGB_BLUE: PaletteColor = rgb(0, 0, 31);
pub const RGB_DARKBLUE: PaletteColor = rgb(0, 0, 15);
pub const RGB_YELLOW: PaletteColor = rgb(31, 31, 0);
pub const RGB_DARKYELLOW: PaletteColor = rgb(21, 21, 0);
pub const RGB_CYAN: PaletteColor = rgb(0, 31, 31);
pub const RGB_AQUA: PaletteColor = rgb(28, 5, 22);
pub const RGB_PINK: PaletteColor = rgb(31, 0, 31);
pub const RGB_PURPLE: PaletteColor = rgb(21, 0, 21);
pub const RGB_BLACK: PaletteColor = rgb(0, 0, 0);
pub const RGB_DARKGRAY: PaletteColor = rgb(10, 10, 10);
pub const RGB_LIGHTGRAY: PaletteColor = rgb(21, 21, 21);
pub const RGB_WHITE: PaletteColor = rgb(31, 31, 31);
pub const RGB_LIGHTFLESH: PaletteColor = rgb(30, 20, 15);
pub const RGB_BROWN: PaletteColor = rgb(10, 10, 0);
pub const RGB_ORANGE: PaletteColor = rgb(30, 20, 0);
pub const RGB_TEAL: PaletteColor = rgb(15, 15, 0);

unsafe extern "sdcccall-0" {
    pub fn set_bkg_palette(first_palette: u8, nb_palettes: u8, rgb_data: *const PaletteColor);
    pub fn set_sprite_palette(first_palette: u8, nb_palettes: u8, rgb_data: *const PaletteColor);
    pub fn set_bkg_palette_entry(palette: u8, entry: u8, rgb_data: u16);
    pub fn set_sprite_palette_entry(palette: u8, entry: u8, rgb_data: u16);
}

unsafe extern "C" {
    pub fn cpu_slow();
    pub fn cpu_fast();
    pub fn set_default_palette();
}
