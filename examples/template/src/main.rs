//! Minimal rust-gb project template: prints "Hello, World!" and idles.
//!
//! Copy this directory to start a new Game Boy project, then build the ROM with
//! `cargo gb build` (output lands in `target/`).

#![no_std]
#![no_main]

use core::ffi::c_char;
use gbdk_sys::stdio::printf;

#[gb_rt::entry]
fn main() -> ! {
    unsafe {
        // Set up the GBDK runtime (display, console font, interrupts).
        gbdk_sys::init();

        // C strings must be NUL-terminated.
        printf(b"Hello, Rust-GB!\n\0".as_ptr() as *const c_char);
    }

    // A Game Boy program never returns; spin forever.
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
