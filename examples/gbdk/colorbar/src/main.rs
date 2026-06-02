//! GBDK Colorbar — A direct port of GBDK's colorbar example.
//!
//! Ported from the original C code in gbdk-2020/examples/gb/colorbar/colorbar.c.

#![no_std]
#![no_main]

mod bar_c;
mod bar_m;

use bar_c::*;
use gbdk_sys::gb::cgb::*;
use gbdk_sys::gb::gb::*;

#[rustfmt::skip]
static BAR_P: [PaletteColor; 32] = [
    CGB_PAL0C0, CGB_PAL0C1, CGB_PAL0C2, CGB_PAL0C3,
    CGB_PAL1C0, CGB_PAL1C1, CGB_PAL1C2, CGB_PAL1C3,
    CGB_PAL2C0, CGB_PAL2C1, CGB_PAL2C2, CGB_PAL2C3,
    CGB_PAL3C0, CGB_PAL3C1, CGB_PAL3C2, CGB_PAL3C3,
    CGB_PAL4C0, CGB_PAL4C1, CGB_PAL4C2, CGB_PAL4C3,
    CGB_PAL5C0, CGB_PAL5C1, CGB_PAL5C2, CGB_PAL5C3,
    CGB_PAL6C0, CGB_PAL6C1, CGB_PAL6C2, CGB_PAL6C3,
    CGB_PAL7C0, CGB_PAL7C1, CGB_PAL7C2, CGB_PAL7C3,
];

#[rustfmt::skip]
static BAR_A: [u8; 360] = [
    7,7,4,4,4,6,6,6,2,2,2,5,5,5,1,1,1,3,3,3,
    7,7,4,4,4,6,6,6,2,2,2,5,5,5,1,1,1,3,3,3,
    7,7,4,4,4,6,6,6,2,2,2,5,5,5,1,1,1,3,3,3,
    7,7,4,4,4,6,6,6,2,2,2,5,5,5,1,1,1,3,3,3,
    7,7,4,4,4,6,6,6,2,2,2,5,5,5,1,1,1,3,3,3,
    7,7,4,4,4,6,6,6,2,2,2,5,5,5,1,1,1,3,3,3,
    7,7,4,4,4,6,6,6,2,2,2,5,5,5,1,1,1,3,3,3,
    7,7,4,4,4,6,6,6,2,2,2,5,5,5,1,1,1,3,3,3,
    7,7,4,4,4,6,6,6,2,2,2,5,5,5,1,1,1,3,3,3,
    7,7,4,4,4,6,6,6,2,2,2,5,5,5,1,1,1,3,3,3,
    7,7,4,4,4,6,6,6,2,2,2,5,5,5,1,1,1,3,3,3,
    7,7,4,4,4,6,6,6,2,2,2,5,5,5,1,1,1,3,3,3,
    7,7,4,4,4,6,6,6,2,2,2,5,5,5,1,1,1,3,3,3,
    7,7,4,4,4,6,6,6,2,2,2,5,5,5,1,1,1,3,3,3,
    3,3,0,0,0,5,5,5,0,0,0,6,6,6,0,0,0,7,7,7,
    3,3,3,3,0,0,0,0,5,5,5,5,0,0,0,0,0,0,0,0,
    3,3,3,3,0,0,0,0,5,5,5,5,0,0,0,0,0,0,0,0,
    3,3,3,3,0,0,0,0,5,5,5,5,0,0,0,0,0,0,0,0,
];

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    unsafe {
        gbdk_sys::init();

        // Transfer color palettes
        set_bkg_palette(7, 1, BAR_P[0..].as_ptr());
        set_bkg_palette(6, 1, BAR_P[4..].as_ptr());
        set_bkg_palette(5, 1, BAR_P[8..].as_ptr());
        set_bkg_palette(4, 1, BAR_P[12..].as_ptr());
        set_bkg_palette(3, 1, BAR_P[16..].as_ptr());
        set_bkg_palette(2, 1, BAR_P[20..].as_ptr());
        set_bkg_palette(1, 1, BAR_P[24..].as_ptr());
        set_bkg_palette(0, 1, BAR_P[28..].as_ptr());

        // CHR code transfer
        set_bkg_data(0, 32, bar_c::TILES.as_ptr());

        // Select VRAM bank 1, set attributes
        VBK_REG.write(VBK_ATTRIBUTES);
        set_bkg_tiles(0, 0, bar_m::WIDTH, bar_m::HEIGHT, BAR_A.as_ptr());

        // Select VRAM bank 0, set tile map
        VBK_REG.write(VBK_TILES);
        set_bkg_tiles(0, 0, bar_m::WIDTH, bar_m::HEIGHT, bar_m::MAP.as_ptr());

        show_bkg();
        enable_interrupts();
        display_on();
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
