//! Interrupt control: the `IME` master switch and the `IE` / `IF` masks.
//!
//! Two gates stand between the hardware and a handler. `IME` is the CPU's master
//! switch, flipped by [`enable`] and [`disable`]; `IE` picks which of the five
//! sources may fire, and [`set_enabled`] writes it. Both must be open. The
//! runtime enters `main` with `IME` off, so a program sees no interrupt until it
//! opens them.
//!
//! Handlers are not installed here: write one with
//! [`#[gb::rt::interrupt]`](macro@crate::rt::interrupt), which binds it straight
//! to a vector.
//!
//! Turning interrupts on is `unsafe` throughout, since it introduces preemption
//! that the surrounding code may have been written to rule out. With the
//! `critical-section-impl` feature these functions are also the only sanctioned
//! way to change `IME`, which the implementation mirrors in HRAM.

pub use critical_section::CriticalSection;

use crate::mmio::{IE, IF, Interrupts};

/// Interrupt entry: the CPU clears `IME` when it dispatches.
#[doc(hidden)]
#[inline(always)]
pub fn __isr_enter() {
    mirror::set(false);
}

/// Interrupt exit: `reti` sets `IME` whatever the handler did to it.
#[doc(hidden)]
#[inline(always)]
pub fn __isr_exit() {
    mirror::set(true);
}

#[cfg(feature = "critical-section-impl")]
mod mirror {
    use gb_hram::{hram, prelude::*};

    // The runtime clears HRAM and enters `main` with interrupts off, so the
    // zero this starts at is already the right answer.
    hram! {
        static IME_ON: HramAtomicCell<bool>;
    }

    #[inline(always)]
    pub fn set(on: bool) {
        IME_ON.set(on);
    }

    struct SingleCore;
    critical_section::set_impl!(SingleCore);

    unsafe impl critical_section::Impl for SingleCore {
        unsafe fn acquire() -> critical_section::RawRestoreState {
            let was_on = IME_ON.get();
            super::disable();
            was_on
        }

        unsafe fn release(was_on: critical_section::RawRestoreState) {
            if was_on {
                unsafe { super::enable() };
            }
        }
    }
}

#[cfg(not(feature = "critical-section-impl"))]
mod mirror {
    #[inline(always)]
    pub fn set(_on: bool) {}
}

/// Clear IME with `di`: interrupts stop being serviced.
#[inline]
pub fn disable() {
    unsafe { core::arch::asm!("di") }
    mirror::set(false);
}

/// Set IME with `ei`, effective after the next instruction.
///
/// # Safety
///
/// Introduces preemption, breaking code that assumes interrupts stay off.
#[inline]
pub unsafe fn enable() {
    mirror::set(true);
    unsafe { core::arch::asm!("ei") }
}

/// Sleep until an interrupt arrives, then service it.
///
/// Emits `ei` and `halt` as one pair: `ei` takes effect only after the next
/// instruction, so an interrupt cannot be serviced in between and leave the `halt`
/// waiting for the following one.
///
/// # Safety
///
/// Introduces preemption, breaking code that assumes interrupts stay off.
#[inline]
pub unsafe fn enable_and_halt() {
    mirror::set(true);
    unsafe { core::arch::asm!("ei", "halt") }
}

/// Sleep until an interrupt arrives. Never returns if no enabled interrupt can fire.
///
/// Emits `halt` followed by a `nop`, which covers the halt bug: with IME clear and
/// an interrupt already pending, the CPU skips the sleep and reads the byte after
/// `halt` twice. See <https://gbdev.io/pandocs/halt.html>.
#[inline]
pub fn halt() {
    unsafe { core::arch::asm!("halt", "nop") }
}

/// Run `f` with interrupts disabled, handing it proof of that.
///
/// # Safety
///
/// Enables interrupts on the way out however they were set on the way in: SM83
/// cannot read `IME` back, so there is nothing to restore. Calling this with
/// interrupts already off ends the enclosing critical section early.
#[inline]
pub unsafe fn free<R>(f: impl FnOnce(CriticalSection<'_>) -> R) -> R {
    disable();
    let r = f(unsafe { CriticalSection::new() });
    unsafe { enable() };
    r
}

/// Read `IE`: the interrupts that may fire.
#[inline]
pub fn enabled() -> Interrupts {
    IE.read()
}

/// Read `IF`: the interrupts that have been requested.
#[inline]
pub fn pending() -> Interrupts {
    IF.read()
}

/// Replace `IE` with `mask`.
///
/// # Safety
///
/// Introduces preemption, breaking code that assumes interrupts stay off.
#[inline]
pub unsafe fn set_enabled(mask: Interrupts) {
    unsafe { IE.write(mask) }
}
