// gb/hblankcpy.h bindings — HBlank VRAM copy

unsafe extern "C" {
    pub fn hblank_copy_vram(sour: *const u8, count: u8);
    pub fn hblank_cpy_vram(sour: *const u8, count: u8);
    pub fn hblank_copy(dest: *mut u8, sour: *const u8, size: u16);
}
