//! GBDK ISR vector, a port of gbdk-2020/examples/gb/isr_vector.
//!
//! The companion of `lcd-isr-wobble`: the same per-scanline wobble, but the STAT
//! (mode 0 / H-Blank) handler is installed **directly at the interrupt vector**
//! instead of through GBDK's dispatcher. The STAT vector jumps to `_on_lcd_stat`;
//! defining a strong one here overrides the weak GBDK dispatcher, so the interrupt
//! runs this handler with no dispatch overhead.

#![no_std]
#![feature(abi_z80_interrupt)]
#![no_main]

use core::ffi::c_char;
use gbdk_sys::gb::gb::*;
use gbdk_sys::stdio::printf;

static OFFSETS: [u8; 16] = [0, 1, 2, 3, 3, 2, 1, 0, 0, 1, 2, 3, 3, 2, 1, 0];

/// Base index into [`OFFSETS`]; written by `main`, read by the ISR. No atomics on
/// SM83, so a volatile byte is the shared channel.
static mut BASE: u8 = 0;

/// STAT (mode 0 / H-Blank) interrupt, installed directly at the vector: set the
/// scroll offset for this scanline. This handler wins over GBDK's weak dispatcher;
/// the `z80-interrupt` convention saves the pairs it clobbers and returns with `reti`.
#[gb_rt::interrupt(LcdStat)]
fn wobble() {
    let base = unsafe { core::ptr::read_volatile(&raw const BASE) } as usize;
    let ly = (unsafe { LY_REG.read() } & 0x07) as usize;
    unsafe { SCX_REG.write(OFFSETS[(base + ly) & 0x0F]) };
}

#[gb_rt::entry]
fn main() -> ! {
    unsafe {
        gbdk_sys::init();
        printf(b"Direct\nISR\nvector\n|\n|\n|\n|\n|\n|\0".as_ptr() as *const c_char);

        // Trigger STAT on mode 0 (H-Blank); add the LCD interrupt to the VBlank
        // that `init` already enabled.
        STAT_REG.write(STATF_MODE00);
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
