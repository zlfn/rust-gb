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
    #[serde(alias = "MBC7")]
    #[serde(alias = "MBC7+SENSOR+RUMBLE+RAM+BATTERY")]
    Mbc7,
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
            Self::Mbc7 => 0x22,
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
            0x22 => Some(Self::Mbc7),
            _ => None,
        }
    }

    /// The highest bank number `switch_bank` can select on this cartridge.
    ///
    /// The narrow values are the width of the register at `0x2000`, which is all the
    /// runtime writes by default. `wide` reports what it reaches once the build also
    /// writes the cartridge's second bank register, which `cargo-gb` enables from
    /// `wide_banks` in `header.toml`.
    ///
    /// Bank 0 is never allocated: MBC1, MBC2, and MBC3 read a written `0` as `1`,
    /// and on MBC5, which does map it, bank 0 already holds the resident region.
    pub fn max_bank(self, wide: bool) -> u16 {
        match self {
            // 0x4000-0x7FFF is fixed, so there is no bank to switch to.
            Self::Rom => 0,
            // 5 bits, plus 2 more at 0x4000 in banking mode 0.
            Self::Mbc1 | Self::Mbc1Ram | Self::Mbc1RamBattery => {
                if wide { 127 } else { 31 }
            }
            // 4 bits.
            Self::Mbc2 | Self::Mbc2Battery => 15,
            // 7 bits.
            Self::Mbc3 | Self::Mbc3TimerBattery | Self::Mbc3TimerRamBattery
            | Self::Mbc3Ram | Self::Mbc3RamBattery | Self::Mbc7 => 127,
            // 8 bits, plus the ninth at 0x3000.
            Self::Mbc5 | Self::Mbc5Ram | Self::Mbc5RamBattery
            | Self::Mbc5Rumble | Self::Mbc5RumbleRam | Self::Mbc5RumbleRamBattery => {
                if wide { 511 } else { 255 }
            }
        }
    }

    /// Bank numbers within [`max_bank`](Self::max_bank) that the cartridge cannot map.
    ///
    /// Only MBC1 has any: its low five bits read as `1` when written `0`, so the
    /// three numbers whose low bits are zero alias onto their successors.
    pub fn excluded_banks(self, wide: bool) -> Vec<u16> {
        match self {
            Self::Mbc1 | Self::Mbc1Ram | Self::Mbc1RamBattery if wide => {
                vec![0x20, 0x40, 0x60]
            }
            _ => Vec::new(),
        }
    }

    /// The `gb_wide_bank` cfg value for this cartridge, or `None` when it has no
    /// second bank register.
    pub fn wide_bank_cfg(self) -> Option<&'static str> {
        match self {
            Self::Mbc1 | Self::Mbc1Ram | Self::Mbc1RamBattery => Some("mbc1"),
            Self::Mbc5 | Self::Mbc5Ram | Self::Mbc5RamBattery
            | Self::Mbc5Rumble | Self::Mbc5RumbleRam | Self::Mbc5RumbleRamBattery => Some("mbc5"),
            _ => None,
        }
    }

    /// The largest ROM the runtime can reach on this cartridge.
    pub fn max_rom_bytes(self, wide: bool) -> usize {
        match self {
            // Both halves are fixed, so 32 KiB and no more.
            Self::Rom => 0x8000,
            _ => (self.max_bank(wide) as usize + 1) * 0x4000,
        }
    }

    /// Whether the cartridge has a switchable window at all.
    pub fn supports_banking(self) -> bool {
        self.max_bank(false) > 0
    }

    /// The controller's name, for the `gb_pak_mbc` cfg.
    pub fn mbc(self) -> Option<&'static str> {
        match self {
            Self::Rom => None,
            Self::Mbc1 | Self::Mbc1Ram | Self::Mbc1RamBattery => Some("mbc1"),
            Self::Mbc2 | Self::Mbc2Battery => Some("mbc2"),
            Self::Mbc3
            | Self::Mbc3TimerBattery
            | Self::Mbc3TimerRamBattery
            | Self::Mbc3Ram
            | Self::Mbc3RamBattery => Some("mbc3"),
            Self::Mbc5
            | Self::Mbc5Ram
            | Self::Mbc5RamBattery
            | Self::Mbc5Rumble
            | Self::Mbc5RumbleRam
            | Self::Mbc5RumbleRamBattery => Some("mbc5"),
            Self::Mbc7 => Some("mbc7"),
        }
    }

    /// Whether the cartridge carries a clock.
    pub fn has_rtc(self) -> bool {
        matches!(self, Self::Mbc3TimerBattery | Self::Mbc3TimerRamBattery)
    }

    /// Whether the cartridge carries a rumble motor.
    pub fn has_rumble(self) -> bool {
        matches!(
            self,
            Self::Mbc5Rumble | Self::Mbc5RumbleRam | Self::Mbc5RumbleRamBattery
        )
    }

    /// Whether the cartridge carries the MBC7 sensor and its EEPROM.
    pub fn has_tilt(self) -> bool {
        matches!(self, Self::Mbc7)
    }

    /// The most 8 KiB save-RAM banks this cartridge can reach.
    ///
    /// Zero for one with no save RAM, which includes MBC2, whose 512 half-bytes
    /// live inside the controller, and MBC7, which saves to an EEPROM instead.
    /// A rumble cartridge reaches eight rather than sixteen, the motor having
    /// taken the bank register's fourth bit.
    pub fn max_sram_banks(self) -> u8 {
        match self {
            Self::Mbc1Ram | Self::Mbc1RamBattery => 4,
            Self::Mbc3Ram | Self::Mbc3RamBattery | Self::Mbc3TimerRamBattery => 4,
            Self::Mbc5Ram | Self::Mbc5RamBattery => 16,
            Self::Mbc5RumbleRam | Self::Mbc5RumbleRamBattery => 8,
            _ => 0,
        }
    }
}

/// The 8 KiB banks a `ram_size` header byte asks for, or `None` if no size uses it.
///
/// `0x01` is one of those: unofficial documents call it 2 KiB, but no cartridge
/// ever carried a chip that size.
pub fn sram_banks(ram_size: u8) -> Option<u8> {
    match ram_size {
        0x00 => Some(0),
        0x02 => Some(1),
        0x03 => Some(4),
        0x04 => Some(16),
        0x05 => Some(8),
        _ => None,
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

/// What `header.toml` says about the cartridge, for the tools that need it before
/// the ROM exists.
#[derive(Clone, Debug)]
pub struct Cartridge {
    /// The highest bank the build can select.
    pub max_bank: u16,
    /// Bank numbers within [`max_bank`](Self::max_bank) the cartridge cannot map.
    pub excluded: Vec<u16>,
    /// The `gb_wide_bank` cfg the cartridge needs, once `wide_banks` is on.
    pub wide: Option<&'static str>,
    /// The controller's name, or `None` for a cartridge without one.
    pub mbc: Option<&'static str>,
    /// 8 KiB save-RAM banks, from `ram_size`.
    pub sram_banks: u8,
    /// The cartridge carries a clock.
    pub rtc: bool,
    /// The cartridge carries a rumble motor.
    pub rumble: bool,
    /// The cartridge carries the MBC7 sensor and its EEPROM.
    pub tilt: bool,
}

/// Read what `header.toml` says about the cartridge.
///
/// Returns `Ok(None)` when the file cannot be read or parsed, leaving the caller to
/// fall back. A `ram_size` the cartridge cannot carry, or `wide_banks` on one with
/// no second bank register, is an error.
pub fn read_cartridge(header_toml: &Path) -> Result<Option<Cartridge>, FixError> {
    #[derive(Deserialize)]
    struct Fields {
        #[serde(default, deserialize_with = "deserialize_cartridge_type")]
        cartridge_type: CartridgeType,
        #[serde(default)]
        ram_size: u8,
        #[serde(default)]
        wide_banks: bool,
    }
    let Ok(content) = std::fs::read_to_string(header_toml) else {
        return Ok(None);
    };
    let Ok(f) = toml::from_str::<Fields>(&content) else {
        return Ok(None);
    };
    let kind = f.cartridge_type;

    let wide = if f.wide_banks {
        match kind.wide_bank_cfg() {
            Some(k) => Some(k),
            None => return Err(FixError::Parse(NO_SECOND_REGISTER.into())),
        }
    } else {
        None
    };

    let Some(banks) = sram_banks(f.ram_size) else {
        return Err(FixError::Parse(format!(
            "ram_size 0x{:02X} is not a size any cartridge carries",
            f.ram_size
        )));
    };
    let most = kind.max_sram_banks();
    if banks > most {
        return Err(FixError::Parse(if most == 0 {
            format!("this cartridge has no save RAM, so ram_size must be 0x00")
        } else {
            format!("this cartridge reaches {most} save-RAM bank(s), and ram_size asks for {banks}")
        }));
    }
    if wide == Some("mbc1") && banks > 1 {
        return Err(FixError::Parse(MBC1_WIDE_AND_SRAM.into()));
    }

    Ok(Some(Cartridge {
        max_bank: kind.max_bank(f.wide_banks),
        excluded: kind.excluded_banks(f.wide_banks),
        wide,
        mbc: kind.mbc(),
        sram_banks: banks,
        rtc: kind.has_rtc(),
        rumble: kind.has_rumble(),
        tilt: kind.has_tilt(),
    }))
}

const NO_SECOND_REGISTER: &str =
    "wide_banks needs a cartridge with a second bank register (MBC1 or MBC5)";

const MBC1_WIDE_AND_SRAM: &str =
    "MBC1 spends the same two register bits on ROM banks above 512 KiB and on SRAM \
     banks, so wide_banks and more than one save-RAM bank cannot both be set";

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
    #[serde(default)]
    wide_banks: bool,
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

    if header.wide_banks && header.cartridge_type.wide_bank_cfg().is_none() {
        return Err(FixError::Parse(NO_SECOND_REGISTER.into()));
    }

    let limit = header.cartridge_type.max_rom_bytes(header.wide_banks);
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
