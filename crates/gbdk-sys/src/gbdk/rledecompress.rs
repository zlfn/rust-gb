// gbdk/rledecompress.h bindings — RLE decompression

pub const RLE_STOP: u8 = 0;

unsafe extern "C" {
    pub fn rle_init(data: *const u8) -> u8;
    pub fn rle_decompress(dest: *mut u8, len: u8) -> u8;
}
