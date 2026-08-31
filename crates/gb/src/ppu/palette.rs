//! Palettes: what the two bits of a pixel turn into.
//!
//! A tile pixel is two bits, so it names a colour *index* rather than a colour.
//! What that index becomes is the palette's business, which is why the same tile
//! can be drawn light in one place and dark in another.
//!
//! On the original Game Boy the four shades are fixed in hardware and a palette only
//! chooses which index gets which of them. On the Game Boy Color a palette holds
//! four real colours. See <https://gbdev.io/pandocs/Palettes.html>.
//!
//! Index 0 is transparent for objects, which leaves the lowest two bits of an
//! object palette unused.
//!
//! An original Game Boy ignores the colour registers and keeps drawing with the shades
//! [`set_background`] chose, so a cartridge that runs on both machines can set
//! colours without asking which it is on.

use crate::mmio::{BGP, OBP0, OBP1, Palette};

/// Which of the two object palettes, as [`OamAttr::dmg_palette`] picks between.
///
/// [`OamAttr::dmg_palette`]: crate::mmio::OamAttr::dmg_palette
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ObjSlot {
    /// `OBP0`.
    Zero,
    /// `OBP1`.
    One,
}

/// The palette the background and window are drawn with.
#[inline]
pub fn background() -> Palette {
    BGP.read()
}

/// Set the palette the background and window are drawn with.
#[inline]
pub fn set_background(palette: Palette) {
    BGP.write(palette);
}

/// One of the two object palettes.
#[inline]
pub fn object(slot: ObjSlot) -> Palette {
    match slot {
        ObjSlot::Zero => OBP0.read(),
        ObjSlot::One => OBP1.read(),
    }
}

/// Set one of the two object palettes.
///
/// Its shade for index 0 is ignored: that index is transparent.
#[inline]
pub fn set_object(slot: ObjSlot, palette: Palette) {
    match slot {
        ObjSlot::Zero => OBP0.write(palette),
        ObjSlot::One => OBP1.write(palette),
    }
}

#[cfg(feature = "cgb")]
pub use cgb::*;

#[cfg(feature = "cgb")]
mod cgb {
    use crate::mmio::cgb::{BCPD, BCPS, OCPD, OCPS, PaletteIndex};
    use crate::ppu::{Access, wait_blank};

    /// Palettes of each kind: this many for the background, and as many again
    /// for objects.
    pub const PALETTES: u8 = 8;

    /// Colours in one palette.
    pub const COLORS: u8 = 4;

    /// One colour, five bits per channel.
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
    pub struct Color(u16);

    impl Color {
        /// Build a colour. Each channel runs 0 to 31; anything above is masked.
        pub const fn new(red: u8, green: u8, blue: u8) -> Self {
            Color(
                (red as u16 & 0x1F) | ((green as u16 & 0x1F) << 5) | ((blue as u16 & 0x1F) << 10),
            )
        }

        /// Red, 0 to 31.
        pub const fn red(self) -> u8 {
            (self.0 & 0x1F) as u8
        }

        /// Green, 0 to 31.
        pub const fn green(self) -> u8 {
            ((self.0 >> 5) & 0x1F) as u8
        }

        /// Blue, 0 to 31.
        pub const fn blue(self) -> u8 {
            ((self.0 >> 10) & 0x1F) as u8
        }
    }

    /// Set one background palette.
    ///
    /// # Panics
    ///
    /// If `palette` is [`PALETTES`] or beyond.
    pub fn set_background_colors(access: Access<'_>, palette: u8, colors: &[Color; COLORS as usize]) {
        assert!(palette < PALETTES);
        write(access, BCPS, BCPD, palette, colors);
    }

    /// Set one object palette.
    ///
    /// Its first colour is never drawn: index 0 is transparent for objects.
    ///
    /// # Panics
    ///
    /// If `palette` is [`PALETTES`] or beyond.
    pub fn set_object_colors(access: Access<'_>, palette: u8, colors: &[Color; COLORS as usize]) {
        assert!(palette < PALETTES);
        write(access, OCPS, OCPD, palette, colors);
    }

    type Index = voladdress::VolAddress<PaletteIndex, voladdress::Safe, voladdress::Safe>;
    type Data = voladdress::VolAddress<u8, voladdress::Safe, voladdress::Safe>;

    #[inline]
    fn write(
        access: Access<'_>,
        index: Index,
        data: Data,
        palette: u8,
        colors: &[Color; COLORS as usize],
    ) {
        // The index advances on every write to the data port, including one the
        // PPU threw away, so a byte lost to mode 3 does not go missing: it shifts
        // every colour after it. That is why `Polled` waits here at all, where a
        // tilemap write could simply let the byte drop.
        //
        // The index register itself is reachable in every mode and needs no
        // wait of its own.
        index.write(
            PaletteIndex::new()
                .with_address(palette * COLORS * 2)
                .with_auto_increment(true),
        );
        match access {
            Access::Polled => stream::<true>(data, colors),
            _ => stream::<false>(data, colors),
        }
    }

    #[inline]
    fn stream<const WAIT: bool>(data: Data, colors: &[Color; COLORS as usize]) {
        for c in colors {
            for b in c.0.to_le_bytes() {
                if WAIT {
                    wait_blank();
                }
                data.write(b);
            }
        }
    }
}
