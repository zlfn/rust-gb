//! The MBC3 clock: seconds to days, running on the cartridge battery.
//!
//! The clock keeps time while the console is off, so a game can tell how long the
//! player was away.
//!
//! # Latching
//!
//! The counters inside the controller are not readable. What the window shows is
//! a snapshot of them, and [`latch`] takes a fresh one. Reading straight through
//! without one would mix an old second with a new minute, since the five
//! registers arrive one at a time and the clock does not wait.
//!
//! ```ignore
//! let (h, m) = gb_pak::rtc::latch(cs, |c| (c.hours(), c.minutes()));
//! ```
//!
//! # Trusting the time
//!
//! A cartridge whose clock has never been set may hold noise.

use core::ptr::{read_volatile, write_volatile};

use crate::{CriticalSection, WINDOW, reg};

const SECONDS: u8 = 0x08;
const MINUTES: u8 = 0x09;
const HOURS: u8 = 0x0A;
const DAYS_LOW: u8 = 0x0B;
const FLAGS: u8 = 0x0C;

const FLAG_DAY_HIGH: u8 = 0x01;
const FLAG_HALTED: u8 = 0x40;
const FLAG_OVERFLOW: u8 = 0x80;

#[inline]
fn get(register: u8) -> u8 {
    reg::select_raw(register);
    unsafe { read_volatile(WINDOW as *const u8) }
}

#[inline]
fn put(register: u8, value: u8) {
    reg::select_raw(register);
    unsafe { write_volatile(WINDOW as *mut u8, value) };
}

/// What the clock counts: time since it was set, not a calendar date.
///
/// `days` runs to 511 and then wraps, setting [`Latch::overflowed`]. A game that
/// wants a date keeps its own epoch and adds this to it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Time {
    pub days: u16,
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
}

/// One snapshot of the clock.
///
/// Reading a field is a register select and a load. They all come from the same
/// snapshot, so an hour and a minute cannot be a tick apart.
pub struct Latch(());

/// Snapshot the clock, switch the RAM on, and read it inside `f`.
///
/// The window holds one thing at a time, so `f` must not reach the save memory
/// or latch again: the rest of `f` would look at whatever that left selected.
pub fn latch<R>(_cs: CriticalSection<'_>, f: impl FnOnce(&Latch) -> R) -> R {
    reg::enable();
    reg::latch();
    let r = f(&Latch(()));
    reg::disable();
    r
}

/// Snapshot the clock and read all of it.
#[inline]
pub fn time(cs: CriticalSection<'_>) -> Time {
    latch(cs, |c| c.time())
}

impl Latch {
    /// Seconds, 0 to 59 on a clock that has been set.
    #[inline]
    pub fn seconds(&self) -> u8 {
        get(SECONDS)
    }

    /// Minutes, 0 to 59 on a clock that has been set.
    #[inline]
    pub fn minutes(&self) -> u8 {
        get(MINUTES)
    }

    /// Hours, 0 to 23 on a clock that has been set.
    #[inline]
    pub fn hours(&self) -> u8 {
        get(HOURS)
    }

    /// Days, 0 to 511 before the counter wraps.
    #[inline]
    pub fn days(&self) -> u16 {
        get(DAYS_LOW) as u16 | ((get(FLAGS) & FLAG_DAY_HIGH) as u16) << 8
    }

    /// All four fields at once.
    #[inline]
    pub fn time(&self) -> Time {
        Time {
            days: self.days(),
            hours: self.hours(),
            minutes: self.minutes(),
            seconds: self.seconds(),
        }
    }

    /// The day counter has wrapped at least once. Stays set until
    /// [`clear_overflow`].
    #[inline]
    pub fn overflowed(&self) -> bool {
        get(FLAGS) & FLAG_OVERFLOW != 0
    }

    /// The clock is stopped.
    #[inline]
    pub fn halted(&self) -> bool {
        get(FLAGS) & FLAG_HALTED != 0
    }
}

/// Set the clock. Starts it if it was stopped, and clears [`Latch::overflowed`].
///
/// Out-of-range fields are written as given.
pub fn set(_cs: CriticalSection<'_>, time: Time) {
    reg::enable();

    // The clock has to be stopped across the writes, or it ticks between them and
    // lands on a time that is part old and part new.
    put(FLAGS, FLAG_HALTED);
    put(SECONDS, time.seconds);
    put(MINUTES, time.minutes);
    put(HOURS, time.hours);
    put(DAYS_LOW, time.days as u8);
    put(FLAGS, ((time.days >> 8) as u8) & FLAG_DAY_HIGH);

    reg::disable();
}

/// Stop the clock, or start it again.
///
/// A stopped clock keeps its reading and does not count.
pub fn set_halted(_cs: CriticalSection<'_>, halted: bool) {
    reg::enable();
    reg::latch();
    let flags = get(FLAGS);
    put(FLAGS, if halted { flags | FLAG_HALTED } else { flags & !FLAG_HALTED });
    reg::disable();
}

/// Acknowledge a day-counter wrap.
pub fn clear_overflow(_cs: CriticalSection<'_>) {
    reg::enable();
    reg::latch();
    let flags = get(FLAGS);
    put(FLAGS, flags & !FLAG_OVERFLOW);
    reg::disable();
}
