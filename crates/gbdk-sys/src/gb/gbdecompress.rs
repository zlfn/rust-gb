// gb/gbdecompress.h bindings — GB compression/decompression

unsafe extern "C" {
    pub fn gb_decompress(sour: *const u8, dest: *mut u8) -> u16;
    pub fn gb_decompress_bkg_data(first_tile: u8, sour: *const u8);
    pub fn gb_decompress_win_data(first_tile: u8, sour: *const u8);
    pub fn gb_decompress_sprite_data(first_tile: u8, sour: *const u8);
}
