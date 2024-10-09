#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(strict_provenance)]

mod gb;

use gb::drawing::{print::{cursor, print}, DmgColor, DrawingStyle};

#[no_mangle]
pub extern fn main() {
    print("TE\nST\0");

    cursor(0, 17);
    print(u8::MAX);

    DrawingStyle::default()
        .foreground(DmgColor::DarkGrey)
        .apply();
    print(i8::MIN);

    DrawingStyle::reversed().apply();
    print(u16::MAX);

    DrawingStyle::default().apply();
    print(i16::MIN);
}

#[allow(unconditional_recursion)]
#[panic_handler]
#[cfg(not(test))]
fn panic_handler_phony(info: &core::panic::PanicInfo) -> ! {
    loop {}
}
