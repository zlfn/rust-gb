//! GBDK serial link comm, a port of gbdk-2020/examples/gb/comm/comm.c.
//!
//! Link-cable demo over the serial port. A / B send / receive a single byte,
//! START / SELECT send / receive a string. Needs a second linked Game Boy.

#![no_std]
#![no_main]

use core::ffi::c_char;
use gbdk_sys::gb::gb::*;
use gbdk_sys::stdio::{printf, puts};

static MESSAGE: [u8; 13] = *b"Hello World!\0";
static mut BUFFER: [u8; 32] = [0; 32];

fn cstr(s: &[u8]) -> *const c_char {
    s.as_ptr() as *const c_char
}

unsafe fn io_status() -> u8 {
    unsafe { core::ptr::read_volatile(&raw const _io_status) }
}
unsafe fn io_in() -> u8 {
    unsafe { core::ptr::read_volatile(&raw const _io_in) }
}

#[gb_rt::entry]
fn main() -> ! {
    unsafe {
        gbdk_sys::init();

        puts(cstr(b"Byte\0"));
        puts(cstr(b"  A      : Send\0"));
        puts(cstr(b"  B      : Receive\0"));
        puts(cstr(b"String\0"));
        puts(cstr(b"  START  : Send\0"));
        puts(cstr(b"  SELECT : Receive\0"));

        // The handler chain must be installed with interrupts disabled.
        disable_interrupts();
        add_SIO(Some(nowait_int_handler));
        enable_interrupts();
        set_interrupts(SIO_IFLAG);

        let mut n: u8 = 0;
        loop {
            let key = waitpad(J_A | J_B | J_START | J_SELECT);
            waitpadup();

            if key == J_A {
                printf(cstr(b"Sending b... \0"));
                core::ptr::write_volatile(&raw mut _io_out, n);
                n = n.wrapping_add(1);
                send_byte();
                while io_status() == IO_SENDING && joypad() == 0 {}
                if io_status() == IO_IDLE {
                    printf(cstr(b"OK\n\0"));
                } else {
                    printf(cstr(b"#%d\n\0"), io_status() as i32);
                }
            } else if key == J_B {
                printf(cstr(b"Receiving b... \0"));
                receive_byte();
                while io_status() == IO_RECEIVING && joypad() == 0 {}
                if io_status() == IO_IDLE {
                    printf(cstr(b"OK\n%d\n\0"), io_in() as i32);
                } else {
                    printf(cstr(b"#%d\n\0"), io_status() as i32);
                }
            } else if key == J_START {
                printf(cstr(b"Sending s... \0"));
                let mut idx = 0usize;
                loop {
                    let c = MESSAGE[idx];
                    core::ptr::write_volatile(&raw mut _io_out, c);
                    loop {
                        send_byte();
                        while io_status() == IO_SENDING && joypad() == 0 {}
                        if !(io_status() != IO_IDLE && joypad() == 0) {
                            break;
                        }
                    }
                    if io_status() != IO_IDLE {
                        printf(cstr(b"#%d\n\0"), io_status() as i32);
                        break;
                    }
                    if c == 0 {
                        break;
                    }
                    idx += 1;
                }
                if io_status() == IO_IDLE {
                    printf(cstr(b"OK\n\0"));
                }
            } else if key == J_SELECT {
                printf(cstr(b"Receiving s... \0"));
                let mut idx = 0usize;
                loop {
                    receive_byte();
                    while io_status() == IO_RECEIVING && joypad() == 0 {}
                    if io_status() != IO_IDLE {
                        printf(cstr(b"#%d\n\0"), io_status() as i32);
                        break;
                    }
                    let c = io_in();
                    BUFFER[idx] = c;
                    if c == 0 {
                        break;
                    }
                    idx += 1;
                }
                if io_status() == IO_IDLE {
                    printf(cstr(b"OK\n%s\n\0"), (&raw const BUFFER) as *const c_char);
                }
            }

            waitpadup();
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
