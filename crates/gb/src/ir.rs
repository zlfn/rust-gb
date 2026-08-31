//! The Game Boy Color's infrared port.
//!
//! What the hardware offers is a lamp and a light sensor. There is no clock, no
//! framing and nothing to say a message has arrived. Talking to anything means
//! building all of that out of [`Port::led`] and [`Port::signal`].
//!
//! ```ignore
//! let Some(port) = ir::open() else { return };
//!
//! port.led(true);
//! ```
//!
//! # Sending
//!
//! A receiver settles to whatever infrared is already in the room, so a lamp
//! held on reads as nothing after a moment and a message has to be pulses. A
//! Philips remote control, for one, sends a run of 32 flashes of 10 and 17.5
//! microseconds where a single one of 880 would say the same thing.
//!
//! Nothing here counts those out. The lengths belong to whatever is on the
//! other end, and [double speed](crate::rt::DOUBLE_SPEED) halves what a loop of
//! a given length takes.
//!
//! See <https://gbdev.io/pandocs/CGB_Registers.html>.

use crate::mmio::cgb::{Infrared, RP};

/// The port, open and drawing current.
///
/// Dropping this leaves it that way. [`close`](Self::close) is what puts it
/// back down.
pub struct Port(());

/// Open the port, if this machine has one.
///
/// A Game Boy Advance runs Color cartridges and has no infrared port, so
/// [`is_cgb`](crate::is_cgb) on its own is not enough to go on.
#[inline]
pub fn open() -> Option<Port> {
    (crate::is_cgb() && !crate::is_gba()).then(|| {
        RP.write(Infrared::new().with_read_enable(0b11));
        Port(())
    })
}

impl Port {
    /// Turn the lamp on or off.
    #[inline]
    pub fn led(&self, on: bool) {
        RP.write(Infrared::new().with_read_enable(0b11).with_led_on(on));
    }

    /// Whether infrared is reaching the sensor.
    ///
    /// This Game Boy's own lamp reaches its own sensor, so [`led`](Self::led)
    /// registers here as readily as anything across the room.
    #[inline]
    pub fn signal(&self) -> bool {
        // The register reads 0 where light is seen, which is the one place
        // this module turns something over.
        !RP.read().receiving()
    }

    /// Close the port.
    ///
    /// The reading half draws current for as long as it is open, which is what
    /// this is for.
    #[inline]
    pub fn close(self) {
        RP.write(Infrared::new());
    }
}
