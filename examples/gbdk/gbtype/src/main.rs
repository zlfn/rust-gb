//! GBDK GBType — A direct port of GBDK's gbtype example.
//!
//! Ported from the original C code in gbdk-2020/examples/gb/gbtype/gbtype.c.
//! Detects the type of Game Boy hardware (DMG, MGB, CGB, SGB, GBA).

#![no_std]
#![no_main]
#![allow(unsafe_op_in_unsafe_fn)]

use gbdk_sys::gb::gb::*;
use gbdk_sys::gb::sgb::*;
use gbdk_sys::stdio::printf;

#[gb_rt::entry]
fn main() -> ! {
    unsafe {
        gbdk_sys::init();

        // A PAL SNES (SGB) needs this startup delay.
        for _ in 0..4u8 {
            vsync();
        }

        display_on();

        // sgb_check() is slow, so it's better to call it once and save the result
        let is_sgb = sgb_check();

        if is_sgb != 0 {
            if _cpu == DMG_TYPE {
                printf(c"This is a SGB 1!".as_ptr());
            } else if _cpu == MGB_TYPE {
                printf(c"This is a SGB 2!".as_ptr());
            }
        } else if _cpu == DMG_TYPE {
            printf(c"This is a DMG! (OG Game Boy)".as_ptr());
        } else if _cpu == MGB_TYPE {
            printf(c"This is a MGB! (Game Boy Pocket / Light)".as_ptr());
        } else if _cpu == CGB_TYPE {
            if _is_GBA == GBA_DETECTED {
                printf(c"This is a GBA in CGB mode!".as_ptr());
            } else {
                printf(c"This is a CGB! (Game Boy Color)".as_ptr());
            }
        }

        loop {
            vsync();
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
