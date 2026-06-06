//! Display and PPU: control, palettes, scroll, and video memory.

use bitfield_struct::{bitenum, bitfield};
use voladdress::{Safe, Unsafe, VolAddress, VolBlock, VolGrid2d};

/// One 8x8 2bpp tile: 16 bytes, two bytes per row.
pub type Tile = [u8; 16];

/// Sprite attribute byte (`OamEntry::attr`).
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7   | `priority`    | R/W | Set draws the sprite behind BG colors 1-3. |
/// | 6   | `y_flip`      | R/W | Mirror vertically. |
/// | 5   | `x_flip`      | R/W | Mirror horizontally. |
/// | 4   | `dmg_palette` | R/W | DMG palette: clear = OBP0, set = OBP1. |
/// | 3   | `cgb_bank`    | R/W | CGB tile VRAM bank. |
/// | 2-0 | `cgb_palette` | R/W | CGB object palette (0-7). |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct OamAttr {
    /// CGB object palette (0-7).
    #[bits(3)]
    pub cgb_palette: u8,
    /// CGB tile VRAM bank.
    pub cgb_bank: bool,
    /// DMG palette: `false` = OBP0, `true` = OBP1.
    pub dmg_palette: bool,
    /// Mirror horizontally.
    pub x_flip: bool,
    /// Mirror vertically.
    pub y_flip: bool,
    /// Priority: `true` draws the sprite behind BG colors 1-3.
    pub priority: bool,
}

/// A single OAM sprite entry (4 bytes).
///
/// | Byte | Field | Meaning |
/// |------|-------|---------|
/// | 0 | `y`    | Screen Y position, offset by 16. |
/// | 1 | `x`    | Screen X position, offset by 8. |
/// | 2 | `tile` | Tile index into the sprite tile data. |
/// | 3 | `attr` | Attribute flags (`OamAttr`). |
#[derive(Clone, Copy)]
#[repr(C)]
pub struct OamEntry {
    /// Screen Y position, offset by 16 (0 and >= 160 hide the sprite).
    pub y: u8,
    /// Screen X position, offset by 8 (0 and >= 168 hide the sprite).
    pub x: u8,
    /// Tile index into the sprite tile data.
    pub tile: u8,
    /// Attribute flags: palette, flip, priority, and CGB bank/palette.
    pub attr: OamAttr,
}

/// LCD control (`LCDC`). Each field's name describes the meaning when *set*.
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7 | `lcd_enable`          | R/W | LCD and PPU enable. |
/// | 6 | `window_tilemap_high` | R/W | Window tile map at 0x9C00 instead of 0x9800. |
/// | 5 | `window_enable`       | R/W | Window enable. |
/// | 4 | `tiledata_8000`       | R/W | BG/window tile data from the 0x8000 area instead of 0x8800. |
/// | 3 | `bg_tilemap_high`     | R/W | BG tile map at 0x9C00 instead of 0x9800. |
/// | 2 | `obj_tall`            | R/W | Object size 8x16 instead of 8x8. |
/// | 1 | `obj_enable`          | R/W | Object (sprite) rendering enable. |
/// | 0 | `bg_window_enable`    | R/W | BG and window enable (DMG); BG/window master priority (CGB). |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct Lcdc {
    /// BG and window enable (DMG); BG/window master priority (CGB).
    pub bg_window_enable: bool,
    /// Object (sprite) rendering enable.
    pub obj_enable: bool,
    /// Object size is 8x16 instead of 8x8.
    pub obj_tall: bool,
    /// BG tile map at `0x9C00` instead of `0x9800`.
    pub bg_tilemap_high: bool,
    /// BG/window tile data uses the `0x8000` (unsigned) instead of `0x8800` area.
    pub tiledata_8000: bool,
    /// Window enable.
    pub window_enable: bool,
    /// Window tile map at `0x9C00` instead of `0x9800`.
    pub window_tilemap_high: bool,
    /// LCD and PPU enable.
    pub lcd_enable: bool,
}

/// Current PPU mode (`STAT` bits 1-0).
#[bitenum(all = false)]
#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum PpuMode {
    /// Mode 0: horizontal blank.
    HBlank = 0,
    /// Mode 1: vertical blank.
    VBlank = 1,
    /// Mode 2: OAM scan.
    OamScan = 2,
    /// Mode 3: drawing pixels.
    #[fallback]
    Drawing = 3,
}

/// LCD status (`STAT`).
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7   | —           |     | Unused. |
/// | 6   | `lyc_int`   | R/W | Raise the STAT interrupt on LYC=LY. |
/// | 5   | `mode2_int` | R/W | Raise the STAT interrupt on mode 2 (OAM scan). |
/// | 4   | `mode1_int` | R/W | Raise the STAT interrupt on mode 1 (VBlank). |
/// | 3   | `mode0_int` | R/W | Raise the STAT interrupt on mode 0 (HBlank). |
/// | 2   | `lyc_eq_ly` | RO  | Set while LY equals LYC. |
/// | 1-0 | `mode`      | RO  | Current PPU mode. |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct Stat {
    /// Current PPU mode (read-only).
    #[bits(2, access = RO)]
    pub mode: PpuMode,
    /// Set while `LY` equals `LYC` (read-only).
    #[bits(1, access = RO)]
    pub lyc_eq_ly: bool,
    /// Raise the STAT interrupt on mode 0 (HBlank).
    pub mode0_int: bool,
    /// Raise the STAT interrupt on mode 1 (VBlank).
    pub mode1_int: bool,
    /// Raise the STAT interrupt on mode 2 (OAM scan).
    pub mode2_int: bool,
    /// Raise the STAT interrupt on the LYC=LY condition.
    pub lyc_int: bool,
    #[bits(1)]
    __: u8,
}

/// One of the four DMG gray shades.
#[bitenum(all = false)]
#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub enum Shade {
    /// Lightest.
    White = 0,
    LightGray = 1,
    DarkGray = 2,
    /// Darkest.
    #[fallback]
    Black = 3,
}

/// A DMG palette (`BGP` / `OBP0` / `OBP1`): one shade per color index. Object
/// palettes ignore color index 0 (transparent).
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7-6 | `id3` | R/W | Shade for color index 3. |
/// | 5-4 | `id2` | R/W | Shade for color index 2. |
/// | 3-2 | `id1` | R/W | Shade for color index 1. |
/// | 1-0 | `id0` | R/W | Shade for color index 0. |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct Palette {
    /// Shade for color index 0.
    #[bits(2)]
    pub id0: Shade,
    /// Shade for color index 1.
    #[bits(2)]
    pub id1: Shade,
    /// Shade for color index 2.
    #[bits(2)]
    pub id2: Shade,
    /// Shade for color index 3.
    #[bits(2)]
    pub id3: Shade,
}

// ── Registers ───────────────────────────────────────────────────────────────

/// LCD control: display/window/sprite/background enables and tile sources.
///
/// # Safety
///
/// Writing is `unsafe`: clearing the `lcd_enable` bit (display off) outside
/// VBlank can damage the panel and desyncs the PPU.
pub const LCDC: VolAddress<Lcdc, Safe, Unsafe> = unsafe { VolAddress::new(0xFF40) };
/// LCD status: PPU mode and STAT interrupt sources.
pub const STAT: VolAddress<Stat, Safe, Safe> = unsafe { VolAddress::new(0xFF41) };
/// Background scroll Y.
pub const SCY: VolAddress<u8, Safe, Safe> = unsafe { VolAddress::new(0xFF42) };
/// Background scroll X.
pub const SCX: VolAddress<u8, Safe, Safe> = unsafe { VolAddress::new(0xFF43) };
/// Current scanline. Read-only.
pub const LY: VolAddress<u8, Safe, ()> = unsafe { VolAddress::new(0xFF44) };
/// Scanline compare value for the LYC=LY STAT interrupt.
pub const LYC: VolAddress<u8, Safe, Safe> = unsafe { VolAddress::new(0xFF45) };
/// OAM DMA: writing a high byte copies 160 bytes from `xx00` into OAM.
///
/// # Safety
///
/// Writing is `unsafe`: during the transfer the CPU may only access HRAM, so it
/// must be started from a wait routine running there.
pub const DMA: VolAddress<u8, Safe, Unsafe> = unsafe { VolAddress::new(0xFF46) };
/// Background palette (DMG).
pub const BGP: VolAddress<Palette, Safe, Safe> = unsafe { VolAddress::new(0xFF47) };
/// Object palette 0 (DMG).
pub const OBP0: VolAddress<Palette, Safe, Safe> = unsafe { VolAddress::new(0xFF48) };
/// Object palette 1 (DMG).
pub const OBP1: VolAddress<Palette, Safe, Safe> = unsafe { VolAddress::new(0xFF49) };
/// Window Y position.
pub const WY: VolAddress<u8, Safe, Safe> = unsafe { VolAddress::new(0xFF4A) };
/// Window X position plus 7.
pub const WX: VolAddress<u8, Safe, Safe> = unsafe { VolAddress::new(0xFF4B) };

// ── Video memory ────────────────────────────────────────────────────────────

/// Tile data: 384 tiles at `0x8000..0x97FF` (shared by BG and sprites on DMG).
pub const VRAM_TILES: VolBlock<Tile, Safe, Safe, 384> =
    unsafe { VolBlock::new(0x8000) };
/// Tile map 0: a 32x32 grid of tile indices at `0x9800`, indexed `(x, y)`.
pub const TILEMAP_0: VolGrid2d<u8, Safe, Safe, 32, 32> =
    unsafe { VolGrid2d::new(0x9800) };
/// Tile map 1: a 32x32 grid of tile indices at `0x9C00`, indexed `(x, y)`.
pub const TILEMAP_1: VolGrid2d<u8, Safe, Safe, 32, 32> =
    unsafe { VolGrid2d::new(0x9C00) };
/// Object attribute memory: 40 sprite entries at `0xFE00..=0xFE9F`.
pub const OAM: VolBlock<OamEntry, Safe, Safe, 40> =
    unsafe { VolBlock::new(0xFE00) };

const _: () = {
    assert!(Lcdc::new().with_lcd_enable(true).into_bits() == 0b1000_0000);
    assert!(Lcdc::new().with_bg_window_enable(true).into_bits() == 0b0000_0001);
    assert!(matches!(Stat::from_bits(0b0000_0011).mode(), PpuMode::Drawing));
    assert!(Stat::new().with_lyc_int(true).into_bits() == 0b0100_0000);
    assert!(Palette::new().with_id3(Shade::Black).into_bits() == 0b1100_0000);
    assert!(Palette::new().with_id0(Shade::Black).into_bits() == 0b0000_0011);
    assert!(OamAttr::new().with_priority(true).into_bits() == 0b1000_0000);
    assert!(OamAttr::new().with_y_flip(true).into_bits() == 0b0100_0000);
    assert!(OamAttr::new().with_x_flip(true).into_bits() == 0b0010_0000);
    assert!(OamAttr::new().with_cgb_palette(0x7).into_bits() == 0b0000_0111);
};
