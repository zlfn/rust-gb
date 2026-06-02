//! GBDK LCD ISR wobble, a port of gbdk-2020/examples/gb/lcd_isr_wobble.
//!
//! An LCD (STAT mode 0) interrupt nudges the background X scroll once per
//! scanline, producing a horizontal wobble whose phase drifts over time.

#![no_std]
#![no_main]

use core::ffi::c_char;
use gbdk_sys::gb::gb::*;
use gbdk_sys::stdio::printf;

static OFFSETS: [u8; 16] = [0, 1, 2, 3, 3, 2, 1, 0, 0, 1, 2, 3, 3, 2, 1, 0];

/// Base index into [`OFFSETS`]; written by `main`, read by the ISR. The Game Boy
/// is single-core with no atomics, so a volatile byte is the shared channel.
static mut BASE: u8 = 0;

/// STAT (mode 0 / H-Blank) interrupt: set the scroll offset for this scanline.
unsafe extern "C" fn scanline_isr() {
    let base = unsafe { core::ptr::read_volatile(&raw const BASE) } as usize;
    let ly = (unsafe { LY_REG.read() } & 0x07) as usize;
    unsafe { SCX_REG.write(OFFSETS[(base + ly) & 0x0F]) };
}

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    unsafe {
        gbdk_sys::init();
        printf(b"This is\na wobble\ntest\nfor DMG\n|\n|\n|\n|\n|\0".as_ptr() as *const c_char);

        // CRITICAL: install the handler chain with interrupts disabled.
        disable_interrupts();
        STAT_REG.write(STATF_MODE00);
        add_LCD(Some(scanline_isr));
        add_LCD(Some(nowait_int_handler));
        enable_interrupts();

        set_interrupts(IE_REG.read() | LCD_IFLAG);

        loop {
            vsync();
            let t = core::ptr::read_volatile(&raw const sys_time);
            core::ptr::write_volatile(&raw mut BASE, ((t >> 2) & 0x07) as u8);
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
