//! Game Boy Color hardware registers.
//!
//! The accessors are safe but take effect only on CGB hardware. The undocumented
//! registers `0xFF72..=0xFF75` are omitted; they have no defined function.

use bitfield_struct::bitfield;
use voladdress::{Safe, VolAddress};

/// CPU speed switch (`KEY1`).
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7   | `double_speed` | RO  | Currently in double-speed mode. |
/// | 6-1 | —              |     | Unused. |
/// | 0   | `armed`        | R/W | Arm a speed switch; it takes effect on the next `STOP`. |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct SpeedSwitch {
    /// Arm a speed switch; it takes effect on the next `STOP`.
    pub armed: bool,
    #[bits(6)]
    __: u8,
    /// Currently in double-speed mode (read-only).
    #[bits(1, access = RO)]
    pub double_speed: bool,
}

/// VRAM DMA length, mode, and start (`HDMA5`).
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7   | `hblank` | R/W | Write: set = HBlank DMA, clear = general-purpose. Read: set = no transfer active. |
/// | 6-0 | `blocks` | R/W | Transfer length in 16-byte blocks, minus one (0 = one block). |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct HdmaCtrl {
    /// Transfer length in 16-byte blocks, minus one (`0` means one block).
    #[bits(7)]
    pub blocks: u8,
    /// On write: `true` selects HBlank DMA, `false` general-purpose DMA. On
    /// read: `true` means no transfer is currently active.
    pub hblank: bool,
}

/// Infrared communications port (`RP`).
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7-6 | `read_enable` | R/W | Set both bits to read the receiver. |
/// | 5-2 | —             |     | Unused. |
/// | 1   | `receiving`   | RO  | Receiving an IR signal (reads 0 while a signal is seen). |
/// | 0   | `led_on`      | R/W | Turn the IR LED on. |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct Infrared {
    /// Turn the IR LED on.
    pub led_on: bool,
    /// Receiving an IR signal (read-only; reads `0` while a signal is seen).
    #[bits(1, access = RO)]
    pub receiving: bool,
    #[bits(4)]
    __: u8,
    /// Read enable: set both bits to read the receiver.
    #[bits(2)]
    pub read_enable: u8,
}

/// Per-cell tilemap attributes, in VRAM bank 1 at the tilemap addresses.
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7   | `priority` | R/W | Set draws this cell's color indices 1-3 over objects. |
/// | 6   | `y_flip`   | R/W | Mirror vertically. |
/// | 5   | `x_flip`   | R/W | Mirror horizontally. |
/// | 4   | —          | R/W | Ignored by the hardware. |
/// | 3   | `bank`     | R/W | Fetch this cell's tile from VRAM bank 1. |
/// | 2-0 | `palette`  | R/W | Which of the eight background palettes. |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct BgAttr {
    /// Which of the eight background palettes.
    #[bits(3)]
    pub palette: u8,
    /// Fetch this cell's tile from VRAM bank 1 rather than bank 0.
    pub bank: bool,
    #[bits(1)]
    __: u8,
    /// Mirror horizontally.
    pub x_flip: bool,
    /// Mirror vertically.
    pub y_flip: bool,
    /// Draw this cell's color indices 1-3 over objects.
    pub priority: bool,
}

/// Color palette index (`BCPS` / `OCPS`).
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7   | `auto_increment` | R/W | Auto-increment the address after each data-port write. |
/// | 6   | —                |     | Unused. |
/// | 5-0 | `address`        | R/W | Byte offset into palette memory reached through the data port. |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct PaletteIndex {
    /// Byte offset into palette memory reached through the data port.
    #[bits(6)]
    pub address: u8,
    #[bits(1)]
    __: u8,
    /// Auto-increment the address after each write to the data port.
    pub auto_increment: bool,
}

/// Packed PCM amplitudes (`PCM12` / `PCM34`). Read-only.
///
/// | Bit | Field | Access | Meaning |
/// |-----|-------|--------|---------|
/// | 7-4 | `high` | RO | Upper channel's amplitude (CH2 for `PCM12`, CH4 for `PCM34`). |
/// | 3-0 | `low`  | RO | Lower channel's amplitude (CH1 for `PCM12`, CH3 for `PCM34`). |
#[bitfield(u8)]
#[derive(PartialEq, Eq)]
pub struct PcmAmplitudes {
    /// Lower channel's amplitude (CH1 for `PCM12`, CH3 for `PCM34`).
    #[bits(4)]
    pub low: u8,
    /// Upper channel's amplitude (CH2 for `PCM12`, CH4 for `PCM34`).
    #[bits(4)]
    pub high: u8,
}

/// CPU mode select. Mostly a boot-ROM / DMG-compatibility register, locked
/// once the boot ROM hands off.
pub const KEY0: VolAddress<u8, Safe, Safe> = unsafe { VolAddress::new(0xFF4C) };
/// Prepare speed switch (double-speed mode).
pub const KEY1: VolAddress<SpeedSwitch, Safe, Safe> = unsafe { VolAddress::new(0xFF4D) };
/// VRAM bank select (bit 0).
pub const VBK: VolAddress<u8, Safe, Safe> = unsafe { VolAddress::new(0xFF4F) };

/// VRAM DMA source, high byte. Write-only.
pub const HDMA1: VolAddress<u8, (), Safe> = unsafe { VolAddress::new(0xFF51) };
/// VRAM DMA source, low byte. Write-only.
pub const HDMA2: VolAddress<u8, (), Safe> = unsafe { VolAddress::new(0xFF52) };
/// VRAM DMA destination, high byte. Write-only.
pub const HDMA3: VolAddress<u8, (), Safe> = unsafe { VolAddress::new(0xFF53) };
/// VRAM DMA destination, low byte. Write-only.
pub const HDMA4: VolAddress<u8, (), Safe> = unsafe { VolAddress::new(0xFF54) };
/// VRAM DMA length/mode/start.
pub const HDMA5: VolAddress<HdmaCtrl, Safe, Safe> = unsafe { VolAddress::new(0xFF55) };

/// Infrared communications port.
pub const RP: VolAddress<Infrared, Safe, Safe> = unsafe { VolAddress::new(0xFF56) };

/// Background color palette index.
pub const BCPS: VolAddress<PaletteIndex, Safe, Safe> = unsafe { VolAddress::new(0xFF68) };
/// Background color palette data at the current `BCPS` index.
pub const BCPD: VolAddress<u8, Safe, Safe> = unsafe { VolAddress::new(0xFF69) };
/// Object color palette index.
pub const OCPS: VolAddress<PaletteIndex, Safe, Safe> = unsafe { VolAddress::new(0xFF6A) };
/// Object color palette data at the current `OCPS` index.
pub const OCPD: VolAddress<u8, Safe, Safe> = unsafe { VolAddress::new(0xFF6B) };

/// Object priority mode: 0 = CGB-style (by OAM index), 1 = DMG-style (by X).
pub const OPRI: VolAddress<u8, Safe, Safe> = unsafe { VolAddress::new(0xFF6C) };

/// WRAM bank select (bits 0-2) for the `0xD000..=0xDFFF` window.
pub const SVBK: VolAddress<u8, Safe, Safe> = unsafe { VolAddress::new(0xFF70) };

/// PCM amplitudes for sound channels 1 and 2. Read-only.
pub const PCM12: VolAddress<PcmAmplitudes, Safe, ()> = unsafe { VolAddress::new(0xFF76) };
/// PCM amplitudes for sound channels 3 and 4. Read-only.
pub const PCM34: VolAddress<PcmAmplitudes, Safe, ()> = unsafe { VolAddress::new(0xFF77) };

const _: () = {
    assert!(SpeedSwitch::new().with_armed(true).into_bits() == 0b0000_0001);
    assert!(SpeedSwitch::from_bits(0b1000_0000).double_speed());
    assert!(HdmaCtrl::new().with_hblank(true).into_bits() == 0b1000_0000);
    assert!(PaletteIndex::new().with_auto_increment(true).into_bits() == 0b1000_0000);
    assert!(Infrared::new().with_read_enable(0b11).into_bits() == 0b1100_0000);
    assert!(PcmAmplitudes::new().with_high(0xF).into_bits() == 0b1111_0000);
};
