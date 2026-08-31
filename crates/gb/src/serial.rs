//! Byte exchange over the link cable.
//!
//! A transfer is always an exchange. Eight bits leave as eight arrive, and one
//! cannot be had without the other. One of the two Game Boys supplies the clock
//! and so decides when it happens: [`drive`] is that side, [`follow`] the other.
//!
//! ```ignore
//! let answer = serial::exchange(b'?', Rate::Slow);
//! ```
//!
//! # Waiting
//!
//! Nothing here gives up on its own. A [`follow`] with nothing at the far end
//! waits for a clock that never comes. Count the frames and
//! [`abort`](Transfer::abort) it. [`drive`] always finishes, reading `$FF`
//! off a cable with nothing on it.
//!
//! A Game Boy about to be clocked has to be waiting before the other one
//! starts, so the clocking side has to pause between bytes. How long depends on
//! what the far end does with them.
//!
//! See <https://gbdev.io/pandocs/Serial_Data_Transfer_(Link_Cable).html>.

use crate::mmio::{SB, SC, SerialCtrl};

/// How fast this Game Boy clocks the wire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rate {
    /// 8192 bits a second, the rate every Game Boy handles.
    Slow,
    /// 262144 bits a second, which only a Game Boy Color can clock. The far
    /// end follows whatever reaches it and need not be one.
    #[cfg(feature = "cgb")]
    #[cfg_attr(docsrs, doc(cfg(feature = "cgb")))]
    Fast,
}

impl Rate {
    const fn fast(self) -> bool {
        match self {
            Rate::Slow => false,
            #[cfg(feature = "cgb")]
            Rate::Fast => true,
        }
    }
}

/// A byte on its way out with another on its way in.
///
/// Dropping this leaves the wire as it was. The transfer runs to its end, or
/// goes on waiting, with nobody to collect what arrives.
pub struct Transfer(());

impl Transfer {
    /// The byte that arrived, once the transfer has finished.
    #[inline]
    pub fn poll(&self) -> Option<u8> {
        (!SC.read().transfer_enable()).then(|| SB.read())
    }

    /// Give up.
    ///
    /// A transfer that has already finished is left alone, so what arrived is
    /// still there to [`poll`](Self::poll).
    #[inline]
    pub fn abort(self) {
        let ctrl = SC.read();
        if ctrl.transfer_enable() {
            SC.write(ctrl.with_transfer_enable(false));
        }
    }
}

/// Clock the wire and wait for the byte coming the other way.
///
/// About 1024 M-cycles at [`Rate::Slow`], a seventeenth of a frame, and 32 at
/// [`Rate::Fast`]. Double speed leaves both figures alone, the wire and the
/// processor keeping step.
#[inline]
pub fn exchange(byte: u8, rate: Rate) -> u8 {
    let transfer = drive(byte, rate);
    loop {
        if let Some(answer) = transfer.poll() {
            return answer;
        }
    }
}

/// Clock the wire and carry on.
#[inline]
pub fn drive(byte: u8, rate: Rate) -> Transfer {
    SB.write(byte);
    SC.write(
        SerialCtrl::new()
            .with_clock_select(true)
            .with_clock_speed(rate.fast())
            .with_transfer_enable(true),
    );
    Transfer(())
}

/// Hold a byte out for the other Game Boy to clock when it is ready.
#[inline]
pub fn follow(byte: u8) -> Transfer {
    SB.write(byte);
    SC.write(SerialCtrl::new().with_transfer_enable(true));
    Transfer(())
}
