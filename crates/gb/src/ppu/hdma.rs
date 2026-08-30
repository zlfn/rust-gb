//! The Game Boy Color's copier into video memory.
//!
//! The quickest way to put data in video memory. There are two, and what
//! separates them is not speed but what the program may do meanwhile. See
//! <https://gbdev.io/pandocs/CGB_Registers.html#ff51ff52--hdma1-hdma2-cgb-mode-only-vram-dma-source-high-low-write-only>.
//!
//! [`copy`] stops the CPU until the whole block has moved, so nothing else runs
//! and there is nothing to get wrong. One of the largest size roughly fills a
//! VBlank, which is most of a tileset.
//!
//! [`stream`] moves one block per HBlank and lets the program run in between,
//! reaching a comparable amount over a frame. What it costs is listed
//! on [`Stream`], and the list is long: it holds the video memory bank and the
//! source bank still for its whole life, and a sleeping CPU stops it. It suits a
//! stretch where nothing else touches video memory, a loading screen rather than
//! a scrolling level.
//!
//! Neither reaches an original Game Boy, which has no such hardware: the write
//! that would start a transfer goes nowhere, and the call returns having moved
//! nothing.

use crate::mmio::cgb::{HDMA1, HDMA2, HDMA3, HDMA4, HDMA5, HdmaCtrl};

use super::Access;

/// Bytes one transfer step moves, and the granularity of everything here.
pub const BLOCK_LEN: usize = 16;

/// Bytes the largest transfer carries.
pub const MAX_LEN: usize = 128 * BLOCK_LEN;

/// Storage the copier can read, aligned as it needs.
///
/// The hardware ignores the low four bits of both addresses, so an unaligned
/// source would be read from the wrong place rather than refused. [`copy`] and
/// [`stream`] check for it instead, and this is how the check is passed: a
/// `[Tile; N]` is byte-aligned and lands wherever the linker puts it.
///
/// ```ignore
/// static TILES: hdma::Source<[Tile; 64]> = hdma::Source([..]);
/// ```
#[repr(align(16))]
pub struct Source<T>(pub T);

/// Copy `src` to `dst` in video memory, stopping the CPU until it is done.
///
/// The copier does not wait for the PPU. The [`Access`] settles where the
/// transfer starts, not where it ends, so a block long enough to outlast the
/// window is still being written while the lines it reaches are drawn, and those
/// come out garbled: size one to the window, or switch the display off.
///
/// Nothing runs while it works, so a source in a switchable bank is safe here in
/// a way it is not in [`stream`].
///
/// # Panics
///
/// If `src` is empty, longer than [`MAX_LEN`], not a whole number of
/// [`BLOCK_LEN`]s, or misaligned; if `dst` is outside `0x8000..0xA000`, is
/// misaligned, or has no room; or if a [`stream`] is still running, which this
/// would stop rather than join.
pub fn copy(access: Access<'_>, dst: u16, src: &[u8]) {
    assert!(HDMA5.read().hblank(), "a stream is still running");
    let blocks = prepare(dst, src);
    if matches!(access, Access::Polled) {
        super::wait_blank();
    }
    HDMA5.write(HdmaCtrl::new().with_blocks(blocks).with_hblank(false));
}

/// Start moving `src` to `dst` sixteen bytes at a time, one step per HBlank.
///
/// `None` if a transfer is already running. Read [`Stream`] before using this:
/// what the program may do while it runs is narrow.
///
/// `src` is `'static` because the copier reads it over many frames and a
/// switchable bank would move underneath it. That rules out banked ROM and
/// cartridge RAM, neither of which lends for longer than a scope.
///
/// # Panics
///
/// As [`copy`].
pub fn stream(dst: u16, src: &'static [u8]) -> Option<Stream> {
    if !HDMA5.read().hblank() {
        return None;
    }
    let blocks = prepare(dst, src);
    // Starting one during HBlank is the documented way to get a broken transfer.
    // Mode 3 is excluded too: it can be a dot from HBlank, which the write below
    // would then land in. From mode 2 the nearest HBlank is 252 dots off.
    while matches!(
        crate::mmio::STAT.read().mode(),
        crate::mmio::PpuMode::HBlank | crate::mmio::PpuMode::Drawing
    ) {}
    HDMA5.write(HdmaCtrl::new().with_blocks(blocks).with_hblank(true));
    Some(Stream(()))
}

/// A stream under way, one block per HBlank.
///
/// Until it is done or [`stopped`](Self::stop), the program must not:
///
/// - change the video memory bank, which rules out
///   [`with_vram_bank`](super::with_vram_bank) and
///   [`edit_attrs`](super::map::edit_attrs);
/// - unmap the bank the source sits in;
/// - execute `halt`, which stops the copier until the CPU wakes. That includes
///   [`wait_vblank`](super::wait_vblank), so a frame paced with it will barely
///   advance the transfer.
///
/// Dropping this leaves the transfer running. Nothing is torn by that; the
/// obligations above simply go unwatched.
pub struct Stream(());

impl Stream {
    /// Whether the copier has stopped, whether by finishing or by [`stop`](Self::stop).
    #[inline]
    pub fn is_done(&self) -> bool {
        HDMA5.read().hblank()
    }

    /// Bytes still to move, zero once [`is_done`](Self::is_done).
    #[inline]
    pub fn remaining(&self) -> u16 {
        let c = HDMA5.read();
        if c.hblank() {
            0
        } else {
            (c.blocks() as u16 + 1) * BLOCK_LEN as u16
        }
    }

    /// Stop early, leaving what has arrived in place. Does nothing once the
    /// transfer has finished on its own.
    #[inline]
    pub fn stop(self) {
        // Clearing bit 7 terminates a running transfer, but starts a
        // general-purpose one where none is running: the addresses this one left
        // behind would be copied from and to all over again. A transfer that
        // ends in the few cycles between the check and the write still gets one,
        // which the hardware gives no way to close.
        if !self.is_done() {
            HDMA5.write(HdmaCtrl::new().with_hblank(false));
        }
    }
}

/// Load the addresses and return the length in blocks, minus one.
fn prepare(dst: u16, src: &[u8]) -> u8 {
    let s = src.as_ptr() as usize;
    assert!(!src.is_empty() && src.len() <= MAX_LEN && src.len() % BLOCK_LEN == 0);
    assert!(s % BLOCK_LEN == 0);
    // Both ends, not just the start: a block running off the end of read-only
    // memory reaches into VRAM, which the copier reads as rubbish.
    let end = s + src.len() - 1;
    assert!((s < 0x8000 && end < 0x8000) || (0xA000..0xE000).contains(&s) && end < 0xE000);
    let d = dst as usize;
    assert!((0x8000..0xA000).contains(&d) && d % BLOCK_LEN == 0 && d + src.len() <= 0xA000);

    HDMA1.write((s >> 8) as u8);
    HDMA2.write(s as u8);
    HDMA3.write((dst >> 8) as u8);
    HDMA4.write(dst as u8);
    (src.len() / BLOCK_LEN - 1) as u8
}
