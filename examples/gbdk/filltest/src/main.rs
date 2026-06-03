//! GBDK Filltest — A direct port of GBDK's filltest example.

#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use core::ffi::c_char;
use gbdk_sys::gb::drawing::*;

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

#[gb_rt::entry]
fn main() {
    unsafe { gbdk_sys::init(); }
    let mut c: c_char = 0;
    unsafe {
        for a in 0..16u8 {
            for b in 0..16u8 {
                gotogxy(b, a);
                let mut d = a/4;
                let e = b/4;
                if d == e {
                    d = 3 - e;
                }
                color(d, e, SOLID);
                gprint(&mut [c, 0] as *mut c_char);
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

        for _c in 0..144u8 {
            for b in 0..143u8 {
                for a in 0..160u8 {
                    core::arch::asm!("di");
                    let px = getpix(a, b+1);
                    core::arch::asm!("ei");
                    color(px, WHITE, SOLID);
                    core::arch::asm!("di");
                    plot_point(a, b);
                    core::arch::asm!("ei");
                }
                color(WHITE, WHITE, SOLID);
            }
            line(0, 143, 159, 143);
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
