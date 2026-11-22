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

        puts(c"Byte".as_ptr());
        puts(c"  A      : Send".as_ptr());
        puts(c"  B      : Receive".as_ptr());
        puts(c"String".as_ptr());
        puts(c"  START  : Send".as_ptr());
        puts(c"  SELECT : Receive".as_ptr());

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
                printf(c"Sending b... ".as_ptr());
                core::ptr::write_volatile(&raw mut _io_out, n);
                n = n.wrapping_add(1);
                send_byte();
                while io_status() == IO_SENDING && joypad() == 0 {}
                if io_status() == IO_IDLE {
                    printf(c"OK\n".as_ptr());
                } else {
                    printf(c"#%d\n".as_ptr(), io_status() as i32);
                }
            } else if key == J_B {
                printf(c"Receiving b... ".as_ptr());
                receive_byte();
                while io_status() == IO_RECEIVING && joypad() == 0 {}
                if io_status() == IO_IDLE {
                    printf(c"OK\n%d\n".as_ptr(), io_in() as i32);
                } else {
                    printf(c"#%d\n".as_ptr(), io_status() as i32);
                }
            } else if key == J_START {
                printf(c"Sending s... ".as_ptr());
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
                        printf(c"#%d\n".as_ptr(), io_status() as i32);
                        break;
                    }
                    if c == 0 {
                        break;
                    }
                    idx += 1;
                }
                if io_status() == IO_IDLE {
                    printf(c"OK\n".as_ptr());
                }
            } else if key == J_SELECT {
                printf(c"Receiving s... ".as_ptr());
                let mut idx = 0usize;
                loop {
                    receive_byte();
                    while io_status() == IO_RECEIVING && joypad() == 0 {}
                    if io_status() != IO_IDLE {
                        printf(c"#%d\n".as_ptr(), io_status() as i32);
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
                    printf(c"OK\n%s\n".as_ptr(), (&raw const BUFFER) as *const c_char);
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
