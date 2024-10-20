#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(strict_provenance)]
#![feature(format_args_nl)]

use core::fmt::Write;

use gb::{drawing::{DmgColor, DrawingStream, DrawingStyle}, io::GbStream};

mod gb;

#[no_mangle]
pub extern fn main() {
    GbStream::clear();
    for n in 0..8 as u8 {
        println!("PLUS{}", n);
    }
}

#[panic_handler]
#[allow(unused_variables)]
pub fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    let mut w = unsafe {DrawingStream::new()};
    DrawingStyle::reversed().apply(&w);
    write!(w, "PANIC occured");
    
    w.cursor(0, 1);
    DrawingStyle::reversed().foreground(DmgColor::LightGrey).apply(&w);
    write!(w, "{}", info.message());

    loop {}
}
