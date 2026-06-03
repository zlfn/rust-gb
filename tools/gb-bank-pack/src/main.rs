//! `gb-bank-pack`: the link-time bank allocator for the `gb-bank` toolchain.
//!
//! It reads the linked objects, demangles their symbols, groups each banked
//! function/static under its module's `BANK` marker, first-fit bin-packs the
//! groups into 16 KiB banks, then (1) patches every module's `BANK` byte to its
//! assigned bank number and (2) emits a linker fragment placing each
//! `.text`/`.rodata`/`.data.<sym>` section into the right `.bankN` region. The
//! resident region stays in bank 0; banked groups go to banks 1 and up.

mod packer;

use std::path::PathBuf;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args[1] == "--help" {
        eprintln!("gb-bank-pack — ROM banking tool for rust-gb");
        eprintln!();
        eprintln!("USAGE:");
        eprintln!("  gb-bank-pack link --obj <file>... --out-dir <dir> [--header <header.toml>]");
        eprintln!("    Processes .o files, generates bank support asm and linker script.");
        process::exit(0);
    }

    if args[1] == "link" {
        cmd_link(&args[2..]);
    } else {
        eprintln!("error: unknown command '{}', use 'link'", args[1]);
        process::exit(1);
    }
}

fn cmd_link(args: &[String]) {
    let (out_dir, obj_files, header_toml) = parse_args(args);

    // MBC type from header.toml (optional)
    let max_banks: u16 = if let Some(ref path) = header_toml {
        match parse_header_for_mbc(path) {
            Some((max, supports)) => {
                if !supports {
                    eprintln!("error: cartridge type does not support banking");
                    process::exit(1);
                }
                max
            }
            None => 512,
        }
    } else {
        512
    };

    // 1. Compute bank layout
    let obj_paths: Vec<&std::path::Path> = obj_files.iter().map(|p| p.as_path()).collect();
    let layout = match packer::compute_layout(&obj_paths, 0x4000, max_banks) {
        Ok(l) => l,
        Err(errors) => {
            for e in &errors { eprintln!("error: {}", e); }
            process::exit(1);
        }
    };

    // 2. Patch each module's BANK marker byte to its assigned bank number, so
    //    `Group::bank()` returns the right value at runtime.
    if let Err(e) = packer::apply_patches(&layout.patches) {
        eprintln!("error: patching BANK markers: {}", e);
        process::exit(1);
    }

    // 3. Generate linker script. No trampolines or bank-shadow object is emitted:
    //    the __current_bank shadow is defined by the gb-rt runtime (shared with
    //    GBDK's C runtime), and bank switching is emitted inline by the gb-bank
    //    macros.
    let ld_script = packer::generate_linker_script(&layout);
    let ld_path = out_dir.join("gb_banked.ld");
    std::fs::write(&ld_path, &ld_script)
        .unwrap_or_else(|e| { eprintln!("error: {}", e); process::exit(1); });

    // 4. Summary
    let mut summary = String::new();
    use std::fmt::Write;
    writeln!(summary, "{} bank(s)", layout.bank_count).unwrap();
    for bank in 1..=layout.bank_count {
        let size = layout.bank_sizes.get(&bank).copied().unwrap_or(0);
        let groups: Vec<_> = layout.group_banks.iter()
            .filter(|(_, b)| **b == bank)
            .map(|(name, _)| name.as_str())
            .collect();
        writeln!(summary, "  ROM Bank {:02X}: {}/16384 bytes [{}]", bank, size, groups.join(", ")).unwrap();
    }
    let _ = std::fs::write(out_dir.join("bank_summary.txt"), &summary);
}

fn parse_args(args: &[String]) -> (PathBuf, Vec<PathBuf>, Option<PathBuf>) {
    let mut out_dir = PathBuf::from(".");
    let mut obj_files = Vec::new();
    let mut header_toml = None;
    let mut i = 0;
    let mut in_obj = false;
    while i < args.len() {
        match args[i].as_str() {
            "--out-dir" => { in_obj = false; i += 1; out_dir = PathBuf::from(&args[i]); }
            "--header" => { in_obj = false; i += 1; header_toml = Some(PathBuf::from(&args[i])); }
            "--obj" => { in_obj = true; }
            _ if in_obj => { obj_files.push(PathBuf::from(&args[i])); }
            _ => {}
        }
        i += 1;
    }
    (out_dir, obj_files, header_toml)
}

fn parse_header_for_mbc(path: &std::path::Path) -> Option<(u16, bool)> {
    #[derive(serde::Deserialize)]
    struct HeaderConfig {
        #[serde(default, deserialize_with = "gb_header_fix::deserialize_cartridge_type")]
        cartridge_type: gb_header_fix::CartridgeType,
    }
    let content = std::fs::read_to_string(path).ok()?;
    let header: HeaderConfig = toml::from_str(&content).ok()?;
    Some((header.cartridge_type.max_banks(), header.cartridge_type.supports_banking()))
}

