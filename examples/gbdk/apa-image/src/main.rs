//! GBDK APA Image — Display a full-screen image using All Points Addressable mode.
//!
//! Tile data generated from scenery.png using gb-image-fx:
//!   gb-image-fx res/scenery.png --tiles-only -o res/scenery

#![no_std]
#![no_main]

use gbdk_sys::gb::gb::*;
use gbdk_sys::gb::cgb::*;
use gbdk_sys::gb::drawing::*;

static SCENERY_TILES: &[u8] = include_bytes!("../res/scenery_tiles.bin");
static SCENERY_PALETTES: &[u8] = include_bytes!("../res/scenery_palettes.bin");

const CGB_BKG_PAL_0: u8 = 0;
const CGB_ONE_PAL: u8 = 1;
const CGB_PAL_BLACK: [PaletteColor; 4] = [RGB_BLACK, RGB_BLACK, RGB_BLACK, RGB_BLACK];

#[gb_rt::entry]
fn main() -> ! {
    unsafe {
        gbdk_sys::init();

        // Set the screen to black via the palettes to hide the image draw
        if _cpu == CGB_TYPE {
            set_bkg_palette(CGB_BKG_PAL_0, CGB_ONE_PAL, CGB_PAL_BLACK.as_ptr());
        } else {
            BGP_REG.write(dmg_palette(DMG_BLACK, DMG_BLACK, DMG_BLACK, DMG_BLACK));
        }

        // Display the image
        // This will automatically switch to APA graphics mode
        // and install it's start and mid-frame ISRs.
        draw_image(SCENERY_TILES.as_ptr() as *mut u8);
        show_bkg();

        // Then load the palettes at the start of a new frame
        vsync();
        if _cpu == CGB_TYPE {
            set_bkg_palette(CGB_BKG_PAL_0, CGB_ONE_PAL,
                SCENERY_PALETTES.as_ptr() as *const PaletteColor);
        } else {
            BGP_REG.write(dmg_palette(DMG_WHITE, DMG_LITE_GRAY, DMG_DARK_GRAY, DMG_BLACK));
        }

        // Loop forever
        loop {
            // Main processing goes here
            // Done processing, yield CPU and wait for start of next frame
            vsync();
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
