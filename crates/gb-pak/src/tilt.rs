//! The MBC7 accelerometer.
//!
//! Two axes of tilt, the only analogue input the console can read. The sensor
//! measures gravity, so it reports which way the cartridge is leaning rather than
//! how fast it is moving, and shaking it reads as noise.
//!
//! A reading is latched before it can be read, the same way `rtc` does: write
//! `0x55` to clear, then `0xAA` to sample.
//!
//! ```ignore
//! let t = gb_pak::tilt::read();
//! let lean = t.x.wrapping_sub(gb_pak::tilt::CENTRE) as i16;
//! ```

use core::ptr::{read_volatile, write_volatile};

use crate::{WINDOW, reg};

const CLEAR: *mut u8 = WINDOW as *mut u8;
const SAMPLE: *mut u8 = (WINDOW + 0x10) as *mut u8;
const X_LOW: *const u8 = (WINDOW + 0x20) as *const u8;
const X_HIGH: *const u8 = (WINDOW + 0x30) as *const u8;
const Y_LOW: *const u8 = (WINDOW + 0x40) as *const u8;
const Y_HIGH: *const u8 = (WINDOW + 0x50) as *const u8;

/// What each axis reads when the cartridge is level.
pub const CENTRE: u16 = 0x8000;

/// One latched pair of axes, as the sensor reports them.
///
/// Subtract [`CENTRE`] for a signed lean. `x` grows leftward and `y` upward, and
/// one g moves either by about `0x70`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tilt {
    pub x: u16,
    pub y: u16,
}

/// Sample both axes.
pub fn read() -> Tilt {
    reg::enable();
    reg::select_raw(0x40);

    unsafe { write_volatile(CLEAR, 0x55) };
    unsafe { write_volatile(SAMPLE, 0xAA) };

    let x = unsafe { read_volatile(X_LOW) } as u16
        | (unsafe { read_volatile(X_HIGH) } as u16) << 8;
    let y = unsafe { read_volatile(Y_LOW) } as u16
        | (unsafe { read_volatile(Y_HIGH) } as u16) << 8;

    reg::disable();
    Tilt { x, y }
}
