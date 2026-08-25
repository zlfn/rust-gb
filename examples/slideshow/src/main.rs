//! Slideshow — Cycle through full-screen images with any button press.
//!
//! Generated with:
//!   gb-image-fx ~/Downloads/korea.png --quantize 160x144 --dither 1.0 --tiles-only -o res/korea1
//!   gb-image-fx ~/Downloads/korea2.png --quantize 160x144 --dither 1.0 --tiles-only -o res/korea2
//!   gb-image-fx ~/Downloads/korea3.jpg --quantize 160x144 --dither 1.0 --tiles-only -o res/korea3
//!   gb-image-fx ~/Downloads/korea4.webp --quantize 160x144 --dither 1.0 --tiles-only -o res/korea4

#![no_std]
#![no_main]

use gbdk_sys::gb::gb::*;
use gbdk_sys::gb::cgb::*;
use gbdk_sys::gb::drawing::*;

struct Slide {
    tiles: &'static [u8],
    palettes: &'static [u8],
    attributes: &'static [u8],
}

static SLIDES: &[Slide] = &[
    Slide {
        tiles: include_bytes!("../res/korea1_tiles.bin"),
        palettes: include_bytes!("../res/korea1_palettes.bin"),
        attributes: include_bytes!("../res/korea1_attributes.bin"),
    },
    Slide {
        tiles: include_bytes!("../res/korea2_tiles.bin"),
        palettes: include_bytes!("../res/korea2_palettes.bin"),
        attributes: include_bytes!("../res/korea2_attributes.bin"),
    },
    Slide {
        tiles: include_bytes!("../res/korea3_tiles.bin"),
        palettes: include_bytes!("../res/korea3_palettes.bin"),
        attributes: include_bytes!("../res/korea3_attributes.bin"),
    },
    Slide {
        tiles: include_bytes!("../res/korea4_tiles.bin"),
        palettes: include_bytes!("../res/korea4_palettes.bin"),
        attributes: include_bytes!("../res/korea4_attributes.bin"),
    },
];

const NUM_PALETTES: u8 = 8;

fn show_slide(slide: &Slide) {
    unsafe {
        // Black out the palettes so the image draw stays hidden.
        if _cpu == CGB_TYPE {
            let black: [PaletteColor; 4] = [RGB_BLACK; 4];
            for i in 0..NUM_PALETTES {
                set_bkg_palette(i, 1, black.as_ptr());
            }
        } else {
            BGP_REG.write(dmg_palette(DMG_BLACK, DMG_BLACK, DMG_BLACK, DMG_BLACK));
        }

        // draw_image switches to APA graphics mode.
        draw_image(slide.tiles.as_ptr() as *mut u8);

        if _cpu == CGB_TYPE {
            set_bkg_attributes(0, 0, 20, 18, slide.attributes.as_ptr());
        }

        show_bkg();

        // Reveal palettes on next frame
        vsync();
        if _cpu == CGB_TYPE {
            set_bkg_palette(0, NUM_PALETTES,
                slide.palettes.as_ptr() as *const PaletteColor);
        } else {
            BGP_REG.write(dmg_palette(DMG_WHITE, DMG_LITE_GRAY, DMG_DARK_GRAY, DMG_BLACK));
        }
    }
}

#[gb_rt::entry]
fn main() -> ! {
    unsafe {
        gbdk_sys::init();

        let mut current: usize = 0;
        show_slide(&SLIDES[current]);

        loop {
            vsync();

            let keys = joypad();
            if keys != 0 {
                current = (current + 1) % SLIDES.len();
                show_slide(&SLIDES[current]);

                // Wait for button release to avoid rapid cycling
                waitpadup();
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
