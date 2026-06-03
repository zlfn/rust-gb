//! `gb-bank-pack` CLI: a thin wrapper over the `gb_bank_pack` library that runs a
//! banking pass and writes the bank summary to `bank_summary.txt`.

use std::fmt::Write as _;
use std::path::PathBuf;
use std::process;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args[1] == "--help" {
        eprintln!("gb-bank-pack — ROM banking tool for rust-gb");
        eprintln!();
        eprintln!("USAGE:");
        eprintln!("  gb-bank-pack link --obj <file>... --out-dir <dir> [--header <header.toml>]");
        eprintln!("    Pack banked objects into ROM banks and emit the linker fragment.");
        process::exit(0);
    }

    if args[1] != "link" {
        eprintln!("error: unknown command '{}', use 'link'", args[1]);
        process::exit(1);
    }

    let (out_dir, obj_files, header_toml) = parse_args(&args[2..]);

    let summary = match gb_bank_pack::link(&out_dir, &obj_files, header_toml.as_deref()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {e}");
            process::exit(1);
        }
    };

    let mut text = String::new();
    writeln!(text, "{} bank(s)", summary.banks.len()).unwrap();
    for b in &summary.banks {
        writeln!(
            text,
            "  ROM Bank {:02X}: {}/{} bytes [{}]",
            b.bank,
            b.used,
            summary.bank_size,
            b.modules.join(", ")
        )
        .unwrap();
    }
    let _ = std::fs::write(out_dir.join("bank_summary.txt"), &text);
}

fn parse_args(args: &[String]) -> (PathBuf, Vec<PathBuf>, Option<PathBuf>) {
    let mut out_dir = PathBuf::from(".");
    let mut obj_files = Vec::new();
    let mut header_toml = None;
    let mut i = 0;
    let mut in_obj = false;
    while i < args.len() {
        match args[i].as_str() {
            "--out-dir" => {
                in_obj = false;
                i += 1;
                out_dir = PathBuf::from(&args[i]);
            }
            "--header" => {
                in_obj = false;
                i += 1;
                header_toml = Some(PathBuf::from(&args[i]));
            }
            "--obj" => in_obj = true,
            _ if in_obj => obj_files.push(PathBuf::from(&args[i])),
            _ => {}
        }
        i += 1;
    }
    (out_dir, obj_files, header_toml)
}
