//! Interrupt control.

pub use critical_section::CriticalSection;

use crate::mmio::{IE, IF, Interrupts};

/// Clear IME with `di`: interrupts stop being serviced.
#[inline]
pub fn disable() {
    unsafe { core::arch::asm!("di") }
}

/// Set IME with `ei`, effective after the next instruction.
///
/// # Safety
///
/// Introduces preemption, breaking code that assumes interrupts stay off.
#[inline]
pub unsafe fn enable() {
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
    unsafe { core::arch::asm!("ei", "halt") }
}

/// Sleep until an interrupt arrives. Never returns if no
/// enabled interrupt can fire.
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
