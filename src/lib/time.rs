use core::ops::{Add, AddAssign, Sub, SubAssign};
use core::time::Duration;

use crate::{irq::Interrupt, mmio};

static mut ENABLE_TIMER: bool = false;
static mut SYSTEM_TIMER: u32 = 0;

unsafe extern "C" fn increase_system_timer() {
    SYSTEM_TIMER += 1;
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum TimerClock {
    MCycle256 = 0x00, // 4096 Hz
    MCycle4 = 0x01,   // 262144 Hz
    MCycle16 = 0x10,  // 65536 Hz
    MCycle64 = 0x11,  // 16384 Hz
}

/// A measurement of a monotonically nondecreasing clock.
/// Opaque and useful only with [`Duration`]
///
/// Provide features similar to `std::time::Instant` using timer intterupt of GameBoy.
///
/// You must enable timer interrupt with the [`Instant::init()`] method before use,
/// and modifying timer interrupt related memory while enabling [`Instant`] will result in
/// inaccurate time clock for [`Instant`]
///
/// There is an error of around 180 seconds per hour (precision 5%).
/// Therefore, it is not recommended to measure a long time with this feature.
/// The RTC (Real Time Clock) feature, which can be used for long time measurements,
/// will be implemented in the future.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(u32);

impl Instant {
    // SYSTEM_TIMER increasing frequency.
    pub const FREQUENCY: u32 = 4096 / 0xFF;

    pub unsafe fn init() -> Interrupt {
        // Make SYSTEM_TIMER increase in 16Hz (Slowest as possible)
        Self::enable_timer(TimerClock::MCycle256, 0);
        ENABLE_TIMER = true;
        Interrupt::Timer.add(increase_system_timer)
    }

    pub fn now() -> Self {
        unsafe {
            if !ENABLE_TIMER {
                panic!("Instant is not initalized");
            }
            return Instant(SYSTEM_TIMER);
        }
    }

    // TODO: handle double speed mode / SGB
    pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        if self.0 < earlier.0 {
            None
        } else {
            let delta_freq = self.0 - earlier.0;

            // Calculate miliseconds from SYSTEM_TIMER change
            Some(Duration::from_millis(
                (delta_freq / Self::FREQUENCY * 1000).into(),
            ))
        }
    }

    pub fn duration_since(&self, earlier: Instant) -> Duration {
        if self.0 < earlier.0 {
            panic!("duration_since get later instance");
        } else {
            let delta_freq = self.0 - earlier.0;

            // Calculate miliseconds from SYSTEM_TIMER change
            Duration::from_millis((delta_freq / Self::FREQUENCY * 1000) as u64)
        }
    }

    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        self.checked_duration_since(earlier).unwrap_or_default()
    }

    pub fn elapsed(&self) -> Duration {
        Instant::now() - *self
    }

    pub fn checked_add(&self, duration: Duration) -> Option<Instant> {
        self.0
            .checked_add(duration.as_millis() as u32 / 1000 * Self::FREQUENCY)
            .map(Instant)
    }

    pub fn checked_sub(&self, duration: Duration) -> Option<Instant> {
        self.0
            .checked_sub(duration.as_millis() as u32 / 1000 * Self::FREQUENCY)
            .map(Instant)
    }

    fn enable_timer(clock: TimerClock, modulo: u8) {
        mmio::TMA.write(modulo);
        mmio::TAC.write(0x4 | clock as u8);
    }

    fn disable_timer() {
        mmio::TAC.write(0);
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;
    fn add(self, rhs: Duration) -> Self::Output {
        self.checked_add(rhs)
            .expect("overflow when adding duration to instant")
    }
}

impl AddAssign<Duration> for Instant {
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;
    fn sub(self, rhs: Duration) -> Self::Output {
        self.checked_sub(rhs)
            .expect("overflow when subtracting duration from instant")
    }
}

impl SubAssign<Duration> for Instant {
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;
    fn sub(self, rhs: Instant) -> Self::Output {
        self.duration_since(rhs)
    }
}

/// Sleep for given durations.
pub fn sleep(dur: Duration) {
    sleep_until(Instant::now() + dur);
}

/// Sleep for given milliseconds.
pub fn sleep_ms(ms: u32) {
    sleep(Duration::from_millis(ms as u64));
}

/// Sleep until given [`Instant`]
pub fn sleep_until(deadline: Instant) {
    while !Instant::now().saturating_duration_since(deadline).is_zero() {}
}
