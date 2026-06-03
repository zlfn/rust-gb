//! `gb-header-fix` CLI: a thin wrapper over the `gb_header_fix` library that
//! patches a ROM's cartridge header and prints its fixed-region usage.

use std::path::PathBuf;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: gb-header-fix <rom-file> <header.toml>");
        process::exit(1);
    }

    let rom_path = PathBuf::from(&args[1]);
    let toml_path = PathBuf::from(&args[2]);

    let info = match gb_header_fix::fix(&rom_path, &toml_path) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    };

    for w in &info.warnings {
        eprintln!("warning: {w}");
    }

    let rom_name = rom_path.file_name().unwrap_or_default().to_string_lossy();
    println!("{} ({}KB)", rom_name, info.total_bytes / 1024);
    println!("  {}: {}/{}", info.label, info.used, info.limit);
}
