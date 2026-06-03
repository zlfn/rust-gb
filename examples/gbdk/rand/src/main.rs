//! GBDK Rand — A direct port of GBDK's rand example.

#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

use gbdk_sys::gb::drawing::*;
use gbdk_sys::gb::gb::*;
use gbdk_sys::rand::*;
use gbdk_sys::stdio::*;

const RANGE_SIZE: u8 = 160u8 / 4u8;
const HALF_RANGE_SIZE: u8 = RANGE_SIZE / 2u8;
const fn rand_range_8bit(randval: u8, range: u8) -> u8 {
    let r = (((randval & 0xFF) as u16 * range as u16) >> 8) as u8;
    if r >= range { unsafe { core::hint::unreachable_unchecked() } }
    r
}

#[gb_rt::entry]
fn main() {
    unsafe { gbdk_sys::init(); }
    let mut seed: u16;
    let mut accu: [u8; RANGE_SIZE as usize] = [0; RANGE_SIZE as usize]; 
    let mut accua: [u8; RANGE_SIZE as usize] = [0; RANGE_SIZE as usize]; 
    let mut accut: [u8; RANGE_SIZE as usize] = [0; RANGE_SIZE as usize]; 
    let mut accub: [u8; RANGE_SIZE as usize] = [0; RANGE_SIZE as usize]; 

    unsafe {
        puts(c"Getting seed".as_ptr());
        puts(c"Push any key (1)".as_ptr());
        waitpad(0xFF);
        waitpadup();
        seed = DIV_REG.read() as u16;
        puts(c"Push any key (2)".as_ptr());
        waitpad(0xFF);
        waitpadup();
        seed |= (DIV_REG.read() as u16) << 8u16;

        initarand(seed);

        line(RANGE_SIZE * 1, 0, RANGE_SIZE * 1, 143);
        line(RANGE_SIZE * 2, 0, RANGE_SIZE * 2, 143);
        line(RANGE_SIZE * 3, 0, RANGE_SIZE * 3, 143);

        loop {
            let r = rand_range_8bit(rand(), RANGE_SIZE);
            let ra = rand_range_8bit(arand(), RANGE_SIZE);
            let rt1 = rand_range_8bit(rand(), HALF_RANGE_SIZE);
            let rt2 = rand_range_8bit(rand(), HALF_RANGE_SIZE);
            let rt = HALF_RANGE_SIZE + (rt1 - rt2);
            if rt >= RANGE_SIZE { core::hint::unreachable_unchecked() }

            let rb1 = rand_range_8bit(rand(), HALF_RANGE_SIZE / 2);
            let rb2 = rand_range_8bit(rand(), HALF_RANGE_SIZE / 2);
            let rb3 = rand_range_8bit(rand(), HALF_RANGE_SIZE / 2);
            let rb4 = rand_range_8bit(rand(), HALF_RANGE_SIZE / 2);
            let rb = HALF_RANGE_SIZE + (rb1 - rb2) + (rb3 - rb4);
            if rb >= RANGE_SIZE { core::hint::unreachable_unchecked() }

            accu[r as usize] += 1;
            let r_bucket_height = accu[r as usize];
            if r_bucket_height > 144 { break; }
            core::arch::asm!("di");
            plot(r + (RANGE_SIZE * 0), 144-r_bucket_height, LTGREY, SOLID);
            core::arch::asm!("ei");

            accua[ra as usize] += 1;
            let ra_bucket_height = accua[ra as usize];
            if ra_bucket_height > 144 { break; }
            core::arch::asm!("di");
            plot(ra + (RANGE_SIZE * 1), 144-ra_bucket_height, DKGREY, SOLID);
            core::arch::asm!("ei");

            accut[rt as usize] += 1;
            let rt_bucket_height = accut[rt as usize];
            if rt_bucket_height > 144 { break; }
            core::arch::asm!("di");
            plot(rt + (RANGE_SIZE * 2), 144-rt_bucket_height, BLACK, SOLID);
            core::arch::asm!("ei");

            accub[rb as usize] += 1;
            let rb_bucket_height = accub[rb as usize];
            if rb_bucket_height > 144 { break; }
            core::arch::asm!("di");
            plot(rb + (RANGE_SIZE * 3), 144-rb_bucket_height, BLACK, SOLID);
            core::arch::asm!("ei");
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
