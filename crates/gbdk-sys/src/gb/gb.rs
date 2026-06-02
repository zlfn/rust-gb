// gb/gb.h bindings — core Game Boy functions
//
// gb.h #includes hardware.h, so re-export it
pub use super::hardware::*;

// ── System constants ────────────────────────────────────────────────────────

pub const SYSTEM_60HZ: u8 = 0x00;
pub const SYSTEM_50HZ: u8 = 0x01;

// Joypad buttons
pub const J_UP: u8 = 0x04;
pub const J_DOWN: u8 = 0x08;
pub const J_LEFT: u8 = 0x02;
pub const J_RIGHT: u8 = 0x01;
pub const J_A: u8 = 0x10;
pub const J_B: u8 = 0x20;
pub const J_SELECT: u8 = 0x40;
pub const J_START: u8 = 0x80;

// Screen modes
pub const M_DRAWING: u8 = 0x01;
pub const M_TEXT_OUT: u8 = 0x02;
pub const M_TEXT_INOUT: u8 = 0x03;
pub const M_NO_SCROLL: u8 = 0x04;
pub const M_NO_INTERP: u8 = 0x08;

// Sprite property flags
pub const S_BANK: u8 = 0x08;
pub const S_PALETTE: u8 = 0x10;
pub const S_FLIPX: u8 = 0x20;
pub const S_FLIPY: u8 = 0x40;
pub const S_PRIORITY: u8 = 0x80;

// Interrupt flags
pub const EMPTY_IFLAG: u8 = 0x00;
pub const VBL_IFLAG: u8 = 0x01;
pub const LCD_IFLAG: u8 = 0x02;
pub const TIM_IFLAG: u8 = 0x04;
pub const SIO_IFLAG: u8 = 0x08;
pub const JOY_IFLAG: u8 = 0x10;

// DMG palette colors
pub const DMG_BLACK: u8 = 0x03;
pub const DMG_DARK_GRAY: u8 = 0x02;
pub const DMG_LITE_GRAY: u8 = 0x01;
pub const DMG_WHITE: u8 = 0x00;

// System type detection
pub const DMG_TYPE: u8 = 0x01;
pub const MGB_TYPE: u8 = 0xFF;
pub const CGB_TYPE: u8 = 0x11;

// IO status
pub const IO_IDLE: u8 = 0x00;
pub const IO_SENDING: u8 = 0x01;
pub const IO_RECEIVING: u8 = 0x02;
pub const IO_ERROR: u8 = 0x04;

// Hardware sprite limits
pub const MAX_HARDWARE_SPRITES: u8 = 40;
pub const HARDWARE_SPRITE_CAN_FLIP_X: u8 = 1;
pub const HARDWARE_SPRITE_CAN_FLIP_Y: u8 = 1;

// GBA detection
pub const GBA_NOT_DETECTED: u8 = 0x00;
pub const GBA_DETECTED: u8 = 0x01;

// Sprite palette helper
pub const fn s_pal(n: u8) -> u8 { n }

// DMG palette helpers
pub const fn dmg_palette(c0: u8, c1: u8, c2: u8, c3: u8) -> u8 {
    ((c3 & 0x03) << 6) | ((c2 & 0x03) << 4) | ((c1 & 0x03) << 2) | (c0 & 0x03)
}
pub const fn compat_palette(c0: u8, c1: u8, c2: u8, c3: u8) -> u8 {
    (c3 << 6) | (c2 << 4) | (c1 << 2) | c0
}

// ── LCD control macros ──────────────────────────────────────────────────────
// These correspond to C macros that modify LCDC_REG directly.

#[inline(always)]
pub unsafe fn display_on()    { LCDC_REG.write(LCDC_REG.read() | LCDCF_ON); }
// display_off is an extern function (waits for VBlank), already declared below.

#[inline(always)]
pub unsafe fn show_bkg()      { LCDC_REG.write(LCDC_REG.read() | LCDCF_BGON); }
#[inline(always)]
pub unsafe fn hide_bkg()      { LCDC_REG.write(LCDC_REG.read() & !LCDCF_BGON); }
#[inline(always)]
pub unsafe fn show_win()      { LCDC_REG.write(LCDC_REG.read() | LCDCF_WINON); }
#[inline(always)]
pub unsafe fn hide_win()      { LCDC_REG.write(LCDC_REG.read() & !LCDCF_WINON); }
#[inline(always)]
pub unsafe fn show_sprites()  { LCDC_REG.write(LCDC_REG.read() | LCDCF_OBJON); }
#[inline(always)]
pub unsafe fn hide_sprites()  { LCDC_REG.write(LCDC_REG.read() & !LCDCF_OBJON); }
#[inline(always)]
pub unsafe fn sprites_8x16()  { LCDC_REG.write(LCDC_REG.read() | LCDCF_OBJ16); }
#[inline(always)]
pub unsafe fn sprites_8x8()   { LCDC_REG.write(LCDC_REG.read() & !LCDCF_OBJ16); }

// Screen dimension aliases
pub const SCREENWIDTH: u8 = DEVICE_SCREEN_PX_WIDTH;
pub const SCREENHEIGHT: u8 = DEVICE_SCREEN_PX_HEIGHT;
pub const MINWNDPOSX: u8 = DEVICE_WINDOW_PX_OFFSET_X;
pub const MINWNDPOSY: u8 = DEVICE_WINDOW_PX_OFFSET_Y;
pub const MAXWNDPOSX: u8 = DEVICE_WINDOW_PX_OFFSET_X + DEVICE_SCREEN_PX_WIDTH - 1;
pub const MAXWNDPOSY: u8 = DEVICE_WINDOW_PX_OFFSET_Y + DEVICE_SCREEN_PX_HEIGHT - 1;

// Inline functions
#[inline(always)]
pub fn get_system() -> u8 { SYSTEM_60HZ }

#[inline(always)]
pub unsafe fn set_shadow_oam_address(address: *mut u8) {
    _shadow_OAM_base = (address as u16 >> 8) as u8;
}

#[inline(always)]
pub unsafe fn device_supports_color() -> bool { _cpu == CGB_TYPE }

// ── Types ───────────────────────────────────────────────────────────────────

pub type IntHandler = Option<unsafe extern "C" fn()>;

#[repr(C)]
pub struct JoypadsT {
    pub npads: u8,
    pub joypads: [u8; 4],
}

#[repr(C)]
pub struct OamItem {
    pub y: u8,
    pub x: u8,
    pub tile: u8,
    pub prop: u8,
}

// ── Global variables ────────────────────────────────────────────────────────

unsafe extern "C" {
    pub static _cpu: u8;
    pub static _is_GBA: u8;
    pub static mut sys_time: u16;
    pub static mut _vbl_done: u8;
    pub static _io_status: u8;
    pub static _io_in: u8;
    pub static mut _io_out: u8;
    pub static mut _current_bank: u8;
    pub static mut _current_1bpp_colors: u16;
    pub static mut _map_tile_offset: u8;
    pub static mut _submap_tile_offset: u8;
    pub static mut shadow_OAM: [OamItem; 40];
    pub static mut _shadow_OAM_base: u8;
}

// ── OLDCALL (sdcccall-0) functions ──────────────────────────────────────────

unsafe extern "sdcccall-0" {
    // Joypad
    pub fn joypad_init(npads: u8, joypads: *mut JoypadsT) -> u8;

    // Palette
    pub fn set_1bpp_colors_ex(fgcolor: u8, bgcolor: u8, mode: u8);

    // Background
    pub fn set_bkg_data(first_tile: u8, nb_tiles: u8, data: *const u8);
    pub fn set_bkg_1bpp_data(first_tile: u8, nb_tiles: u8, data: *const u8);
    pub fn get_bkg_data(first_tile: u8, nb_tiles: u8, data: *mut u8);
    pub fn set_bkg_tiles(x: u8, y: u8, w: u8, h: u8, tiles: *const u8);
    pub fn get_bkg_tiles(x: u8, y: u8, w: u8, h: u8, tiles: *mut u8);
    pub fn get_bkg_tile_xy(x: u8, y: u8) -> u8;

    // Window
    pub fn set_win_data(first_tile: u8, nb_tiles: u8, data: *const u8);
    pub fn set_win_1bpp_data(first_tile: u8, nb_tiles: u8, data: *const u8);
    pub fn get_win_data(first_tile: u8, nb_tiles: u8, data: *mut u8);
    pub fn set_win_tiles(x: u8, y: u8, w: u8, h: u8, tiles: *const u8);
    pub fn get_win_tiles(x: u8, y: u8, w: u8, h: u8, tiles: *mut u8);
    pub fn get_win_tile_xy(x: u8, y: u8) -> u8;

    // Sprites
    pub fn set_sprite_data(first_tile: u8, nb_tiles: u8, data: *const u8);
    pub fn set_sprite_1bpp_data(first_tile: u8, nb_tiles: u8, data: *const u8);
    pub fn get_sprite_data(first_tile: u8, nb_tiles: u8, data: *mut u8);

    // Tile helpers
    pub fn set_tiles(x: u8, y: u8, w: u8, h: u8, vram_addr: *mut u8, tiles: *const u8);
    pub fn get_tiles(x: u8, y: u8, w: u8, h: u8, vram_addr: *mut u8, tiles: *mut u8);
    pub fn set_tile_data(first_tile: u8, nb_tiles: u8, data: *const u8, base: u8);

    // Init / fill
    pub fn init_bkg(c: u8);
    pub fn init_win(c: u8);
    pub fn vmemset(s: *mut u8, c: u8, n: u16);
    pub fn fill_bkg_rect(x: u8, y: u8, w: u8, h: u8, tile: u8);
    pub fn fill_win_rect(x: u8, y: u8, w: u8, h: u8, tile: u8);

    // DMA
    pub fn hiramcpy(dst: u8, src: *const u8, n: u8);
}

// ── Non-OLDCALL (sdcccall-1 / default) functions ────────────────────────────

unsafe extern "C" {
    // Interrupt handlers
    pub fn add_VBL(h: IntHandler);
    pub fn add_LCD(h: IntHandler);
    pub fn add_TIM(h: IntHandler);
    pub fn add_low_priority_TIM(h: IntHandler);
    pub fn add_SIO(h: IntHandler);
    pub fn add_JOY(h: IntHandler);
    pub fn remove_VBL(h: IntHandler);
    pub fn remove_LCD(h: IntHandler);
    pub fn remove_TIM(h: IntHandler);
    pub fn remove_SIO(h: IntHandler);
    pub fn remove_JOY(h: IntHandler);
    pub fn nowait_int_handler();
    pub fn wait_int_handler();

    // System (non-inline extern functions)
    pub fn mode(m: u8);
    pub fn get_mode() -> u8;
    pub fn send_byte();
    pub fn receive_byte();
    pub fn delay(d: u16);
    pub fn joypad() -> u8;
    pub fn waitpad(mask: u8) -> u8;
    pub fn waitpadup();
    pub fn joypad_ex(joypads: *mut JoypadsT);
    pub fn set_interrupts(flags: u8);
    pub fn reset();
    pub fn vsync();
    pub fn wait_vbl_done();
    pub fn display_off();
    pub fn refresh_OAM();

    // VRAM access
    pub fn set_vram_byte(addr: *mut u8, v: u8);
    pub fn get_vram_byte(addr: *mut u8) -> u8;
    pub fn get_bkg_xy_addr(x: u8, y: u8) -> *mut u8;
    pub fn set_data(vram_addr: *mut u8, data: *const u8, len: u16);
    pub fn get_data(data: *mut u8, vram_addr: *mut u8, len: u16);
    pub fn vmemcpy(dest: *mut u8, sour: *mut u8, len: u16);

    // Background (non-OLDCALL, non-inline)
    pub fn set_bkg_tile_xy(x: u8, y: u8, t: u8) -> *mut u8;
    pub fn set_bkg_submap(x: u8, y: u8, w: u8, h: u8, map: *const u8, map_w: u8);

    // Window (non-OLDCALL, non-inline)
    pub fn set_win_tile_xy(x: u8, y: u8, t: u8) -> *mut u8;
    pub fn set_win_submap(x: u8, y: u8, w: u8, h: u8, map: *const u8, map_w: u8);
    pub fn get_win_xy_addr(x: u8, y: u8) -> *mut u8;
}

// ── Inline functions (reimplemented in Rust) ────────────────────────────────
// These are `inline` in GBDK's gb.h and have no library symbols.

#[inline(always)]
pub unsafe fn cancel_pending_interrupts() -> u8 { IF_REG.write(0); 0 }

#[inline(always)]
pub unsafe fn enable_interrupts() { core::arch::asm!("ei"); }

#[inline(always)]
pub unsafe fn disable_interrupts() { core::arch::asm!("di"); }

#[inline(always)]
pub unsafe fn set_2bpp_palette(_palette: u16) { /* no-op on DMG */ }

#[inline(always)]
pub unsafe fn set_1bpp_colors(fgcolor: u8, bgcolor: u8) { set_1bpp_colors_ex(fgcolor, bgcolor, 0); }

#[inline(always)]
pub unsafe fn set_bkg_based_tiles(x: u8, y: u8, w: u8, h: u8, tiles: *const u8, base_tile: u8) {
    _map_tile_offset = base_tile;
    set_bkg_tiles(x, y, w, h, tiles);
    _map_tile_offset = 0;
}

#[inline(always)]
pub unsafe fn set_bkg_attributes(x: u8, y: u8, w: u8, h: u8, tiles: *const u8) {
    VBK_REG.write(VBK_ATTRIBUTES);
    set_bkg_tiles(x, y, w, h, tiles);
    VBK_REG.write(VBK_TILES);
}

#[inline(always)]
pub unsafe fn set_bkg_based_submap(x: u8, y: u8, w: u8, h: u8, map: *const u8, map_w: u8, base_tile: u8) {
    _submap_tile_offset = base_tile;
    set_bkg_submap(x, y, w, h, map, map_w);
    _submap_tile_offset = 0;
}

#[inline(always)]
pub unsafe fn set_bkg_submap_attributes(x: u8, y: u8, w: u8, h: u8, map: *const u8, map_w: u8) {
    VBK_REG.write(VBK_ATTRIBUTES);
    set_bkg_submap(x, y, w, h, map, map_w);
    VBK_REG.write(VBK_TILES);
}

#[inline(always)]
pub unsafe fn set_bkg_attribute_xy(x: u8, y: u8, a: u8) -> *mut u8 {
    VBK_REG.write(VBK_ATTRIBUTES);
    let addr = set_bkg_tile_xy(x, y, a);
    VBK_REG.write(VBK_TILES);
    addr
}

#[inline(always)]
pub unsafe fn move_bkg(x: u8, y: u8) { SCX_REG.write(x); SCY_REG.write(y); }

#[inline(always)]
pub unsafe fn scroll_bkg(x: i8, y: i8) {
    SCX_REG.write(SCX_REG.read().wrapping_add(x as u8));
    SCY_REG.write(SCY_REG.read().wrapping_add(y as u8));
}

#[inline(always)]
pub unsafe fn set_win_based_tiles(x: u8, y: u8, w: u8, h: u8, tiles: *const u8, base_tile: u8) {
    _map_tile_offset = base_tile;
    set_win_tiles(x, y, w, h, tiles);
    _map_tile_offset = 0;
}

#[inline(always)]
pub unsafe fn set_win_based_submap(x: u8, y: u8, w: u8, h: u8, map: *const u8, map_w: u8, base_tile: u8) {
    _submap_tile_offset = base_tile;
    set_win_submap(x, y, w, h, map, map_w);
    _submap_tile_offset = 0;
}

#[inline(always)]
pub unsafe fn move_win(x: u8, y: u8) { WX_REG.write(x); WY_REG.write(y); }

#[inline(always)]
pub unsafe fn scroll_win(x: i8, y: i8) {
    WX_REG.write(WX_REG.read().wrapping_add(x as u8));
    WY_REG.write(WY_REG.read().wrapping_add(y as u8));
}

#[inline(always)]
unsafe fn oam_ptr(nb: u8) -> *mut OamItem {
    core::ptr::addr_of_mut!(shadow_OAM).cast::<OamItem>().add(nb as usize)
}

#[inline(always)]
pub unsafe fn set_sprite_tile(nb: u8, tile: u8) { (*oam_ptr(nb)).tile = tile; }

#[inline(always)]
pub unsafe fn get_sprite_tile(nb: u8) -> u8 { (*oam_ptr(nb)).tile }

#[inline(always)]
pub unsafe fn set_sprite_prop(nb: u8, prop: u8) { (*oam_ptr(nb)).prop = prop; }

#[inline(always)]
pub unsafe fn get_sprite_prop(nb: u8) -> u8 { (*oam_ptr(nb)).prop }

#[inline(always)]
pub unsafe fn move_sprite(nb: u8, x: u8, y: u8) {
    (*oam_ptr(nb)).y = y;
    (*oam_ptr(nb)).x = x;
}

#[inline(always)]
pub unsafe fn scroll_sprite(nb: u8, x: i8, y: i8) {
    let p = oam_ptr(nb);
    (*p).y = (*p).y.wrapping_add(y as u8);
    (*p).x = (*p).x.wrapping_add(x as u8);
}

#[inline(always)]
pub unsafe fn hide_sprite(nb: u8) { (*oam_ptr(nb)).y = 0; }

#[inline(always)]
pub unsafe fn set_native_tile_data(first_tile: u16, nb_tiles: u8, data: *const u8) {
    if first_tile < 256 {
        set_bkg_data(first_tile as u8, nb_tiles, data);
    } else {
        set_sprite_data((first_tile - 256) as u8, nb_tiles, data);
    }
}

#[inline(always)]
pub unsafe fn set_bkg_native_data(first_tile: u8, nb_tiles: u8, data: *const u8) {
    set_bkg_data(first_tile, nb_tiles, data);
}

#[inline(always)]
pub unsafe fn set_sprite_native_data(first_tile: u8, nb_tiles: u8, data: *const u8) {
    set_sprite_data(first_tile, nb_tiles, data);
}
