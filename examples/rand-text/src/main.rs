#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(asm_experimental_arch)]

mod gbdk;

use core::ffi::c_char;

use gbdk::{gb::{drawing::{gotogxy, gprint, plot, plot_point, LTGREY, SOLID}, gb::{delay, waitpad, waitpadup}}, rand::{initarand, rand}};

#[no_mangle]
pub extern fn main() {
    unsafe {
        gprint("Getting seed\0".as_ptr() as *const c_char);
        gotogxy(0, 1);
        gprint("Push any key (1)\0".as_ptr() as *const c_char);
        let a: u8 = waitpad(0xFF);
        waitpadup();
        let mut seed = a as u16;
        gotogxy(0, 2);
        gprint("Push any key (2)\0".as_ptr() as *const c_char);
        let b: u8 = waitpad(0xFF);
        waitpadup();
        seed |= (b as u16) << 8;

        initarand(seed);

        loop {
            let r = rand();
            gprint([r, 0].as_ptr() as *const c_char);
        }
    }
}

#[allow(unconditional_recursion)]
#[cfg(not(test))]
#[panic_handler]
fn panic_handler_phony(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
