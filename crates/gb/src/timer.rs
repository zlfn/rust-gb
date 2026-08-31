//! The timer and the free-running divider.
//!
//! [`divider`] is free-running and cannot be configured; the timer counts at a
//! rate a program picks and raises an interrupt when it overflows. See
//! <https://gbdev.io/pandocs/Timer_and_Divider_Registers.html>.
//!
//! [`Timer::start`] hands back the proof that ticks are being counted.
//! Timestamps come from it and cannot outlive it.
//!
//! ```ignore
//! let timer = unsafe { Timer::start(rate::Hz256) }.unwrap();
//! let began = timer.now();
//! // ...
//! if began.elapsed() >= Duration::from_ms(5000) {
//!     expire();
//! }
//! ```
//!
//! Carrying the rate in the type is what settles the arithmetic behind
//! [`Duration`] at compile time, and it keeps a span from one rate from being
//! compared against a span from another.
//!
//! [`Duration`] is not [`core::time::Duration`], which holds a `u64` of seconds
//! beside a `u32` of nanoseconds and does its arithmetic to match; a span here
//! is a `u32` of ticks.
//!
//! # A handler of one's own
//!
//! A program that wants work at the tick, a sound driver most often, writes
//! `#[gb::rt::interrupt(Timer)]` and takes that vector. Nothing reports it, and
//! the count then moves only where the handler calls [`timer_tick`].
//!
//! The rate is still [`Timer::start`]'s to set. The eleven in [`rate`] are
//! powers of two; a tempo that falls between them means implementing [`Rate`]
//! for a type of one's own.

use core::marker::PhantomData;
use core::ops::{Add, Sub};

use crate::mmio::{DIV, Interrupts, TAC, TIMA, TMA, TimerClock, TimerCtrl};

// Little-endian, and incremented a byte at a time so that a tick costs one
// `ldh` pair in the common case.
crate::hram! {
    static TICKS: HramArea<4>;
}

/// Advance the tick count.
///
/// Needed only by a timer handler that replaced the one this module installs.
/// The rate is still [`Timer::start`]'s to set, so the two do not drift apart.
#[inline]
pub fn timer_tick() {
    unsafe {
        core::arch::asm!(
            "ldh a, ({t} + 0)", "inc a", "ldh ({t} + 0), a", "jr nz, 2f",
            "ldh a, ({t} + 1)", "inc a", "ldh ({t} + 1), a", "jr nz, 2f",
            "ldh a, ({t} + 2)", "inc a", "ldh ({t} + 2), a", "jr nz, 2f",
            "ldh a, ({t} + 3)", "inc a", "ldh ({t} + 3), a",
            "2:",
            t = sym TICKS,
            out("a") _,
            options(nostack),
        );
    }
}

// Weak, so a handler in the program replaces it. Not `pub`: the symbol is what
// the vector needs, and calling this as a function would return through `reti`.
#[linkage = "weak"]
#[unsafe(no_mangle)]
extern "z80-interrupt" fn on_timer() {
    timer_tick();
}

/// Read the four bytes, retrying until the top three agree across the low one.
///
/// The handler may land in the middle, and `di` is not an option here: it would
/// end a critical section the caller was inside. Reading the high bytes on both
/// sides of the low one catches a carry that crossed the read.
fn count_ticks() -> u32 {
    let (b0, b1, b2, b3): (u8, u8, u8, u8);
    unsafe {
        core::arch::asm!(
            "2:",
            "ldh a, ({t} + 3)", "ld d, a",
            "ldh a, ({t} + 2)", "ld e, a",
            "ldh a, ({t} + 1)", "ld b, a",
            "ldh a, ({t} + 0)", "ld c, a",
            "ldh a, ({t} + 1)", "cp b", "jr nz, 2b",
            "ldh a, ({t} + 2)", "cp e", "jr nz, 2b",
            "ldh a, ({t} + 3)", "cp d", "jr nz, 2b",
            t = sym TICKS,
            out("a") _,
            out("b") b1,
            out("c") b0,
            out("d") b3,
            out("e") b2,
            options(nostack, readonly),
        );
    }
    u32::from_le_bytes([b0, b1, b2, b3])
}

/// The divider, which counts at 16384 Hz whatever the timer is doing.
///
/// Programs read it for a value that is hard to predict, a random seed most
/// often.
#[inline]
pub fn divider() -> u8 {
    DIV.read()
}

/// Reset the divider to zero.
///
/// The timer shares the divider's counter, so this can advance it once. The APU
/// counts its envelopes and length timers off the same place; those step early
/// as well.
#[inline]
pub fn reset_divider() {
    DIV.write(0);
}

/// How often the timer overflows, as a type.
///
/// An implementation names an input clock and what to divide it by, which is
/// what the hardware takes; everything else follows. [`rate`] has the eleven a
/// program can usually afford, and an implementation of one's own is how to
/// reach a rate between them: a music driver wanting a particular tempo would
/// write it out rather than round to a power of two.
///
/// ```ignore
/// #[derive(Clone, Copy)]
/// struct Tempo;
///
/// impl Rate for Tempo {
///     const CLOCK: TimerClock = TimerClock::Hz65536;
///     const DIVISOR: u16 = 66;      // 992.96 Hz, the closest to a millisecond
/// }
/// ```
pub trait Rate: Copy {
    /// The input clock the hardware counts at.
    const CLOCK: TimerClock;

    /// What that clock is divided by, `1..=256`.
    const DIVISOR: u16;

    /// Overflows per second, rounded down.
    ///
    /// The clock over the divisor is the exact answer; this one loses the
    /// fraction, so 992.96 Hz reads as 992.
    const HZ: u32 = clock_hz(Self::CLOCK) / Self::DIVISOR as u32;

    /// What the hardware reloads the counter with.
    const MODULO: u8 = {
        assert!(Self::DIVISOR >= 1 && Self::DIVISOR <= 256, "a divisor is one of 1..=256");
        (256 - Self::DIVISOR) as u8
    };

    /// What ticks are multiplied by to reach milliseconds, before
    /// [`MS_SHIFT`](Self::MS_SHIFT).
    ///
    /// A tick is `1000 * divisor / clock` ms. 1000 is `8 * 125` and the clock is
    /// a power of two, so the division comes out as a shift; the twos the
    /// divisor brings are cancelled here rather than left to make this larger
    /// than it need be.
    const MS_NUM: u32 = reduce(125 * Self::DIVISOR as u32, clock_hz(Self::CLOCK).trailing_zeros() - 3).0;

    /// What the product is shifted by to reach milliseconds.
    const MS_SHIFT: u32 = reduce(125 * Self::DIVISOR as u32, clock_hz(Self::CLOCK).trailing_zeros() - 3).1;
}

/// What a clock counts at.
///
/// The names are the nominal ones; the timer is among the things CGB double
/// speed mode runs twice as fast, so a machine built for it counts double.
const fn clock_hz(clock: TimerClock) -> u32 {
    let nominal = match clock {
        TimerClock::Hz4096 => 4096,
        TimerClock::Hz16384 => 16384,
        TimerClock::Hz65536 => 65536,
        TimerClock::Hz262144 => 262_144,
    };
    nominal << crate::rt::DOUBLE_SPEED as u32
}

/// Cancel the twos shared by the multiplier and the shift, so [`Duration::ms`]
/// does not saturate sooner than it has to.
const fn reduce(mut num: u32, mut shift: u32) -> (u32, u32) {
    while shift > 0 && num % 2 == 0 {
        num /= 2;
        shift -= 1;
    }
    (num, shift)
}

/// The rates the timer can be run at.
///
/// Each names what it counts. The hardware divides one of four clocks by a
/// power of two, and these are the ones a program can afford: every overflow
/// runs the handler.
///
/// | Hz | 16 | 32 | 64 | 128 | 256 | 512 | 1024 | 2048 | 4096 | 8192 | 16384 |
/// |---|---|---|---|---|---|---|---|---|---|---|---|
/// | CPU | 0.05% | 0.1% | 0.2% | 0.4% | 0.7% | 1.5% | 2.9% | 5.9% | 12% | 23% | 47% |
///
/// The figures are for an original Game Boy. CGB double speed mode halves each
/// of them, since the handler costs the same and the processor issues twice as
/// much in a second.
///
/// The hardware counts faster than 16384 Hz, but the handler would take more of
/// the processor than the program has left to give.
///
/// Each rate is paired with the fastest clock that reaches it, which leaves
/// [`Timer::count`] dividing the period as finely as it can.
pub mod rate {
    use crate::mmio::TimerClock;

    macro_rules! rates {
        ($($name:ident = $hz:literal, $clock:ident / $div:literal;)*) => {$(
            #[doc = concat!($hz, " Hz.")]
            #[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
            pub struct $name;

            impl super::Rate for $name {
                const CLOCK: TimerClock = TimerClock::$clock;
                const DIVISOR: u16 = $div;
            }

            // The name is the derived rate, not a second source of truth.
            const _: () = assert!(<$name as super::Rate>::HZ == $hz);
        )*};
    }

    // The clock names are nominal, so CGB double speed mode needs a different
    // divisor for the same rate.
    #[cfg(not(feature = "cgb-double-speed"))]
    rates! {
        Hz16 = 16, Hz4096 / 256;
        Hz32 = 32, Hz4096 / 128;
        Hz64 = 64, Hz16384 / 256;
        Hz128 = 128, Hz16384 / 128;
        Hz256 = 256, Hz65536 / 256;
        Hz512 = 512, Hz65536 / 128;
        Hz1024 = 1024, Hz262144 / 256;
        Hz2048 = 2048, Hz262144 / 128;
        Hz4096 = 4096, Hz262144 / 64;
        Hz8192 = 8192, Hz262144 / 32;
        Hz16384 = 16384, Hz262144 / 16;
    }

    #[cfg(feature = "cgb-double-speed")]
    rates! {
        Hz32 = 32, Hz4096 / 256;
        Hz64 = 64, Hz4096 / 128;
        Hz128 = 128, Hz16384 / 256;
        Hz256 = 256, Hz16384 / 128;
        Hz512 = 512, Hz65536 / 256;
        Hz1024 = 1024, Hz65536 / 128;
        Hz2048 = 2048, Hz262144 / 256;
        Hz4096 = 4096, Hz262144 / 128;
        Hz8192 = 8192, Hz262144 / 64;
        Hz16384 = 16384, Hz262144 / 32;
    }
}

/// A running timer, and the scope its timestamps belong to.
///
/// [`stop`](Self::stop) takes it by value, so a timestamp taken from it keeps
/// the count running for as long as it is held.
pub struct Timer<R: Rate>(PhantomData<R>);

// A proof about the hardware's current state belongs to the context that made it.
impl<R: Rate> !Send for Timer<R> {}
impl<R: Rate> !Sync for Timer<R> {}

impl<R: Rate> Timer<R> {
    /// Start counting at `R`, or `None` if the timer is already running.
    ///
    /// Writing the control register can advance the timer once, so the first
    /// interrupt may come early.
    ///
    /// # Safety
    ///
    /// Lets the timer interrupt through and turns interrupts on. That is
    /// preemption, which the surrounding code may have been written to rule
    /// out, and it happens whether or not a timer is handed back.
    #[inline]
    pub unsafe fn start(_rate: R) -> Option<Self> {
        // `IE` is read and written back, and another module may have turned
        // interrupts on already, so the pair is kept off the air.
        crate::interrupt::disable();
        let free = !TAC.read().enable();
        if free {
            TMA.write(R::MODULO);
            TAC.write(TimerCtrl::new().with_clock(R::CLOCK).with_enable(true));
            unsafe {
                crate::interrupt::set_enabled(crate::interrupt::enabled() | Interrupts::TIMER);
            }
        }
        unsafe { crate::interrupt::enable() };
        free.then(|| Timer(PhantomData))
    }

    /// The tick count now.
    #[inline]
    pub fn now(&self) -> Instant<'_, R> {
        Instant(count_ticks(), PhantomData)
    }

    /// Sleep until `span` has passed.
    ///
    /// The CPU sleeps while waiting, so this does not return if `IE` lacks
    /// `TIMER` or interrupts are off: the count only moves when the handler
    /// runs.
    pub fn wait(&self, span: Duration<R>) {
        let from = count_ticks();
        while count_ticks().wrapping_sub(from) < span.0 {
            crate::interrupt::halt();
        }
    }

    /// How far the timer has counted towards its next overflow.
    ///
    /// This is the hardware register, so it moves at the input clock rather than
    /// the overflow rate and costs no interrupt at all.
    #[inline]
    pub fn count(&self) -> u8 {
        TIMA.read()
    }

    /// Stop counting.
    #[inline]
    pub fn stop(self) {
        TAC.write(TAC.read().with_enable(false));
    }
}

/// A point in the tick count.
///
/// Comparable and subtractable, and bounded to the [`Timer`] it came from: a
/// count that stopped and started again would leave it meaning nothing.
pub struct Instant<'a, R: Rate>(u32, PhantomData<&'a Timer<R>>);

impl<R: Rate> Clone for Instant<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R: Rate> Copy for Instant<'_, R> {}
impl<R: Rate> PartialEq for Instant<'_, R> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<R: Rate> Eq for Instant<'_, R> {}
impl<R: Rate> PartialOrd for Instant<'_, R> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<R: Rate> Ord for Instant<'_, R> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<R: Rate> Instant<'_, R> {
    /// How long since this was taken.
    #[inline]
    pub fn elapsed(self) -> Duration<R> {
        Duration(count_ticks().wrapping_sub(self.0), PhantomData)
    }

    /// How long from `earlier` to this.
    ///
    /// Zero if `earlier` is the later of the two.
    #[inline]
    pub fn saturating_duration_since(self, earlier: Self) -> Duration<R> {
        Duration(self.0.saturating_sub(earlier.0), PhantomData)
    }

    /// How long from `earlier` to this, or `None` if `earlier` is the later.
    #[inline]
    pub fn checked_duration_since(self, earlier: Self) -> Option<Duration<R>> {
        self.0.checked_sub(earlier.0).map(|d| Duration(d, PhantomData))
    }
}

impl<R: Rate> Sub for Instant<'_, R> {
    type Output = Duration<R>;

    /// How long from `earlier` to this, wrapping if they are the wrong way round.
    #[inline]
    fn sub(self, earlier: Self) -> Duration<R> {
        Duration(self.0.wrapping_sub(earlier.0), PhantomData)
    }
}

impl<'a, R: Rate> Add<Duration<R>> for Instant<'a, R> {
    type Output = Instant<'a, R>;

    #[inline]
    fn add(self, span: Duration<R>) -> Self {
        Instant(self.0.wrapping_add(span.0), PhantomData)
    }
}

impl<'a, R: Rate> Sub<Duration<R>> for Instant<'a, R> {
    type Output = Instant<'a, R>;

    #[inline]
    fn sub(self, span: Duration<R>) -> Self {
        Instant(self.0.wrapping_sub(span.0), PhantomData)
    }
}

/// A span of ticks at rate `R`.
///
/// The count wraps after 2^32 ticks, which is twelve days at 4096 Hz and months
/// below that. Nothing here allows for that: a span measured across the wrap
/// comes back wrong rather than saturated, so a machine left running that long
/// after [`Timer::start`] reads the wrong time.
pub struct Duration<R: Rate>(u32, PhantomData<R>);

impl<R: Rate> Clone for Duration<R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R: Rate> Copy for Duration<R> {}
impl<R: Rate> PartialEq for Duration<R> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<R: Rate> Eq for Duration<R> {}
impl<R: Rate> PartialOrd for Duration<R> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<R: Rate> Ord for Duration<R> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<R: Rate> Add for Duration<R> {
    type Output = Self;

    /// The two spans one after the other, saturating rather than wrapping.
    #[inline]
    fn add(self, other: Self) -> Self {
        Duration(self.0.saturating_add(other.0), PhantomData)
    }
}

impl<R: Rate> Sub for Duration<R> {
    type Output = Self;

    /// What is left of this span after `other`, or nothing if `other` is the
    /// longer of the two.
    #[inline]
    fn sub(self, other: Self) -> Self {
        Duration(self.0.saturating_sub(other.0), PhantomData)
    }
}

impl<R: Rate> Duration<R> {
    /// A span of `ticks`.
    pub const fn from_ticks(ticks: u32) -> Self {
        Duration(ticks, PhantomData)
    }

    /// The shortest span of at least `ms` milliseconds.
    ///
    /// Rounded up to a whole tick, so comparing against this is the answer to
    /// "has `ms` passed". At 16 Hz a tick is 63 ms, and nothing shorter can be
    /// told apart.
    pub const fn from_ms(ms: u32) -> Self {
        // The shift overruns 32 bits before the divide brings it back,
        // so the whole and the remainder are taken apart: each is a part of the
        // answer, so each fits wherever the answer does.
        let whole = ms / R::MS_NUM;
        if whole > u32::MAX >> R::MS_SHIFT {
            return Duration(u32::MAX, PhantomData);
        }
        let rest = ((ms % R::MS_NUM) << R::MS_SHIFT) + R::MS_NUM - 1;
        Duration(
            (whole << R::MS_SHIFT).saturating_add(rest / R::MS_NUM),
            PhantomData,
        )
    }

    /// The span in ticks.
    pub const fn ticks(self) -> u32 {
        self.0
    }

    /// The span in milliseconds, rounded down.
    ///
    /// Do not compare against this. It multiplies, and that is not cheap enough
    /// to run each time a program asks whether a span has passed.
    /// [`from_ms`](Self::from_ms) converts at compile time instead, leaving a
    /// plain comparison on ticks:
    ///
    /// ```ignore
    /// if elapsed >= Duration::from_ms(5000) { .. }   // one comparison
    /// if elapsed.ms() >= 5000 { .. }                 // a multiply first
    /// ```
    ///
    /// This is for a number to show or to log.
    pub fn ms(self) -> u32 {
        // The product overruns 32 bits and is taken in halves. Each half is a
        // part of the answer, so each fits wherever the answer does,
        // and the answer is a `u32` of milliseconds: seven weeks.
        let hi = (self.0 >> 16) * R::MS_NUM;
        let lo = (self.0 & 0xFFFF) * R::MS_NUM;
        (hi << (16 - R::MS_SHIFT)).wrapping_add(lo >> R::MS_SHIFT)
    }
}
