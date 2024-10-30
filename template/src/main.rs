#![no_std]
#![no_main]
#![allow(dead_code)]
#![allow(static_mut_refs)]
#![feature(strict_provenance)]
#![feature(format_args_nl)]

use gb::{gbdk_c::gb::gb::vsync, io::Joypad, println};

#[no_mangle]
pub extern fn main() {
    println!("Hello, Rust-GB!");
    loop {
        unsafe { vsync() };
        let pad = Joypad::read();
        if pad.left() {
            println!("left");
        }
        if pad.right() {
            println!("right");
        }
        if pad.up() {
            println!("up");
        }
        if pad.down() {
            println!("down");
        }
    }
}

#[panic_handler]
#[allow(unused_variables)]
pub fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    loop {}
}
