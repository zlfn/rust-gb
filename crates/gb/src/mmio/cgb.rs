//! Game Boy Color hardware registers, compiled in only with the `cgb` feature.
//! These accessors are safe but only take effect on CGB hardware. The
//! undocumented registers `0xFF72..=0xFF75` are omitted; they have no defined
//! function.

use bitfield_struct::bitfield;
use voladdress::{Safe, VolAddress};

/// Speed switch (`KEY1`).
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

/// VRAM DMA length/mode/start (`HDMA5`).
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

/// Color palette index (`BCPS` / `OCPS`).
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

/// Packed PCM amplitudes (`PCM12` / `PCM34`).
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
