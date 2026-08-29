//! Objects: the 40 sprites the PPU draws over the background.
//!
//! Each is four bytes in OAM at `0xFE00`, described by [`OamEntry`]. Position is
//! offset so that a sprite can sit partly off screen: `y` of 16 and `x` of 8 put
//! it at the top left corner, and either axis at zero hides it.
//!
//! Object tiles always read as [`Base8000`](super::tile::Addressing::Base8000),
//! whatever the background is set to. In [`Tall`](Size::Tall) the low bit of the
//! index is ignored and the pair is drawn as one 8 by 16 sprite. See
//! <https://gbdev.io/pandocs/OAM.html>.
//!
//! # What the hardware drops
//!
//! Only [`PER_SCANLINE`] objects are drawn on any one scanline; the rest of that
//! line's are skipped. Which ones survive is by `x` on the original Game Boy, lower first,
//! and by OAM index on the Game Boy Color. Nothing reports the loss.
//!
//! # Writing OAM
//!
//! Writing entries directly costs a blanking window per byte. The usual way is a
//! [`OamShadow`] page in work RAM, edited whenever, handed to the hardware in one
//! [`OamDma`] once a frame. See
//! <https://gbdev.io/pandocs/OAM_DMA_Transfer.html>.

use gb_hram::HramArea;
use gb_ram_fn::{RamFn, ram_fn};

use crate::interrupt::CriticalSection;
use crate::mmio::{LCDC, OAM, OamAttr, OamEntry};

use super::Access;

/// Objects OAM holds.
pub const ENTRY_COUNT: u8 = 40;

/// Objects the PPU draws on one scanline.
pub const PER_SCANLINE: u8 = 10;

/// How tall an object is, from `LCDC` bit 2. It applies to all of them at once.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Size {
    /// 8 by 8.
    Small,
    /// 8 by 16: the tile index's low bit is ignored and the pair is drawn.
    Tall,
}

/// Write one entry.
///
/// # Panics
///
/// If `i` is [`ENTRY_COUNT`] or beyond.
#[inline]
pub fn set(access: Access<'_>, i: u8, entry: OamEntry) {
    let dst = OAM.index(i as usize).as_usize() as *mut u8;
    let bytes = [entry.y, entry.x, entry.tile, u8::from(entry.attr)];
    unsafe { access.write(dst, &bytes) };
}

/// Move an object off screen, which is the only way to stop it being drawn.
///
/// There is no per-object enable bit; [`set_enabled`] switches all forty at once.
/// Writing zero to `y` puts the object sixteen pixels above the top edge, and
/// that one byte is the whole of it: `x`, the tile and the attributes can stay.
///
/// [`OamShadow::new`] leaves every object hidden this way, so a page starts
/// empty rather than needing to be cleared.
///
/// # Panics
///
/// If `i` is [`ENTRY_COUNT`] or beyond.
#[inline]
pub fn hide(access: Access<'_>, i: u8) {
    let dst = OAM.index(i as usize).as_usize() as *mut u8;
    unsafe { access.write(dst, &[0]) };
}

/// Whether the PPU draws objects at all, from `LCDC` bit 1.
#[inline]
pub fn enabled() -> bool {
    LCDC.read().obj_enable()
}

/// Draw objects, or stop.
#[inline]
pub fn set_enabled(on: bool) {
    // Read-modify-write: the enable bit is the only reason this is unsafe.
    unsafe { LCDC.write(LCDC.read().with_obj_enable(on)) };
}

/// The size every object is drawn at.
#[inline]
pub fn size() -> Size {
    if LCDC.read().obj_tall() { Size::Tall } else { Size::Small }
}

/// Set the size every object is drawn at.
#[inline]
pub fn set_size(size: Size) {
    unsafe { LCDC.write(LCDC.read().with_obj_tall(size == Size::Tall)) };
}

/// Bytes past the entries that the alignment pays for anyway.
pub const SPARE: usize = 256 - ENTRY_COUNT as usize * core::mem::size_of::<OamEntry>();

/// A page of entries for [`OamDma`] to hand over.
///
/// The hardware takes only the high byte of the source address, so this is
/// aligned to 256. Anywhere the linker puts a `static` is a legal source: work
/// RAM runs `0xC000..0xE000` and read-only data sits below `0x8000`, both inside
/// the `0x0000..0xE000` the transfer can read.
///
/// Alignment rounds the size up from the 160 bytes of entries, and
/// [`spare`](Self::spare) is the remainder. A transfer reads the entries only, so
/// those bytes are free for whatever wants to travel with the objects: per-object
/// velocities, animation counters, what a metasprite each belongs to.
#[repr(align(256))]
pub struct OamShadow {
    /// What the transfer hands to the hardware.
    pub entries: [OamEntry; ENTRY_COUNT as usize],
    /// Room the alignment leaves over. The transfer does not read it.
    pub spare: [u8; SPARE],
}

// Catches the entries and the spare drifting out of step with the alignment.
const _: () = assert!(core::mem::size_of::<OamShadow>() == 256);

impl OamShadow {
    /// A page with every object hidden and the spare zeroed.
    pub const fn new() -> Self {
        OamShadow {
            entries: [OamEntry { y: 0, x: 0, tile: 0, attr: OamAttr::new() }; ENTRY_COUNT as usize],
            spare: [0; SPARE],
        }
    }
}

impl Default for OamShadow {
    fn default() -> Self {
        Self::new()
    }
}

/// Bytes [`OamDma::install`] needs. Keep in step with the `ram_fn` below.
pub const DMA_LEN: usize = 8;

// The CPU may touch nothing but HRAM while the transfer runs, so the code that
// starts it and waits it out has to be running from there. `install` copies this.
#[ram_fn(max = 8)]
fn dma_routine(page: u8) {
    unsafe {
        core::arch::asm!(
            "ldh ($46), a",
            "ld a, 40",
            "2:",
            "dec a",
            "jr nz, 2b",
            inout("a") page => _,
            options(nostack),
        );
    }
}

/// The transfer routine, once it is in HRAM and callable.
///
/// Holding one is the proof that the copy has happened, so [`run`](Self::run)
/// needs no check and cannot be reached before it.
pub struct OamDma(<dma_routine::Handle as RamFn>::Fn);

// A bare `fn` is not `BankSafe` because it may point into a bank a switch would
// unmap. This one is installed into HRAM, which no switch touches.
#[cfg(feature = "bank")]
unsafe impl crate::bank::BankSafe for OamDma {}

impl OamDma {
    /// Copy the routine into `buf` and return the handle.
    ///
    /// Call once. Doing it again is harmless but copies the bytes over
    /// themselves.
    pub fn install(buf: &'static HramArea<DMA_LEN>) -> Self {
        OamDma(unsafe { dma_routine.install(buf.as_mut_ptr() as *mut [u8; DMA_LEN]) })
    }

    /// Hand `src` to the hardware, then wait out the 160 M-cycle transfer.
    ///
    /// The [`CriticalSection`] is the one hard requirement: an interrupt during
    /// the transfer would send the CPU to a vector it cannot read.
    ///
    /// The PPU cannot read OAM meanwhile either, so a transfer reaching over a
    /// visible line leaves that line's objects undrawn.
    /// [`Polled`](Access::Polled) waits for a VBlank with room for the whole of
    /// it; [`Direct`](Access::Direct) starts at once, which is what a VBlank
    /// handler wants and what a deliberate mid-frame transfer is written with.
    #[inline]
    pub fn run(&self, access: Access<'_>, _cs: CriticalSection<'_>, src: &OamShadow) {
        // 640 dots is longer than any blanking period but a VBlank's, so mode 0
        // is no help here and the line is what decides. Reading 151 leaves 913
        // dots, and 152 only 457; with the LCD off nothing is drawn and nothing
        // has to be waited for.
        if matches!(access, Access::Polled) && LCDC.read().lcd_enable() {
            while !(144..=151).contains(&crate::mmio::LY.read()) {}
        }
        (self.0)((src as *const OamShadow as usize >> 8) as u8);
    }
}
