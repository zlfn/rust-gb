#![no_std]
#![no_main]
#![allow(dead_code)]
#![feature(asm_experimental_arch)]

use core::ffi::c_char;

mod gbdk;
use gbdk::gb::drawing::{r#box, circle, color, getpix, gotogxy, gprint, line, plot_point, BLACK, DKGREY, LTGREY, M_FILL, M_NOFILL, SOLID, WHITE, XOR};

fn linetest(x: u8, y: u8, w: u8) {
    let w = w as i8;
    unsafe {
        color(DKGREY, WHITE, SOLID);
        for i in -w..w+1 {
            line(x, y, x+i as u8, y-w as u8);
        }
        for i in -w..w+1 {
            line(x, y, x+w as u8, y+i as u8);
        }
        for i in -w..w+1 {
            line(x, y, x+i as u8, y+w as u8);
        }
        for i in -w..w+1 {
            line(x, y, x-w as u8, y+i as u8);
        }
    }
}

#[no_mangle]
pub extern fn main() {
    let mut c: c_char = 0;
    unsafe {
        for a in 0..16 {
            for b in 0..16 {
                gotogxy(b, a);
                let mut d = a/4;
                let e = b/4;
                if d == e {
                    d = 3 - e;
                }
                color(d, e, SOLID);
                gprint(&[c, 0] as *const c_char);
                c += 1;
            }
        }

        color(LTGREY,WHITE,SOLID);
        circle(140,20,15,M_FILL);
        color(BLACK,WHITE,SOLID);
        circle(140,20,10,M_NOFILL);
        color(DKGREY,WHITE,XOR);
        circle(120,40,30,M_FILL);
        line(0,0,159,143);
        color(BLACK,LTGREY,SOLID);
        r#box(0,130,40,143,M_NOFILL);
        r#box(50,130,90,143,M_FILL);

        linetest(130, 100, 20);

        for _c in 0..144 {
            for b in 0..143 {
                for a in 0..160 {
                    color(getpix(a, b+1), WHITE, SOLID);
                    plot_point(a, b);
                }
                color(WHITE, WHITE, SOLID);
            }
            line(0, 143, 159, 143);
        }
    }
}

#[allow(unconditional_recursion)]
#[cfg(not(test))]
#[panic_handler]
fn panic_handler_phony(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
