//! Port of GBDK's `ram_function`: copy a function's machine code into RAM and
//! HRAM, then run it from there.

#![no_std]
#![no_main]

use core::ffi::{c_char, c_uint};
use gb_hram::hram;
use gb_ram_fn::{RamFn, ram_fn};
use gbdk_sys::stdio::{printf, puts};

static mut COUNTER: u16 = 0;

// `#[ram_fn]` puts `inc` in its own section so its length is known and its bytes
// can be copied. It is position-independent because it only touches `COUNTER` by
// absolute address. `max` caps its compiled size, checked when `cargo-gb` builds
// the ROM.
#[ram_fn(max = 16)]
fn inc() {
    unsafe {
        let c = core::ptr::read_volatile(&raw const COUNTER);
        core::ptr::write_volatile(&raw mut COUNTER, c.wrapping_add(1));
    }
}

// Landing sites for the copied code: an ordinary WRAM static and a gb-hram area.
static mut RAM_BUF: [u8; 16] = [0; 16];
hram! {
    static HRAM_BUF: HramArea<16>;
}

fn print_counter() {
    unsafe {
        printf(
            b" Counter is %u\n\0".as_ptr() as *const c_char,
            core::ptr::read_volatile(&raw const COUNTER) as c_uint,
        );
    }
}

#[gb_rt::entry]
fn main() -> ! {
    unsafe {
        gbdk_sys::init();

        puts(b"Program Start...\0".as_ptr() as *const c_char);
        print_counter();

        puts(b"Call ROM\0".as_ptr() as *const c_char);
        inc.rom()();
        print_counter();

        puts(b"Call RAM\0".as_ptr() as *const c_char);
        let ram_inc = inc.install(&raw mut RAM_BUF);
        ram_inc();
        print_counter();

        puts(b"Call HIRAM\0".as_ptr() as *const c_char);
        let hram_inc = inc.install(HRAM_BUF.as_array_ptr());
        hram_inc();
        print_counter();

        puts(b"The End...\0".as_ptr() as *const c_char);
    }

    loop {}
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
