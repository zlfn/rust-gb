//! Minimal rust-gb project template: prints "Hello, World!" and idles.
//!
//! Copy this directory to start a new Game Boy project, then build the ROM with
//! `cargo gb build` (output lands in `target/`).

#![no_std]
#![no_main]

use gbdk_sys::stdio::printf;

#[gb_rt::entry]
fn main() -> ! {
    unsafe {
        // Set up the GBDK runtime (display, console font, interrupts).
        gbdk_sys::init();

        // C strings must be NUL-terminated.
        printf(c"Hello, Rust-GB!\n".as_ptr());
    }

    // A Game Boy program never returns; spin forever.
    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
