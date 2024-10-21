#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(strict_provenance)]
#![feature(format_args_nl)]

mod gb;

#[no_mangle]
pub extern fn main() {
    for i in 0..10000 as u16 {
        println!("{}", i);
    }
}

#[panic_handler]
#[allow(unused_variables)]
pub fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    /*
    let mut w = unsafe {DrawingStream::new()};
    DrawingStyle::reversed().apply(&w);
    write!(w, "PANIC occured");
    
    w.cursor(0, 1);
    DrawingStyle::reversed().foreground(DmgColor::LightGrey).apply(&w);
    write!(w, "{}", info.message());
    */

    loop {}
}
