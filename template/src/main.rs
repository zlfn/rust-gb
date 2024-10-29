#![no_std]
#![no_main]
#![allow(dead_code)]
#![allow(static_mut_refs)]
#![feature(strict_provenance)]
#![feature(format_args_nl)]

use gb::{mmio::JOYP, println};

#[no_mangle]
pub extern fn main() {
    println!("Hello, Rust-GB!");
    JOYP.write(0x10);
    loop {
        println!("{}", JOYP.read());
    }
}

#[panic_handler]
#[allow(unused_variables)]
pub fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    loop {}
}
