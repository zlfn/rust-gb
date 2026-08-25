//! Game Boy ROM header configuration and patching.

use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Default, Clone, Copy, Debug)]
pub enum CartridgeType {
    #[default]
    #[serde(alias = "ROM")]
    Rom,
    #[serde(alias = "MBC1")]
    Mbc1,
    #[serde(alias = "MBC1+RAM")]
    Mbc1Ram,
    #[serde(alias = "MBC1+RAM+BATTERY")]
    Mbc1RamBattery,
    #[serde(alias = "MBC2")]
    Mbc2,
    #[serde(alias = "MBC2+BATTERY")]
    Mbc2Battery,
    #[serde(alias = "MBC3")]
    Mbc3,
    #[serde(alias = "MBC3+TIMER+BATTERY")]
    Mbc3TimerBattery,
    #[serde(alias = "MBC3+TIMER+RAM+BATTERY")]
    Mbc3TimerRamBattery,
    #[serde(alias = "MBC3+RAM")]
    Mbc3Ram,
    #[serde(alias = "MBC3+RAM+BATTERY")]
    Mbc3RamBattery,
    #[serde(alias = "MBC5")]
    Mbc5,
    #[serde(alias = "MBC5+RAM")]
    Mbc5Ram,
    #[serde(alias = "MBC5+RAM+BATTERY")]
    Mbc5RamBattery,
    #[serde(alias = "MBC5+RUMBLE")]
    Mbc5Rumble,
    #[serde(alias = "MBC5+RUMBLE+RAM")]
    Mbc5RumbleRam,
    #[serde(alias = "MBC5+RUMBLE+RAM+BATTERY")]
    Mbc5RumbleRamBattery,
}

impl CartridgeType {
    pub fn to_byte(self) -> u8 {
        match self {
            Self::Rom => 0x00,
            Self::Mbc1 => 0x01,
            Self::Mbc1Ram => 0x02,
            Self::Mbc1RamBattery => 0x03,
            Self::Mbc2 => 0x05,
            Self::Mbc2Battery => 0x06,
            Self::Mbc3 => 0x11,
            Self::Mbc3TimerBattery => 0x0F,
            Self::Mbc3TimerRamBattery => 0x10,
            Self::Mbc3Ram => 0x12,
            Self::Mbc3RamBattery => 0x13,
            Self::Mbc5 => 0x19,
            Self::Mbc5Ram => 0x1A,
            Self::Mbc5RamBattery => 0x1B,
            Self::Mbc5Rumble => 0x1C,
            Self::Mbc5RumbleRam => 0x1D,
            Self::Mbc5RumbleRamBattery => 0x1E,
        }
    }

    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x00 => Some(Self::Rom),
            0x01 => Some(Self::Mbc1),
            0x02 => Some(Self::Mbc1Ram),
            0x03 => Some(Self::Mbc1RamBattery),
            0x05 => Some(Self::Mbc2),
            0x06 => Some(Self::Mbc2Battery),
            0x0F => Some(Self::Mbc3TimerBattery),
            0x10 => Some(Self::Mbc3TimerRamBattery),
            0x11 => Some(Self::Mbc3),
            0x12 => Some(Self::Mbc3Ram),
            0x13 => Some(Self::Mbc3RamBattery),
            0x19 => Some(Self::Mbc5),
            0x1A => Some(Self::Mbc5Ram),
            0x1B => Some(Self::Mbc5RamBattery),
            0x1C => Some(Self::Mbc5Rumble),
            0x1D => Some(Self::Mbc5RumbleRam),
            0x1E => Some(Self::Mbc5RumbleRamBattery),
            _ => None,
        }
    }

    /// The highest bank number `switch_bank` can select on this cartridge.
    ///
    /// Each value is the width of the register at `0x2000`, which is the only one
    /// the runtime writes. MBC1 and MBC5 reach further through a second register
    /// (at `0x4000` and `0x3000`), so raising these means widening `switch_bank`
    /// first.
    ///
    /// Bank 0 is never allocated: MBC1, MBC2, and MBC3 read a written `0` as `1`,
    /// and on MBC5, which does map it, bank 0 already holds the resident region.
    pub fn max_bank(self) -> u16 {
        match self {
            // 0x4000-0x7FFF is fixed, so there is no bank to switch to.
            Self::Rom => 0,
            // 5 bits.
            Self::Mbc1 | Self::Mbc1Ram | Self::Mbc1RamBattery => 31,
            // 4 bits.
            Self::Mbc2 | Self::Mbc2Battery => 15,
            // 7 bits.
            Self::Mbc3 | Self::Mbc3TimerBattery | Self::Mbc3TimerRamBattery
            | Self::Mbc3Ram | Self::Mbc3RamBattery => 127,
            // 8 bits, the ninth sitting in the register at 0x3000.
            Self::Mbc5 | Self::Mbc5Ram | Self::Mbc5RamBattery
            | Self::Mbc5Rumble | Self::Mbc5RumbleRam | Self::Mbc5RumbleRamBattery => 255,
        }
    }

    /// The largest ROM the runtime can reach on this cartridge.
    pub fn max_rom_bytes(self) -> usize {
        match self {
            // Both halves are fixed, so 32 KiB and no more.
            Self::Rom => 0x8000,
            _ => (self.max_bank() as usize + 1) * 0x4000,
        }
    }

    /// Whether the cartridge has a switchable window at all.
    pub fn supports_banking(self) -> bool {
        self.max_bank() > 0
    }
}

/// Custom deserializer that accepts both string ("MBC5") and integer (0x19) formats.
pub fn deserialize_cartridge_type<'de, D: serde::Deserializer<'de>>(d: D) -> Result<CartridgeType, D::Error> {
    use serde::de::Error;
    let value = toml::Value::deserialize(d)?;
    match &value {
        toml::Value::String(s) => {
            toml::Value::String(s.clone()).try_into()
                .map_err(|_| D::Error::custom(format!("unknown cartridge type: '{s}'")))
        }
        toml::Value::Integer(n) => {
            let byte = *n as u8;
            CartridgeType::from_byte(byte)
                .ok_or_else(|| D::Error::custom(format!("unknown cartridge type: 0x{byte:02X}")))
        }
        _ => Err(D::Error::custom("cartridge_type must be a string or integer")),
    }
}

const NINTENDO_LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

#[derive(Deserialize)]
struct Header {
    title: String,
    #[serde(default)]
    cgb_flag: CgbFlag,
    #[serde(default)]
    sgb_flag: bool,
    #[serde(default, deserialize_with = "deserialize_cartridge_type")]
    cartridge_type: CartridgeType,
    #[serde(default)]
    ram_size: u8,
    #[serde(default)]
    destination: Destination,
    #[serde(default)]
    old_licensee_code: u8,
    #[serde(default)]
    new_licensee_code: Option<String>,
    #[serde(default)]
    version: u8,
}

#[derive(Deserialize, Default, Clone, Copy, PartialEq, Eq)]
pub enum CgbFlag {
    #[default]
    None,
    Hybrid,
    CgbOnly,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "lowercase")]
enum Destination {
    Japan,
    #[default]
    Worldwide,
}

/// Fixed-region usage and final size of a patched ROM.
pub struct RomInfo {
    /// Final ROM size in bytes (padded to a power of two).
    pub total_bytes: usize,
    /// The region the usage refers to: `"ROM Bank 00"` (banked) or `"ROM"` (32 KiB).
    pub label: &'static str,
    /// Bytes used in the fixed region (up to the last non-`0xFF` byte).
    pub used: usize,
    /// Size of the fixed region in bytes.
    pub limit: usize,
    /// Non-fatal configuration warnings.
    pub warnings: Vec<String>,
    /// The CGB compatibility flag taken from the header.
    pub cgb: CgbFlag,
}

/// A failure while fixing a ROM header.
#[derive(Debug)]
pub enum FixError {
    Io(std::io::Error),
    Parse(String),
    /// The ROM is larger than the cartridge type can address.
    RomTooLarge { bytes: usize, limit: usize },
}

impl std::fmt::Display for FixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FixError::Io(e) => write!(f, "{e}"),
            FixError::Parse(e) => write!(f, "{e}"),
            FixError::RomTooLarge { bytes, limit } => write!(
                f,
                "ROM is {} KiB, over the {} KiB reachable on this cartridge type",
                bytes / 1024,
                limit / 1024,
            ),
        }
    }
}

impl std::error::Error for FixError {}

impl From<std::io::Error> for FixError {
    fn from(e: std::io::Error) -> Self {
        FixError::Io(e)
    }
}

fn rom_size_code(len: usize) -> u8 {
    match len {
        0..=32768 => 0x00,
        0..=65536 => 0x01,
        0..=131072 => 0x02,
        0..=262144 => 0x03,
        0..=524288 => 0x04,
        0..=1048576 => 0x05,
        0..=2097152 => 0x06,
        0..=4194304 => 0x07,
        _ => 0x08,
    }
}

fn pad_to_power_of_two(rom: &mut Vec<u8>) {
    let min_size = 32 * 1024;
    let target = if rom.len() <= min_size {
        min_size
    } else {
        rom.len().next_power_of_two()
    };
    rom.resize(target, 0xFF);
}

/// Patch the cartridge header of the ROM at `rom_path` in place from
/// `header_toml`, then return the fixed-region usage and final size.
pub fn fix(rom_path: &Path, header_toml: &Path) -> Result<RomInfo, FixError> {
    let toml_str = std::fs::read_to_string(header_toml)?;
    let header: Header = toml::from_str(&toml_str).map_err(|e| FixError::Parse(e.to_string()))?;

    let mut rom = std::fs::read(rom_path)?;
    if rom.len() < 0x150 {
        rom.resize(0x150, 0xFF);
    }
    pad_to_power_of_two(&mut rom);

    let limit = header.cartridge_type.max_rom_bytes();
    if rom.len() > limit {
        return Err(FixError::RomTooLarge { bytes: rom.len(), limit });
    }

    // 0x0104-0x0133: Nintendo logo
    rom[0x0104..0x0134].copy_from_slice(&NINTENDO_LOGO);

    // 0x0134-0x0143: Title (15 bytes when the CGB flag is used, else 16)
    let max_title = match header.cgb_flag {
        CgbFlag::None => 16,
        _ => 15,
    };
    let title_bytes = header.title.as_bytes();
    let title_len = title_bytes.len().min(max_title);
    for b in &mut rom[0x0134..0x0144] {
        *b = 0x00;
    }
    rom[0x0134..0x0134 + title_len].copy_from_slice(&title_bytes[..title_len]);

    // 0x0143: CGB flag
    rom[0x0143] = match header.cgb_flag {
        CgbFlag::None => 0x00,
        CgbFlag::Hybrid => 0x80,
        CgbFlag::CgbOnly => 0xC0,
    };

    // 0x0144-0x0145: New licensee code (used when old_licensee == 0x33)
    if let Some(ref code) = header.new_licensee_code {
        let bytes = code.as_bytes();
        rom[0x0144] = bytes.first().copied().unwrap_or(0x00);
        rom[0x0145] = bytes.get(1).copied().unwrap_or(0x00);
    }

    // 0x0146: SGB flag
    rom[0x0146] = if header.sgb_flag { 0x03 } else { 0x00 };

    // 0x0147: Cartridge type
    rom[0x0147] = header.cartridge_type.to_byte();

    // 0x0148: ROM size (from the padded size)
    rom[0x0148] = rom_size_code(rom.len());

    // 0x0149: RAM size
    rom[0x0149] = header.ram_size;

    // 0x014A: Destination code
    rom[0x014A] = match header.destination {
        Destination::Japan => 0x00,
        Destination::Worldwide => 0x01,
    };

    // 0x014B: Old licensee code. 0x33 selects the new licensee at 0x0144-0x0145,
    // and SGB detection also requires 0x33.
    let mut warnings = Vec::new();
    let override_licensee = header.new_licensee_code.is_some() || header.sgb_flag;
    if override_licensee && header.old_licensee_code != 0x00 && header.old_licensee_code != 0x33 {
        let reason = if header.sgb_flag && header.new_licensee_code.is_some() {
            "sgb_flag and new_licensee_code"
        } else if header.sgb_flag {
            "sgb_flag"
        } else {
            "new_licensee_code"
        };
        warnings.push(format!(
            "old_licensee_code 0x{:02X} overridden to 0x33 (required by {reason})",
            header.old_licensee_code,
        ));
    }
    rom[0x014B] = if override_licensee {
        0x33
    } else {
        header.old_licensee_code
    };

    // 0x014C: Mask ROM version
    rom[0x014C] = header.version;

    // 0x014D: Header checksum
    let mut hck: u8 = 0;
    for &b in &rom[0x0134..=0x014C] {
        hck = hck.wrapping_sub(b).wrapping_sub(1);
    }
    rom[0x014D] = hck;

    // 0x014E-0x014F: Global checksum (sum of all bytes, with these two as zero)
    rom[0x014E] = 0x00;
    rom[0x014F] = 0x00;
    let mut gck: u16 = 0;
    for &b in rom.iter() {
        gck = gck.wrapping_add(b as u16);
    }
    rom[0x014E] = (gck >> 8) as u8;
    rom[0x014F] = gck as u8;

    std::fs::write(rom_path, &rom)?;

    // Fixed-region usage (last non-0xFF byte). With an MBC, only ROM Bank 00
    // (0x0000-0x3FFF) is permanently mapped; 0x4000-0x7FFF is the switchable
    // window. Without banking the whole 0x0000-0x7FFF (32 KiB) is fixed.
    let (label, limit) = if header.cartridge_type.supports_banking() {
        ("ROM Bank 00", 0x4000usize)
    } else {
        ("ROM", 0x8000usize)
    };
    let used = rom[..limit.min(rom.len())]
        .iter()
        .rposition(|&b| b != 0xFF)
        .map(|i| i + 1)
        .unwrap_or(0);

    Ok(RomInfo {
        total_bytes: rom.len(),
        label,
        used,
        limit,
        warnings,
        cgb: header.cgb_flag,
    })
}
