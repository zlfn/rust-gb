#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(strict_provenance)]

mod gb;

use gb::drawing::{gb_print, DrawingStyle, DmgColor};

#[no_mangle]
pub extern fn main() {
    gb_print("TEST\0");

    DrawingStyle::default()
        .foreground(DmgColor::DarkGrey)
        .apply();
    gb_print("TEST\0");

    DrawingStyle::default()
        .foreground(DmgColor::White)
        .background(DmgColor::Black)
        .apply();
    gb_print("TEST\0");
}

#[allow(unconditional_recursion)]
#[panic_handler]
#[cfg(not(test))]
fn panic_handler_phony(info: &core::panic::PanicInfo) -> ! {
    loop {}
}
