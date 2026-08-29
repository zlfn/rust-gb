//! Tilemaps: two 32 by 32 grids of tile indices.
//!
//! One byte per cell, naming a tile the way [`tile`](super::tile) describes.
//! Which grid a layer reads is set on the layer, by [`bg::set_map`](super::bg::set_map)
//! and [`window::set_map`](super::window::set_map); the two layers can share one. See <https://gbdev.io/pandocs/Tile_Maps.html>.
//!
//! # Wrapping
//!
//! The grid is 32 cells square where the screen shows 20 by 18, and it wraps on
//! both axes: cell 31 is followed by cell 0. Scrolling relies on that, so
//! coordinates here are taken modulo 32 rather than checked. There is no
//! out-of-range cell on a torus.

use crate::mmio::{TILEMAP_0, TILEMAP_1};

use super::{Access, wait_blank};

/// Cells along one side of a tilemap.
pub const SIDE: u8 = 32;

/// Which of the two tilemaps.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Map {
    /// The grid at `0x9800`.
    Zero,
    /// The grid at `0x9C00`.
    One,
}

impl Map {
    const fn base(self) -> usize {
        match self {
            Map::Zero => TILEMAP_0.as_usize(),
            Map::One => TILEMAP_1.as_usize(),
        }
    }
}

/// What [`TileGrid`] and [`AttrGrid`] are both made of: a rectangle inside a
/// larger grid, carrying that grid's row stride.
#[derive(Clone, Copy)]
struct Grid<'a> {
    data: &'a [u8],
    stride: u8,
    x: u8,
    y: u8,
    w: u8,
    h: u8,
}

impl<'a> Grid<'a> {
    const fn new(data: &'a [u8], width: u8) -> Self {
        let rows = data.len() / width as usize;
        let h = if rows > u8::MAX as usize { u8::MAX } else { rows as u8 };
        Grid { data, stride: width, x: 0, y: 0, w: width, h }
    }

    const fn sub(self, x: u8, y: u8, w: u8, h: u8) -> Self {
        // The origin is carried rather than re-slicing, so this stays const and
        // the stride never has to be recomputed. Saturating, so that an origin
        // past the end is refused by the write rather than wrapping back into
        // range.
        Grid { x: self.x.saturating_add(x), y: self.y.saturating_add(y), w, h, ..self }
    }
}

macro_rules! grid_kind {
    ($(#[$m:meta])* $name:ident, $what:literal) => {
        #[doc = concat!("A rectangle of ", $what, ", with the row stride of the grid it sits in.")]
        ///
        /// A level map is wider than a tilemap, so the strip that scrolls into
        /// view is not contiguous in the source. Binding the stride to the data
        /// rather than passing it alongside keeps the two from being paired
        /// wrongly, and the two kinds of grid are separate types so that one
        /// cannot be written where the other belongs.
        ///
        /// ```ignore
        #[doc = concat!("const LEVEL: ", stringify!($name), " = ", stringify!($name), "::new(&DATA, 100);")]
        #[doc = concat!("map::write(d, Map::Zero, x, 0, LEVEL.sub(col, 0, 1, 18));")]
        /// ```
        $(#[$m])*
        #[derive(Clone, Copy)]
        pub struct $name<'a>(Grid<'a>);

        $(#[$m])*
        impl<'a> $name<'a> {
            /// All of `data`, laid out `width` cells per row.
            ///
            /// The height follows from the length, so a partial last row is
            /// dropped, and so is anything past row 255.
            ///
            /// # Panics
            ///
            /// If `width` is zero.
            pub const fn new(data: &'a [u8], width: u8) -> Self {
                $name(Grid::new(data, width))
            }

            /// The `w` by `h` rectangle at `(x, y)` within this one.
            ///
            /// Not checked here. One reaching outside the data it was built from
            /// is refused when it is written, not when it is taken.
            pub const fn sub(self, x: u8, y: u8, w: u8, h: u8) -> Self {
                $name(self.0.sub(x, y, w, h))
            }

            /// Cells across.
            pub const fn width(self) -> u8 {
                self.0.w
            }

            /// Cells down.
            pub const fn height(self) -> u8 {
                self.0.h
            }
        }
    };
}

grid_kind!(TileGrid, "tile indices");
grid_kind!(
    #[cfg(feature = "cgb")]
    #[cfg_attr(docsrs, doc(cfg(feature = "cgb")))]
    AttrGrid,
    "attribute bytes"
);

/// Put `tile` in one cell.
#[inline]
pub fn set(access: Access<'_>, map: Map, x: u8, y: u8, tile: u8) {
    let dst = cell(map, x, y);
    unsafe { access.write(dst, &[tile]) };
}

/// Put `tile` in every cell of a `w` by `h` rectangle.
///
/// # Panics
///
/// If `w` or `h` is past [`SIDE`], which would wrap over what was just written.
pub fn fill(access: Access<'_>, map: Map, x: u8, y: u8, w: u8, h: u8, tile: u8) {
    match access {
        Access::Polled => unsafe { fill_rect::<true>(map, x, y, w, h, tile) },
        _ => unsafe { fill_rect::<false>(map, x, y, w, h, tile) },
    }
}

/// Copy `src` in with its top left at `(x, y)`.
///
/// # Panics
///
/// If `src` is wider or taller than [`SIDE`], which would wrap over what was
/// just written, or if it does not fit the data it was built from.
pub fn write(access: Access<'_>, map: Map, x: u8, y: u8, src: TileGrid<'_>) {
    match access {
        Access::Polled => unsafe { blit::<true>(map, x, y, src.0) },
        _ => unsafe { blit::<false>(map, x, y, src.0) },
    }
}

#[inline(always)]
fn cell(map: Map, x: u8, y: u8) -> *mut u8 {
    let x = (x % SIDE) as usize;
    let y = (y % SIDE) as usize;
    (map.base() + y * SIDE as usize + x) as *mut u8
}

unsafe fn blit<const WAIT: bool>(map: Map, x: u8, y: u8, src: Grid<'_>) {
    if src.w == 0 || src.h == 0 {
        return;
    }
    assert!(src.w <= SIDE && src.h <= SIDE, "a write wider or taller than the grid would overwrite itself");
    // Check the rectangle once so the run below can index unchecked: it has to
    // fit the stride, or a row would read into its neighbour, and its far corner
    // has to be inside the data. That corner is computed in `usize` and checked:
    // `usize` is sixteen bits here, and the coordinates are bytes, so either step
    // can carry past what the other operand can represent.
    let last = (src.y as usize + src.h as usize - 1)
        .checked_mul(src.stride as usize)
        .and_then(|v| v.checked_add(src.x as usize + src.w as usize - 1));
    assert!(
        src.x as usize + src.w as usize <= src.stride as usize
            && matches!(last, Some(l) if l < src.data.len()),
        "a rectangle reaching outside the grid it was taken from"
    );

    let x = x % SIDE;
    // A row reaching past the right edge continues at column zero, so it is one
    // contiguous run or two, decided once rather than per cell.
    let head = if src.w < SIDE - x { src.w } else { SIDE - x };

    // Step the source by one stride per row rather than multiplying again: the
    // multiply is a software routine on this target.
    let mut s = unsafe {
        src.data
            .as_ptr()
            .add(src.y as usize * src.stride as usize + src.x as usize)
    };
    let mut off = row_offset(y);
    for _ in 0..src.h {
        let d = (map.base() | off) as *mut u8;

        unsafe { copy::<WAIT>(d.add(x as usize), s, head) };
        if src.w > head {
            unsafe { copy::<WAIT>(d, s.add(head as usize), src.w - head) };
        }
        s = unsafe { s.add(src.stride as usize) };
        off = next_row(off);
    }
}

unsafe fn fill_rect<const WAIT: bool>(map: Map, x: u8, y: u8, w: u8, h: u8, tile: u8) {
    if w == 0 || h == 0 {
        return;
    }
    assert!(w <= SIDE && h <= SIDE, "a fill wider or taller than the grid would overwrite itself");

    let x = x % SIDE;
    let head = if w < SIDE - x { w } else { SIDE - x };

    let mut off = row_offset(y);
    for _ in 0..h {
        let d = (map.base() | off) as *mut u8;
        unsafe { spread::<WAIT>(d.add(x as usize), head, tile) };
        if w > head {
            unsafe { spread::<WAIT>(d, w - head, tile) };
        }
        off = next_row(off);
    }
}

/// Byte offset of row `y` from the start of a grid.
#[inline(always)]
fn row_offset(y: u8) -> usize {
    (y % SIDE) as usize * SIDE as usize
}

/// The row below, wrapping at the bottom.
///
/// A grid is 1024 bytes and both sit at a 1024 byte boundary, so an offset needs
/// only masking and the base needs only an `or`. Stepping beats recomputing the
/// row address, which costs a multiply the row loop would otherwise repeat.
#[inline(always)]
fn next_row(off: usize) -> usize {
    (off + SIDE as usize) & (SIDE as usize * SIDE as usize - 1)
}

#[inline]
unsafe fn copy<const WAIT: bool>(dst: *mut u8, src: *const u8, n: u8) {
    for i in 0..n as usize {
        let b = unsafe { *src.add(i) };
        if WAIT {
            wait_blank();
        }
        unsafe { core::ptr::write_volatile(dst.add(i), b) };
    }
}

#[inline]
unsafe fn spread<const WAIT: bool>(dst: *mut u8, n: u8, tile: u8) {
    for i in 0..n as usize {
        if WAIT {
            wait_blank();
        }
        unsafe { core::ptr::write_volatile(dst.add(i), tile) };
    }
}
/// Proof that the attribute plane is the grid mapped at the tilemap addresses.
///
/// The plane is a second 32 by 32 grid in VRAM bank 1, sharing the tilemap's
/// addresses. Each byte holds the palette, the flips, the object priority and
/// the tile bank for the cell of the same coordinates. Pan Docs calls these the
/// BG map attributes, but they reach the window too: whichever layer reads a
/// tilemap reads its attributes alongside.
///
/// They belong to the cell, not to the tile it names. Two cells showing one tile
/// can be drawn from different palettes, and giving a cell a different tile
/// leaves its attributes where they were.
///
/// [`BgAttr::bank`](crate::mmio::cgb::BgAttr::bank) says which VRAM bank the PPU
/// fetches that cell's tile from, a separate question from which bank the CPU
/// has mapped. [`edit_attrs`] settles the second so that a write lands in the
/// plane rather than in the tile indices.
///
/// Handed out by [`edit_attrs`] and bounded to its closure, the way
/// [`Access::Direct`] is. See
/// <https://gbdev.io/pandocs/Tile_Maps.html#bg-map-attributes-cgb-mode-only>.
#[cfg(feature = "cgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "cgb")))]
#[derive(Clone, Copy)]
pub struct AttrAccess<'a> {
    _private: core::marker::PhantomData<&'a ()>,
}

#[cfg(feature = "cgb")]
impl<'a> !Send for AttrAccess<'a> {}
#[cfg(feature = "cgb")]
impl<'a> !Sync for AttrAccess<'a> {}

/// Map the attribute plane and leave it mapped for `f`.
///
/// [`with_vram_bank`](super::with_vram_bank) nested inside `f` puts the mapping
/// back as it leaves, so the token is good again after it. During it the token
/// is not: a write through it then lands in whatever that scope mapped. So does
/// one made after a handler switched the bank and left it switched.
///
/// There is no plane to map on an original Game Boy, and nothing refuses the
/// attempt: `f` writes attribute bytes over the tile indices instead. A
/// cartridge that runs on both machines has to ask [`is_cgb`](crate::is_cgb)
/// first.
///
/// ```ignore
/// map::edit_attrs(|a| {
///     a.set(d, Map::Zero, 3, 3, highlight);
///     a.set(d, Map::Zero, 4, 3, highlight);
/// });
/// ```
#[cfg(feature = "cgb")]
#[cfg_attr(docsrs, doc(cfg(feature = "cgb")))]
#[inline]
pub fn edit_attrs<R>(f: impl FnOnce(AttrAccess<'_>) -> R) -> R {
    crate::ppu::with_vram_bank(crate::ppu::VramBank::One, || {
        f(AttrAccess { _private: core::marker::PhantomData })
    })
}

#[cfg(feature = "cgb")]
impl AttrAccess<'_> {
    /// Give one cell its attributes.
    #[inline]
    pub fn set(self, access: Access<'_>, map: Map, x: u8, y: u8, attr: crate::mmio::cgb::BgAttr) {
        set(access, map, x, y, attr.into_bits());
    }

    /// Give every cell of a rectangle the same attributes.
    ///
    /// # Panics
    ///
    /// As [`fill`].
    #[inline]
    pub fn fill(
        self,
        access: Access<'_>,
        map: Map,
        x: u8,
        y: u8,
        w: u8,
        h: u8,
        attr: crate::mmio::cgb::BgAttr,
    ) {
        fill(access, map, x, y, w, h, attr.into_bits());
    }

    /// Copy a rectangle of attribute bytes in, its top left at `(x, y)`.
    ///
    /// # Panics
    ///
    /// As [`write()`].
    #[inline]
    pub fn write(self, access: Access<'_>, map: Map, x: u8, y: u8, src: AttrGrid<'_>) {
        // Straight to the blit: `write` takes tile indices, and the bytes travel
        // the same way whichever plane is mapped.
        match access {
            Access::Polled => unsafe { blit::<true>(map, x, y, src.0) },
            _ => unsafe { blit::<false>(map, x, y, src.0) },
        }
    }
}
