#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(asm_experimental_arch)]

const GRAPHICS_WIDTH: u8 = 160;
const GRAPHICS_HEIGHT: u8 = 144;

const WHITE: u8 = 0;
const LTGREY: u8 = 1;
const DKGREY: u8 = 2;
const BLACK: u8 = 3;

const SIGNED: u8 = 1;
const UNSIGNED: u8 = 0;

const M_NOFILL: u8 = 0;
const M_FILL: u8 = 1;

const SOLID: u8 = 0x00;
const OR: u8 = 0x01;
const XOR: u8 = 0x02;
const AND: u8 = 0x03;

extern "C" {
    fn delay(delay: u16);
    #[link_name="circle __sdcccall(0)"]
    fn circle(x: u8, y: u8, radius: u8, style: u8);
    #[link_name="color __sdcccall(0)"]
    fn color(forecolor: u8, backcolor: u8, mode: u8);
}

#[no_mangle]
pub extern fn main() {
    let mut x = 20;
    let mut y;
    unsafe {
        color(BLACK, WHITE, SOLID);
        while x < GRAPHICS_WIDTH - 15 {
            y = 20;
            while y < GRAPHICS_HEIGHT - 15 {
                circle(x, y, 15, M_NOFILL);
                y += 10;
            }
            x += 10;
        }
    }
}

#[no_mangle]
pub fn add(left: u32, right: u32) -> u32 {
    left * right
}

#[allow(unconditional_recursion)]
#[cfg(not(test))]
#[panic_handler]
fn panic_handler_phony(info: &core::panic::PanicInfo) -> ! {
    panic_handler_phony(info)
}
