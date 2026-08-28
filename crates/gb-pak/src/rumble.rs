//! The MBC5 rumble motor.
//!
//! The motor is one bit of the register that also selects the SRAM bank, and
//! that register cannot be read back, so this crate keeps the last value written
//! and rewrites both fields together. A cartridge with rumble therefore reaches
//! eight SRAM banks at most.
//!
//! The bit drives the motor directly: it runs until switched off.
//!
//! ```ignore
//! gb_pak::rumble::set(cs, true);
//! // ... some frames ...
//! gb_pak::rumble::set(cs, false);
//! ```

use crate::{CriticalSection, reg};

/// Start or stop the motor.
#[inline]
pub fn set(_cs: CriticalSection<'_>, on: bool) {
    reg::set_rumble(on);
}
