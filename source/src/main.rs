#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(strict_provenance)]
#![feature(format_args_nl)]

mod gb;

#[no_mangle]
pub extern fn main() {
    println!("Hello, Rust-GB!");
}

#[panic_handler]
#[allow(unused_variables)]
pub fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    println!("PANIC occured");

    // If you want to see more information, uncomment below line.
    // println!("{}", info.message());

    loop {}
}
