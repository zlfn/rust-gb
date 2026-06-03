use gb_header_fix::{CartridgeType, deserialize_cartridge_type};
use serde::Deserialize;
use std::path::PathBuf;

const NINTENDO_LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83,
    0x00, 0x0C, 0x00, 0x0D, 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E,
    0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63,
    0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
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

#[derive(Deserialize, Default)]
enum CgbFlag {
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

fn rom_size_code(len: usize) -> u8 {
    match len {
        0..=32768 => 0x00,       // 32KB  - 2 banks
        0..=65536 => 0x01,       // 64KB  - 4 banks
        0..=131072 => 0x02,      // 128KB - 8 banks
        0..=262144 => 0x03,      // 256KB - 16 banks
        0..=524288 => 0x04,      // 512KB - 32 banks
        0..=1048576 => 0x05,     // 1MB   - 64 banks
        0..=2097152 => 0x06,     // 2MB   - 128 banks
        0..=4194304 => 0x07,     // 4MB   - 256 banks
        _ => 0x08,               // 8MB   - 512 banks
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: gb-header-fix <rom-file> <header.toml>");
        std::process::exit(1);
    }

    let rom_path = PathBuf::from(&args[1]);
    let toml_path = PathBuf::from(&args[2]);

    let toml_str = std::fs::read_to_string(&toml_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", toml_path.display()));
    let header: Header = toml::from_str(&toml_str)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {e}", toml_path.display()));

    let mut rom = std::fs::read(&rom_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", rom_path.display()));

    if rom.len() < 0x150 {
        rom.resize(0x150, 0xFF);
    }

    pad_to_power_of_two(&mut rom);

    // 0x0104-0x0133: Nintendo logo
    rom[0x0104..0x0134].copy_from_slice(&NINTENDO_LOGO);

    // 0x0134-0x0143: Title (up to 15 bytes if CGB flag used, 16 for DMG-only)
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

    // 0x0144-0x0145: New licensee code (2 ASCII chars, used when old_licensee == 0x33)
    if let Some(ref code) = header.new_licensee_code {
        let bytes = code.as_bytes();
        rom[0x0144] = bytes.first().copied().unwrap_or(0x00);
        rom[0x0145] = bytes.get(1).copied().unwrap_or(0x00);
    }

    // 0x0146: SGB flag
    rom[0x0146] = if header.sgb_flag { 0x03 } else { 0x00 };

    // 0x0147: Cartridge type
    rom[0x0147] = header.cartridge_type.to_byte();

    // 0x0148: ROM size (auto-calculated from padded size)
    rom[0x0148] = rom_size_code(rom.len());

    // 0x0149: RAM size
    rom[0x0149] = header.ram_size;

    // 0x014A: Destination code
    rom[0x014A] = match header.destination {
        Destination::Japan => 0x00,
        Destination::Worldwide => 0x01,
    };

    // 0x014B: Old licensee code (0x33 means use new licensee at 0x0144-0x0145)
    // SGB also requires 0x33 for detection
    // 0x014B: Old licensee code (0x33 means use new licensee at 0x0144-0x0145)
    // SGB also requires 0x33 for detection by SNES hardware
    let override_licensee = header.new_licensee_code.is_some() || header.sgb_flag;
    if override_licensee && header.old_licensee_code != 0x00 && header.old_licensee_code != 0x33 {
        eprintln!(
            "warning: old_licensee_code 0x{:02X} overridden to 0x33 (required by {})",
            header.old_licensee_code,
            if header.sgb_flag && header.new_licensee_code.is_some() {
                "sgb_flag and new_licensee_code"
            } else if header.sgb_flag {
                "sgb_flag"
            } else {
                "new_licensee_code"
            }
        );
    }
    rom[0x014B] = if override_licensee {
        0x33
    } else {
        header.old_licensee_code
    };

    // 0x014C: Mask ROM version
    rom[0x014C] = header.version;

    // 0x014D: Header checksum — x = 0; for i in 0x0134..=0x014C { x = x - rom[i] - 1 }
    let mut hck: u8 = 0;
    for &b in &rom[0x0134..=0x014C] {
        hck = hck.wrapping_sub(b).wrapping_sub(1);
    }
    rom[0x014D] = hck;

    // 0x014E-0x014F: Global checksum — sum of all ROM bytes (excluding these two bytes)
    rom[0x014E] = 0x00;
    rom[0x014F] = 0x00;
    let mut gck: u16 = 0;
    for &b in rom.iter() {
        gck = gck.wrapping_add(b as u16);
    }
    rom[0x014E] = (gck >> 8) as u8;
    rom[0x014F] = gck as u8;

    std::fs::write(&rom_path, &rom)
        .unwrap_or_else(|e| panic!("Failed to write {}: {e}", rom_path.display()));

    // Fixed-region usage (last non-0xFF byte). With an MBC, only ROM Bank 00
    // (0x0000-0x3FFF = 16 KiB) is permanently mapped: 0x4000-0x7FFF is the switchable
    // window. Without banking (ROM ONLY) the whole 0x0000-0x7FFF (32 KiB) is fixed and
    // addressable, so report the entire ROM.
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

    let rom_name = rom_path.file_name().unwrap_or_default().to_string_lossy();
    println!("{} ({}KB)", rom_name, rom.len() / 1024);
    println!("  {}: {}/{}", label, used, limit);
}
