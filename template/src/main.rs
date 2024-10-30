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
    let mut mem: [u8; 8] = [0; 8];
    loop {
        unsafe { vsync() };
        if Joypad::a() {
            if mem[0] == 0 {
                println!("a");
                mem[0] = 1;
            }
        }
        else {
            mem[0] = 0;
        }

        if Joypad::b() {
            if mem[1] == 0 {
                println!("b");
                mem[1] = 1;
            }
        }
        else {
            mem[1] = 0;
        }

        if Joypad::select() {
            if mem[2] == 0 {
                println!("select");
                mem[2] = 1;
            }
        }
        else {
            mem[2] = 0;
        }

        if Joypad::start() {
            if mem[3] == 0 {
                println!("start");
                mem[3] = 1;
            }
        }
        else {
            mem[3] = 0;
        }

        if Joypad::up() {
            if mem[4] == 0 {
                println!("up");
                mem[4] = 1;
            }
        }
        else {
            mem[4] = 0;
        }

        if Joypad::down() {
            if mem[5] == 0 {
                println!("down");
                mem[5] = 1;
            }
        }
        else {
            mem[5] = 0;
        }

        if Joypad::left() {
            if mem[6] == 0 {
                println!("left");
                mem[6] = 1;
            }
        }
        else {
            mem[6] = 0;
        }

        if Joypad::right() {
            if mem[7] == 0 {
                println!("right");
                mem[7] = 1;
            }
        }
        else {
            mem[7] = 0;
        }
    }
}

#[panic_handler]
#[allow(unused_variables)]
pub fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    loop {}
}
