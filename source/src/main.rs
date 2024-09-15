#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(asm_experimental_arch)]

mod gbdk;

use core::ffi::c_char;

use gbdk::{gb::drawing::gprint, rand::{initarand, rand}};

#[no_mangle]
pub extern fn main() {
    unsafe {
        initarand(0);

        let mut array: [u8; 10] = [0; 10]; 

        loop {
            let r = rand();
            if r < 10 {
                array[r as usize] += 1;
                gprint([array[r as usize], 0].as_ptr() as *const c_char);
                if array[r as usize] == 100 {
                    break;
                }
            }
        }
    }
}

#[allow(unconditional_recursion)]
#[cfg(not(test))]
#[panic_handler]
fn panic_handler_phony(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
