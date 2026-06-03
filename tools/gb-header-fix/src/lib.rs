//! Shared types for Game Boy ROM header configuration.

use serde::Deserialize;

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

    pub fn max_banks(self) -> u16 {
        match self {
            Self::Rom => 0,
            Self::Mbc1 | Self::Mbc1Ram | Self::Mbc1RamBattery => 32,
            Self::Mbc2 | Self::Mbc2Battery => 16,
            Self::Mbc3 | Self::Mbc3TimerBattery | Self::Mbc3TimerRamBattery
            | Self::Mbc3Ram | Self::Mbc3RamBattery => 128,
            Self::Mbc5 | Self::Mbc5Ram | Self::Mbc5RamBattery
            | Self::Mbc5Rumble | Self::Mbc5RumbleRam | Self::Mbc5RumbleRamBattery => 512,
        }
    }

    pub fn supports_banking(self) -> bool {
        self.max_banks() > 0
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
