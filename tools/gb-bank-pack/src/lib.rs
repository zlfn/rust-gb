//! Link-time bank allocator for the `gb-bank` toolchain.
//!
//! It reads the linked objects, demangles their symbols, groups each banked
//! function/static under its module's `BANK` marker, first-fit bin-packs the
//! groups into 16 KiB banks, then patches every module's `BANK` marker to its
//! assigned bank number and emits a linker fragment placing each
//! `.text`/`.rodata`/`.data.<sym>` section into its `.bankN` region. The resident
//! region stays in bank 0; banked groups go to banks 1 and up.

use std::path::{Path, PathBuf};

mod packer;

/// One banked ROM bank's allocation.
pub struct BankInfo {
    /// Bank number (1 and up; bank 0 is the resident region).
    pub bank: u16,
    /// Bytes occupied by the groups placed in this bank.
    pub used: usize,
    /// Module paths whose code and data live in this bank, sorted.
    pub modules: Vec<String>,
}

/// The per-bank result of a banking pass.
pub struct LinkSummary {
    /// Size of one ROM bank in bytes.
    pub bank_size: usize,
    /// Banked banks (1 and up), in ascending order.
    pub banks: Vec<BankInfo>,
}

/// A failure during the banking pass.
#[derive(Debug)]
pub enum LinkError {
    /// One or more groups could not be placed (oversized, or too many banks).
    Layout(Vec<String>),
    /// An object file could not be read or patched.
    Io(std::io::Error),
}

impl std::fmt::Display for LinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkError::Layout(errors) => write!(f, "{}", errors.join("; ")),
            LinkError::Io(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for LinkError {}

impl From<std::io::Error> for LinkError {
    fn from(e: std::io::Error) -> Self {
        LinkError::Io(e)
    }
}

/// Pack the banked symbols in `obj_files` into ROM banks.
///
/// Patches the `BANK` markers in the objects in place, writes the `gb_banked.ld`
/// linker fragment into `out_dir`, and returns the per-bank allocation.
/// `header_toml`, when given, bounds the bank count by the cartridge's MBC type.
/// With no MBC nothing is banked: the 32 KiB is fixed end to end, so splitting it
/// at `0x4000` would only wall off the two halves.
pub fn link(
    out_dir: &Path,
    obj_files: &[PathBuf],
    header_toml: Option<&Path>,
) -> Result<LinkSummary, LinkError> {
    let bank_size = 0x4000;

    // The highest bank `switch_bank` can select, or 0 with no MBC, in which case the
    // packer leaves every group flat.
    let limits = header_toml.and_then(|p| gb_header_fix::read_cartridge(p).ok().flatten());
    let (max_bank, excluded) = match limits {
        Some(l) => (l.max_bank, l.excluded),
        None => (255, Vec::new()),
    };
    let excluded = &excluded[..];

    let obj_paths: Vec<&Path> = obj_files.iter().map(|p| p.as_path()).collect();
    let layout = packer::compute_layout(&obj_paths, bank_size, max_bank, excluded)
        .map_err(LinkError::Layout)?;

    packer::apply_patches(&layout.patches)?;

    let ld_script = packer::generate_linker_script(&layout);
    std::fs::write(out_dir.join("gb_banked.ld"), ld_script)?;

    let mut banks = Vec::new();
    for bank in 1..=layout.bank_count {
        let used = layout.bank_sizes.get(&bank).copied().unwrap_or(0);
        let mut modules: Vec<String> = layout
            .group_banks
            .iter()
            .filter(|(_, b)| **b == bank)
            .map(|(name, _)| name.clone())
            .collect();
        modules.sort();
        banks.push(BankInfo {
            bank,
            used,
            modules,
        });
    }

    Ok(LinkSummary { bank_size, banks })
}

