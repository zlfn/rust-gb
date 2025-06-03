#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(asm_experimental_arch)]

use core::ffi::c_char;

use gb::{gbdk_c::{gb::{drawing::{gotogxy, gprint, plot, DKGREY, LTGREY, SOLID}, gb::{waitpad, waitpadup}}, rand::{arand, initarand, rand}, stdio::puts}, mmio::DIV};

#[no_mangle]
pub extern "C" fn main() {
    let mut accu: [u8; 80] = [0; 80];
    let mut accua: [u8; 80] = [0; 80];

    unsafe {
        gprint("Getting seed\0".as_ptr() as *const c_char);
        gotogxy(0, 1);
        puts("Push any key (1)\0".as_ptr() as *const c_char);
        waitpad(0xFF);
        waitpadup();
        let a: u8 = DIV.read();
        let mut seed = a as u16;
        gotogxy(0, 2);
        puts("Push any key (2)\0".as_ptr() as *const c_char);
        waitpad(0xFF);
        waitpadup();
        let b: u8 = DIV.read();
        seed |= (b as u16) << 8;
        initarand(seed);

        loop {
            let r = rand();
            let s = arand();

            if r < 80 {
                accu[r as usize] += 1;
                plot(r, 144 - accu[r as usize], LTGREY, SOLID);
                if accu[r as usize] == 144 {
                    break;
                }
            }

            if s < 80 {
                accua[s as usize] += 1;
                plot(s + 80, 144 - accua[s as usize], DKGREY, SOLID);
                if accua[s as usize] == 144 {
                    break;
                }
            }
        }
    }
}

#[allow(unconditional_recursion)]
#[panic_handler]
#[cfg(not(test))]
fn panic_handler_phony(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
