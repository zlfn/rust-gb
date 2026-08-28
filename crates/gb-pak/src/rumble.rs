//! The MBC5 rumble motor.
//!
//! The motor is one bit of the register that also selects the SRAM bank, so a
//! cartridge with rumble reaches eight SRAM banks at most. That register cannot
//! be read back, so this crate remembers both fields and writes them together,
//! whichever one a call is changing.
//!
//! The bit drives the motor directly: it runs until switched off.
//!
//! ```ignore
//! gb_pak::rumble::set(true);
//! // ... some frames ...
//! gb_pak::rumble::set(false);
//! ```

use crate::{reg};

/// Start or stop the motor.
#[inline]
pub fn set(on: bool) {
    reg::set_rumble(on);
}
