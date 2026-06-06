//! Port of GBDK's `hram` example: place a variable in High RAM (the `0xFF80` page
//! reached with the immediate `ldh` instructions) and print its value and address.

#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use core::ffi::{c_char, c_int, c_uint};
use gb_hram::{hram, prelude::*};
use gbdk_sys::stdio::printf;

// GBDK's `SFR my_hram_variable;`: an 8-bit cell in HRAM, read and written with the
// immediate `ldh` form.
hram! {
    static MY_HRAM_VARIABLE: u8;
}

#[gb_rt::entry]
fn main() -> ! {
    unsafe {
        gbdk_sys::init();

        MY_HRAM_VARIABLE.set(5);
        // `c_int`/`c_uint` are 16-bit on sm83, so use `%d`/`%x` (which consume a
        // 16-bit int); `%hd` would consume only one byte and misalign `%x`.
        printf(
            b"value is: %d at %x\0".as_ptr() as *const c_char,
            MY_HRAM_VARIABLE.get() as c_int,
            MY_HRAM_VARIABLE.as_ptr() as usize as c_uint,
        );
    }

    loop {}
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
