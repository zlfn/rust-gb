//! The pixel-processing unit, which draws the screen.
//!
//! The picture comes from six modules: [`tile`] holds the pixels, [`map`] says
//! which tile goes in which cell, [`bg`] and [`window`] place the two layers
//! that read a map, [`obj`] carries the sprites drawn over them, and
//! [`palette`] decides what a pixel's colour index becomes.
//!
//! # Reaching video memory
//!
//! VRAM and OAM are not always the CPU's to write. The PPU takes them while it
//! draws, and a write made then is dropped without a word. Every function that
//! writes takes an [`Access`], which is the answer to "how do you know this will
//! land".
//!
//! [`Direct`](Access::Direct) says the caller is already somewhere it will:
//! inside a VBlank, or with the LCD switched off. It is the fast answer, and
//! [`Vblank::with`] and [`with_lcd_off`] are how one is come by. Nothing enforces
//! the window's length, so a closure that runs past the end of a VBlank loses
//! the rest of its writes.
//!
//! [`Polled`](Access::Polled) says nothing about when it is called, and waits
//! for the PPU itself instead. It reaches far more of the frame than VBlank
//! alone, since HBlank recurs on every line, and it blocks until the whole write
//! is through. For anyone coming from GBDK, this is the shape all of its video
//! memory writes take.
//!
//! ```ignore
//! let vblank = unsafe { Vblank::listen() };
//!
//! // A frame's worth of updates, taken inside the window
//! vblank.with(|d| {
//!     map::write(d, bg::map(), col % 32, 0, LEVEL.sub(col, 0, 1, 18));
//!     obj::set(d, 0, player);
//! });
//!
//! // A bulk load: the screen goes blank, and the window has no deadline
//! ppu::with_lcd_off(|d| tile::write_all(d, 0, &TILESET));
//!
//! // Nowhere in particular: let the write wait for the PPU itself
//! tile::write(Access::Polled, 5, &spark);
//! ```
//!
//! Work that has to happen between frames belongs before [`Vblank::with`], not
//! inside it: decide what to draw first, then take the window and spend it on
//! writes.
//!
//! # Writing from a handler
//!
//! Much of what follows reads a register, changes a bit and writes it back,
//! `LCDC` most of all: [`bg`], [`window`], [`obj`] and [`tile`] each own a bit
//! of it and leave the rest alone. A handler doing the same to the same register
//! races whatever it interrupted, and one of the two changes is lost.
//!
//! [`palette`] fares worse. Its colours travel through an index register, set
//! once and stepped by the hardware, so a handler writing a palette of its own
//! sends the rest of the interrupted write into whichever palette it left
//! selected.
//!
//! So a program driving a raster effect from a `STAT` handler should keep its
//! other writes to those registers in VBlank, where the two cannot overlap.
//!
//! # Frame pacing
//!
//! A program that uses this module links a weak `_on_vblank` advancing a frame
//! counter. Writing `#[gb::rt::interrupt(VBlank)]` takes that vector instead,
//! which nothing reports: such a handler must call [`frame_tick`], or
//! [`Vblank::wait`] never returns.

pub mod bg;
#[cfg(feature = "cgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "cgb")))]
pub mod hdma;
pub mod map;
pub mod obj;
pub mod palette;
pub mod tile;
pub mod window;

use core::marker::PhantomData;

use gb_hram::HramAtomicAccess;

// One byte, so each access is a single `ldh` that cannot tear against the
// handler, and the wrap every 256 frames is harmless to the inequality
// `Vblank::wait` compares with.
crate::hram! {
    static FRAME: HramAtomicCell<u8>;
}

// Weak, so a handler in the program replaces it. Not `pub`: the symbol is what
// the vector needs, and calling this as a function would return through `reti`.
#[linkage = "weak"]
#[unsafe(no_mangle)]
extern "z80-interrupt" fn on_vblank() {
    frame_tick();
}

/// How a write reaches video memory.
///
/// VRAM at `0x8000` and OAM at `0xFE00` are locked on different schedules: VRAM
/// is reachable except in mode 3, OAM except in modes 2 and 3, and turning the
/// LCD off opens both. See
/// <https://gbdev.io/pandocs/Accessing_VRAM_and_OAM.html>.
///
/// Where the hardware has them locked it ignores writes and reads back `0xFF`,
/// so a value written there is lost rather than wrong, and nothing reports it.
#[derive(Clone, Copy)]
pub enum Access<'a> {
    /// The caller is already inside a window where both are open, so writes go
    /// straight through.
    ///
    /// Minted by [`Vblank::with`] and [`with_lcd_off`], and bounded to the closure
    /// they run. It is zero-sized and `Copy`; the lifetime is the only thing
    /// stopping it being carried out of the window:
    ///
    /// ```compile_fail
    /// # use gb::ppu::{Access, Vblank};
    /// # let vblank = unsafe { Vblank::listen() };
    /// let mut saved: Option<Access> = None;
    /// vblank.with(|d| { saved = Some(d); }); // ERROR: `d` escapes the closure
    /// ```
    #[non_exhaustive]
    Direct(PhantomData<&'a ()>),

    /// Wait for the PPU to release video memory before writing, which makes the
    /// call safe at any point in the frame.
    ///
    /// This reaches far more of the frame than one VBlank does, since HBlank
    /// recurs on every line. What it costs is the wait, which a bulk write pays
    /// for every byte: one this way can hold the CPU for more than a frame while
    /// the picture stays up.
    Polled,
}

// A proof about the PPU's current state belongs to the context that read it.
impl<'a> !Send for Access<'a> {}
impl<'a> !Sync for Access<'a> {}

impl<'a> Access<'a> {
    /// Mint [`Direct`](Access::Direct), asserting that video memory is reachable.
    ///
    /// The escape hatch for a context [`Vblank::with`] cannot serve, such as a
    /// VBlank handler, which is already inside the window it would wait for.
    ///
    /// # Safety
    ///
    /// The PPU must be in mode 0 or 1, or the LCD off. The returned lifetime is
    /// unconstrained, so the caller must bound it to the period that holds.
    pub const unsafe fn assume() -> Self {
        Access::Direct(PhantomData)
    }

    /// Copy `src` into video memory at `dst`.
    ///
    /// # Safety
    ///
    /// `dst` must be a VRAM or OAM address with room for all of `src`.
    pub(crate) unsafe fn write(self, dst: *mut u8, src: &[u8]) {
        // One routine for VRAM and OAM: `wait_blank` covers the OAM lock as well
        // as the VRAM one, so there is nothing left to tell the two apart.
        //
        // Resolve the discipline once. Monomorphising on it keeps the branch out
        // of the loop, which matters where a run is only a byte or two long.
        match self {
            Access::Polled => unsafe { run::<true>(dst, src) },
            _ => unsafe { run::<false>(dst, src) },
        }
    }
}

unsafe fn run<const WAIT: bool>(dst: *mut u8, src: &[u8]) {
    // Walk the destination rather than indexing from `dst`: one fewer value live
    // across the wait, in a loop the register allocator is already tight on.
    let mut d = dst;
    for b in src {
        if WAIT {
            wait_blank();
        }
        unsafe { core::ptr::write_volatile(d, *b) };
        d = unsafe { d.add(1) };
    }
}

/// Block until the PPU is between lines or between frames.
///
/// Modes 0 and 1 are the two, and seeing either leaves at least 80 dots before
/// video memory is taken again. Waiting only for mode 3 to pass would not: mode
/// 2 can be a dot from mode 3, and a write made on the strength of that check
/// would be dropped after it.
#[inline(always)]
pub(crate) fn wait_blank() {
    use crate::mmio::PpuMode;
    while matches!(
        crate::mmio::STAT.read().mode(),
        PpuMode::OamScan | PpuMode::Drawing
    ) {}
}

/// Advance the frame counter.
///
/// Needed only by a VBlank handler that replaced the one this module installs.
#[inline(always)]
pub fn frame_tick() {
    // A read-modify-write, and single-writer: the VBlank handler is the only
    // caller that matters, and the CPU clears IME on dispatch, so it cannot
    // interrupt itself.
    FRAME.set(FRAME.get().wrapping_add(1));
}

/// The frame clock: proof that the VBlank interrupt is reaching the counter.
///
/// Everything that waits for a frame hangs without it.
pub struct Vblank(());

// A proof about the hardware's current state belongs to the context that made it.
impl !Send for Vblank {}
impl !Sync for Vblank {}

impl Vblank {
    /// Listen for the VBlank interrupt.
    ///
    /// # Safety
    ///
    /// Turns interrupts on. That is preemption, which the surrounding code may
    /// have been written to rule out.
    #[inline]
    pub unsafe fn listen() -> Self {
        // `IE` is read and written back, and another module may have turned
        // interrupts on already, so the pair is kept off the air.
        crate::interrupt::disable();
        unsafe {
            crate::interrupt::set_enabled(
                crate::interrupt::enabled() | crate::mmio::Interrupts::VBLANK,
            );
            crate::interrupt::enable();
        }
        Vblank(())
    }

    /// Frames counted since boot, wrapping at 256.
    ///
    /// [`wrapping_sub`](u8::wrapping_sub) of two readings is the frames between
    /// them; a plain subtraction is what overflows across the wrap.
    #[inline(always)]
    pub fn frame_count(&self) -> u8 {
        FRAME.get()
    }

    /// Block until the next frame.
    ///
    /// The CPU sleeps while waiting. Does not return with the LCD off, since no
    /// VBlank arrives then, nor where a replacement handler skips
    /// [`frame_tick`].
    pub fn wait(&self) {
        let seen = FRAME.get();
        loop {
            // VBlank is requested at dot 0 of line 144, so LY reading 143 can be
            // a single dot away from it, and halting there would sleep through
            // the frame being waited for. Every other line leaves at least 457
            // dots, against the ~60 this takes to reach the halt.
            //
            // LY is read before the counter so a VBlank landing between the two
            // is caught by the counter read rather than missed. Volatile and
            // `asm!` accesses keep that order.
            let near_vblank = crate::mmio::LY.read() == 143;
            if FRAME.get() != seen {
                return;
            }
            if !near_vblank {
                crate::interrupt::halt();
            }
        }
    }

    /// Wait for the next frame, then run `f` inside its VBlank.
    ///
    /// Nothing enforces the window. `f` keeps running once the PPU has moved on,
    /// and writes past that point are dropped in silence.
    ///
    /// ```ignore
    /// vblank.with(|d| {
    ///     bg::set_scroll(d, camera_x, camera_y);
    ///     obj::set(d, 0, player);
    /// });
    ///
    /// // Overruns: the tail of the tileset lands after the window and is lost.
    /// // `with_lcd_off` has no deadline.
    /// vblank.with(|d| load_tileset(&TILES, d));
    /// ```
    ///
    /// Waiting goes through [`wait`](Self::wait), so this does not return under
    /// the same conditions.
    pub fn with<R>(&self, f: impl FnOnce(Access<'_>) -> R) -> R {
        self.wait();
        f(unsafe { Access::assume() })
    }
}

/// Turn the LCD off for the length of `f`.
///
/// The screen goes blank, and in exchange video memory stays reachable for as
/// long as `f` runs rather than the roughly 1140 M-cycles [`Vblank::with`]
/// allows, or 2280 in CGB double speed mode. A program with more tiles than one
/// VBlank fits loads them this way.
///
/// Unlike [`Vblank::with`], the wait here is a poll and needs no interrupt, so
/// this runs before a program has turned them on.
///
/// [`Vblank::wait`] does not return inside `f`, since no VBlank arrives with the LCD
/// off. An LCD that was already off is left that way and `f` runs at once.
///
/// Turning it back on restarts the PPU at line 0, and the screen stays blank
/// through that first frame. See <https://gbdev.io/pandocs/LCDC.html>.
pub fn with_lcd_off<R>(f: impl FnOnce(Access<'_>) -> R) -> R {
    let lcdc = crate::mmio::LCDC.read();

    // Waiting with the LCD already off would never return, and a caller that
    // switched it off means to keep it off.
    if !lcdc.lcd_enable() {
        return f(unsafe { Access::assume() });
    }

    // Clearing the enable bit outside VBlank can damage the panel, so wait for
    // one. The line is polled rather than waited on through `Vblank`: this runs
    // before interrupts are on, which is where a program loads its tiles,
    // and it leaves `IME` alone rather than ending a critical section it was
    // called inside.
    //
    // The first loop skips a VBlank already under way, so the second catches the
    // next one near its start. A handler landing between that read and the write
    // would have to outlast the rest of the VBlank to push the write out of it.
    while crate::mmio::LY.read() >= 146 {}
    while crate::mmio::LY.read() < 145 {}

    // Re-read: a handler may have reconfigured the PPU during the wait.
    unsafe { crate::mmio::LCDC.write(crate::mmio::LCDC.read().with_lcd_enable(false)) };

    let r = f(unsafe { Access::assume() });

    // Re-read rather than writing `lcdc` back: `f` may have reconfigured the
    // PPU, and only the enable bit is this function's to restore.
    unsafe { crate::mmio::LCDC.write(crate::mmio::LCDC.read().with_lcd_enable(true)) };
    r
}

/// Which half of the Game Boy Color's video memory the CPU sees at `0x8000`.
///
/// The PPU reads both halves through its own paths, so this is only about CPU
/// access.
#[cfg(feature = "cgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "cgb")))]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VramBank {
    /// Tile slots and the two tilemaps, as on the original Game Boy.
    Zero = 0,
    /// A second set of tile slots, and per-cell attributes where bank zero holds
    /// the tilemaps.
    One = 1,
}

/// Run `f` with `bank` mapped at `0x8000`, then put back the one that was there.
///
/// The Game Boy Color banks all of `0x8000..0xA000`, so the same
/// [`tile`] or map write lands in a different place depending on this. Bracketing
/// it keeps that visible at the call site and out of the surrounding code.
///
/// `f` takes nothing: the [`Access`] an enclosing [`Vblank::with`] or
/// [`with_lcd_off`] handed out is still good here, since which bank is mapped and
/// whether video memory is reachable are separate questions. The two scopes nest
/// in either order.
///
/// A handler that changes the bank must put it back, the way one that switches
/// ROM banks must, and one that writes video memory has to set the bank it means
/// rather than assume zero: it may well have interrupted this.
///
/// An original Game Boy has no such register: the write goes nowhere and `f` runs against
/// the one bank there is, overwriting what was already in it. A cartridge that
/// runs on both machines has to ask [`is_cgb`](crate::is_cgb) first, there being
/// no second bank to fall back to.
///
/// ```ignore
/// ppu::with_lcd_off(|d| {
///     tile::write_all(d, 0, &TILES);
///     ppu::with_vram_bank(VramBank::One, || tile::write_all(d, 0, &MORE));
/// });
/// ```
#[cfg(feature = "cgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "cgb")))]
#[inline]
pub fn with_vram_bank<R>(bank: VramBank, f: impl FnOnce() -> R) -> R {
    // Bit 0 is the bank; the rest read as ones.
    let saved = crate::mmio::cgb::VBK.read() & 1;
    crate::mmio::cgb::VBK.write(bank as u8);
    let r = f();
    crate::mmio::cgb::VBK.write(saved);
    r
}
