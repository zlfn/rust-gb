//! GBDK GBType — A direct port of GBDK's gbtype example.
//!
//! Ported from the original C code in gbdk-2020/examples/gb/gbtype/gbtype.c.
//! Detects the type of Game Boy hardware (DMG, MGB, CGB, SGB, GBA).

#![no_std]
#![no_main]
#![allow(unsafe_op_in_unsafe_fn)]

use core::ffi::c_char;
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
                printf(b"This is a SGB 1!\0".as_ptr() as *const c_char);
            } else if _cpu == MGB_TYPE {
                printf(b"This is a SGB 2!\0".as_ptr() as *const c_char);
            }
        } else if _cpu == DMG_TYPE {
            printf(b"This is a DMG! (OG Game Boy)\0".as_ptr() as *const c_char);
        } else if _cpu == MGB_TYPE {
            printf(b"This is a MGB! (Game Boy Pocket / Light)\0".as_ptr() as *const c_char);
        } else if _cpu == CGB_TYPE {
            if _is_GBA == GBA_DETECTED {
                printf(b"This is a GBA in CGB mode!\0".as_ptr() as *const c_char);
            } else {
                printf(b"This is a CGB! (Game Boy Color)\0".as_ptr() as *const c_char);
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
