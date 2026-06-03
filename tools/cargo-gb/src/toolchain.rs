//! Toolchain discovery: locate the rust-z80 sysroot and its bundled SM83 linker.
//!
//! Everything cargo-gb needs is taken from the active toolchain's sysroot, so a
//! correctly built rust-z80 needs no linker configuration.

use std::path::PathBuf;
use std::process::Command;

pub const TARGET: &str = "sm83-nintendo-none-elf";

pub struct Toolchain {
    /// The SM83-capable linker, `gcc-ld/ld.lld`.
    pub lld: PathBuf,
    /// `llvm-ar`, used to repack the patched banked objects.
    pub ar: PathBuf,
}

impl Toolchain {
    pub fn discover() -> Result<Toolchain, String> {
        // Gate 1: the active toolchain must support the SM83 target.
        let targets = rustc(&["--print", "target-list"])?;
        if !targets.lines().any(|t| t == TARGET) {
            return Err(format!(
                "the active Rust toolchain does not support {TARGET}.\n       \
                 cargo-gb requires the rust-z80 toolchain."
            ));
        }

        let sysroot = PathBuf::from(rustc(&["--print", "sysroot"])?.trim());
        let host = parse_host(&rustc(&["-vV"])?)?;
        let bin = sysroot.join("lib").join("rustlib").join(host).join("bin");
        let lld = bin.join("gcc-ld").join("ld.lld");
        let ar = bin.join("llvm-ar");

        // Gate 2: the linker must be bundled in the sysroot.
        if !lld.exists() {
            return Err(
                "ld.lld is not bundled in this rust-z80 toolchain.\n       \
                 Rebuild rust-z80 with `lld = true` in bootstrap.toml (in-tree LLVM)."
                    .to_string(),
            );
        }

        Ok(Toolchain { lld, ar })
    }
}

fn rustc(args: &[&str]) -> Result<String, String> {
    let out = Command::new("rustc")
        .args(args)
        .output()
        .map_err(|e| format!("failed to run rustc: {e}"))?;
    if !out.status.success() {
        return Err(format!("`rustc {}` failed", args.join(" ")));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn parse_host(version_verbose: &str) -> Result<String, String> {
    version_verbose
        .lines()
        .find_map(|l| l.strip_prefix("host: "))
        .map(|s| s.trim().to_string())
        .ok_or_else(|| "could not determine host triple from `rustc -vV`".to_string())
}
