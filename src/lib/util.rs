//! GameBoy utilities.

use core::time::Duration;

use crate::gbdk_c;

/// Sleep for given durations.
///
/// # Caution
/// Due to the impossibility of u64 operations, for [`Duration`] beyond [`u32::MAX`] (about 1193
/// hours), it is bounded to [`u32::MAX`]. Most of cases, the GameBoy can't stay on for that long, so
/// it shouldn't be a big problem.
pub fn sleep(dur: Duration) {
    let mut time = dur.as_millis() as u32;
    while time > (u16::MAX) as u32 {
        unsafe {gbdk_c::gb::gb::delay(u16::MAX)};
        time -= u16::MAX as u32;
    }
    unsafe {gbdk_c::gb::gb::delay(time as u16)};
}

/// Sleep for given milliseconds.
pub fn sleep_ms(ms: u32) {
    sleep(Duration::from_millis(ms as u64));
}
