//! Bank layout: group banked symbols by module, bin-pack into 16 KiB banks,
//! and emit the placement + the `BANK`-marker patches that wire `Group::bank()`.
//!
//! The `#[bank]` macro already gives us everything we need in the object files:
//!
//! - Each banked body/static is its own section (`.text.<mangled>` /
//!   `.rodata.<mangled>` / `.data.<mangled>`), named after a symbol whose
//!   demangled path leaf is `__bank_fn_*`, `__bank_static_*`, or `__bank_*`
//!   (a method).
//! - Each banked module emits one `BANK` marker: a 1-byte static whose demangled
//!   name is `<CRATE::MOD::__BankGroup as ..::Group>::bank::BANK`. Its module
//!   path identifies the group; patching its byte sets the runtime bank number.

use object::read::elf::ElfFile32;
use object::{Object, ObjectSection, ObjectSymbol};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};


#[derive(Debug)]
pub struct BankLayout {
    pub bank_count: u8,
    /// module path -> assigned bank.
    pub group_banks: HashMap<String, u8>,
    pub bank_sizes: HashMap<u8, usize>,
    /// bank -> input section names to place there (deterministic order).
    pub placements: HashMap<u8, Vec<String>>,
    /// `BANK` marker bytes to overwrite before linking.
    pub patches: Vec<Patch>,
}

#[derive(Debug)]
pub struct Patch {
    pub obj: PathBuf,
    /// File offset of the marker's data byte (in its `.rodata.*` section).
    pub file_offset: u64,
    pub bank: u8,
}

/// Demangle a target symbol (drop the one leading `_` the target prepends),
/// omitting the disambiguator hashes.
fn demangle(name: &str) -> String {
    let n = name.strip_prefix('_').unwrap_or(name);
    format!("{:#}", rustc_demangle::demangle(n))
}

/// `<CRATE::MOD::__BankGroup as ..>::bank::BANK` -> `CRATE::MOD`.
fn bank_marker_module(demangled: &str) -> Option<String> {
    let pre = demangled.split("::__BankGroup").next()?;
    Some(pre.trim_start_matches('<').to_string())
}

/// Whether a demangled name has a path segment that starts with `__bank_`,
/// considering only segments *outside* any `<...>` span. This catches the banked
/// bodies/statics/methods (`__bank_fn_*` etc.) and anything lexically nested in one
/// (a closure or local fn: `..::__bank_fn_foo::{closure#0}`), while a `<T as Trait>`
/// qualifier head keeps its method name (`<T as Tr>::__bank_m`, the name sits after
/// the `>`) and a resident symbol merely *parameterised by* a banked type
/// (`resident::<..::__bank_fn_foo::{closure}>`, the `__bank_` is inside `<...>`) is
/// left resident.
fn nested_in_banked(demangled: &str) -> bool {
    let mut depth: i32 = 0;
    let mut seg = String::new();
    let mut hit = false;
    let flush = |seg: &mut String, hit: &mut bool| {
        if seg.starts_with("__bank_") {
            *hit = true;
        }
        seg.clear();
    };
    let mut chars = demangled.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '<' => depth += 1,
            '>' => depth -= 1,
            ':' if depth == 0 && chars.peek() == Some(&':') => {
                chars.next();
                flush(&mut seg, &mut hit);
            }
            _ if depth == 0 => seg.push(c),
            _ => {}
        }
    }
    flush(&mut seg, &mut hit);
    hit
}

/// The longest banked module that is a path prefix of `clean`.
fn group_of<'a>(clean: &str, modules: &'a [String]) -> Option<&'a String> {
    modules
        .iter()
        .filter(|m| clean.starts_with(&format!("{m}::")))
        .max_by_key(|m| m.len())
}

struct BankedSym {
    /// raw symbol name as it appears in section names (`.text.<raw>`).
    raw: String,
    /// demangled path with angle brackets stripped, for prefix grouping.
    clean: String,
}

struct PendingPatch {
    obj: PathBuf,
    file_offset: u64,
    module: String,
}

/// Scan the object files, group banked symbols by module, and bin-pack into banks.
pub fn compute_layout(
    obj_files: &[&Path],
    max_bank_size: usize,
    max_banks: u16,
) -> Result<BankLayout, Vec<String>> {
    let mut section_sizes: HashMap<String, usize> = HashMap::new();
    let mut banked: Vec<BankedSym> = Vec::new();
    let mut bank_modules: Vec<String> = Vec::new();
    let mut pending: Vec<PendingPatch> = Vec::new();
    // module -> pinned bank (from a `bank::module!(N)` marker whose initial byte is N).
    let mut pinned: HashMap<String, u8> = HashMap::new();

    for obj_path in obj_files {
        let Ok(data) = std::fs::read(obj_path) else { continue };
        let Ok(elf) = ElfFile32::<object::Endianness>::parse(&*data) else { continue };

        for sec in elf.sections() {
            if let Ok(name) = sec.name() {
                section_sizes.insert(name.to_string(), sec.size() as usize);
            }
        }

        for sym in elf.symbols() {
            let Ok(name) = sym.name() else { continue };
            if name.is_empty() {
                continue;
            }
            let dm = demangle(name);

            // BANK marker: `<MOD::__BankGroup as ..>::bank::BANK`.
            if dm.ends_with("::bank::BANK") && dm.contains("__BankGroup") {
                let Some(module) = bank_marker_module(&dm) else { continue };
                bank_modules.push(module.clone());
                if let Some(idx) = sym.section_index() {
                    if let Ok(sec) = elf.section_by_index(idx) {
                        if let Some((off, _)) = sec.file_range() {
                            let fo = off + sym.address();
                            // The marker's initial byte is the pin: non-zero means the
                            // module fixed itself to that bank via `bank::module!(N)`.
                            if let Some(&byte) = data.get(fo as usize) {
                                if byte != 0 {
                                    pinned.insert(module.clone(), byte);
                                }
                            }
                            pending.push(PendingPatch {
                                obj: obj_path.to_path_buf(),
                                file_offset: fo,
                                module,
                            });
                        }
                    }
                }
                continue;
            }

            // Banked body / static / method (`__bank_fn_*` / `__bank_static_*` /
            // `__bank_*`), *and anything nested inside one*: a closure or local fn
            // demangles to `..::__bank_fn_foo::{closure#0}` / `..::__bank_fn_foo::helper`,
            // whose leaf is not `__bank_*` but an ancestor segment is. Pinning those
            // into their parent's bank stops the optimizer from leaking an outlined
            // one into the resident region (bank 0). The resident macros name their
            // symbols `__fixed_*` / `__dyn_*` / `__far_*` and the group marker
            // `__BankGroup` (capital B), none of which match.
            let clean = dm.replace(['<', '>'], "");
            if nested_in_banked(&dm) {
                banked.push(BankedSym { raw: name.to_string(), clean });
            }
        }
    }

    bank_modules.sort();
    bank_modules.dedup();

    // Group placement sections by module.
    let mut group_sections: HashMap<String, Vec<String>> = HashMap::new();
    for b in &banked {
        let Some(group) = group_of(&b.clean, &bank_modules) else { continue };
        let entry = group_sections.entry(group.clone()).or_default();
        for kind in [".text.", ".rodata.", ".data."] {
            let sec = format!("{kind}{}", b.raw);
            if section_sizes.contains_key(&sec) {
                entry.push(sec);
            }
        }
    }

    // Size each group from the unique sizes of its placement sections.
    let mut errors = Vec::new();
    let mut group_size: Vec<(String, usize)> = Vec::new();
    for (group, secs) in &group_sections {
        let mut seen = HashSet::new();
        let mut total = 0usize;
        for s in secs {
            if seen.insert(s.clone()) {
                total += section_sizes.get(s).copied().unwrap_or(0);
            }
        }
        if total > max_bank_size {
            errors.push(format!(
                "group '{group}' is {total} bytes, exceeds {max_bank_size} byte bank limit"
            ));
        }
        group_size.push((group.clone(), total));
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    // First-fit decreasing bin-packing.
    group_size.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    let mut group_banks: HashMap<String, u8> = HashMap::new();
    let mut bank_sizes: HashMap<u8, usize> = HashMap::new();

    // Pinned groups take their requested bank; those banks are reserved so the
    // auto-assigned groups never share them.
    let reserved: HashSet<u8> = pinned.values().copied().collect();
    for (group, size) in &group_size {
        if let Some(&n) = pinned.get(group) {
            group_banks.insert(group.clone(), n);
            *bank_sizes.entry(n).or_insert(0) += size;
        }
    }

    // Bin-pack the rest into the non-reserved banks.
    let mut next_bank: u8 = 1;
    for (group, size) in &group_size {
        if pinned.contains_key(group) {
            continue;
        }
        let mut placed = false;
        for bank in 1..next_bank {
            if reserved.contains(&bank) {
                continue;
            }
            let cur = bank_sizes.get(&bank).copied().unwrap_or(0);
            if cur + size <= max_bank_size {
                group_banks.insert(group.clone(), bank);
                *bank_sizes.entry(bank).or_insert(0) += size;
                placed = true;
                break;
            }
        }
        if !placed {
            while reserved.contains(&next_bank) {
                next_bank += 1;
            }
            if next_bank as u16 > max_banks {
                return Err(vec![format!("too many banks needed (>{max_banks})")]);
            }
            group_banks.insert(group.clone(), next_bank);
            *bank_sizes.entry(next_bank).or_insert(0) += size;
            next_bank += 1;
        }
    }

    // A pinned bank may hold several groups (or collide with auto sizing); make sure
    // no bank overflows after everything is placed.
    for (bank, sz) in &bank_sizes {
        if *sz > max_bank_size {
            return Err(vec![format!(
                "bank {bank} is {sz} bytes, exceeds the {max_bank_size} byte limit"
            )]);
        }
    }

    // Placements per bank (sorted, deduped for deterministic output).
    let mut placements: HashMap<u8, Vec<String>> = HashMap::new();
    for (group, secs) in &group_sections {
        let bank = group_banks[group];
        placements.entry(bank).or_default().extend(secs.iter().cloned());
    }
    for v in placements.values_mut() {
        v.sort();
        v.dedup();
    }

    // Resolve each marker patch to its group's bank.
    let patches = pending
        .into_iter()
        .map(|p| Patch {
            obj: p.obj,
            file_offset: p.file_offset,
            bank: group_banks.get(&p.module).copied().unwrap_or(0),
        })
        .collect();

    Ok(BankLayout {
        // Highest bank number in use (pinned banks can exceed the auto counter).
        bank_count: bank_sizes.keys().copied().max().unwrap_or(0),
        group_banks,
        bank_sizes,
        placements,
        patches,
    })
}

/// Overwrite each `BANK` marker's data byte with its assigned bank number.
pub fn apply_patches(patches: &[Patch]) -> std::io::Result<()> {
    // Batch by object so each file is read and written once.
    let mut by_obj: HashMap<&Path, Vec<&Patch>> = HashMap::new();
    for p in patches {
        by_obj.entry(p.obj.as_path()).or_default().push(p);
    }
    for (obj, ps) in by_obj {
        let mut data = std::fs::read(obj)?;
        for p in ps {
            let off = p.file_offset as usize;
            if off < data.len() {
                data[off] = p.bank;
            }
        }
        std::fs::write(obj, data)?;
    }
    Ok(())
}

/// Generate the linker-script fragment placing each bank at its LMA.
pub fn generate_linker_script(layout: &BankLayout) -> String {
    if layout.bank_count == 0 {
        return String::new();
    }

    let mut out = String::new();
    out.push_str("/* Auto-generated by gb-bank-pack */\n\n");

    for bank in 1..=layout.bank_count {
        let lma = bank as u32 * 0x4000;
        out.push_str(&format!("    .bank{bank} 0x4000 : AT(0x{lma:X}) {{\n"));
        if let Some(secs) = layout.placements.get(&bank) {
            for s in secs {
                out.push_str(&format!("        *({s})\n"));
            }
        }
        out.push_str("    }\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::nested_in_banked;

    #[test]
    fn banked_bodies_and_nested_items_match() {
        // banked free fn / static / method
        assert!(nested_in_banked("bank_test::arithmetic::__bank_fn_add"));
        assert!(nested_in_banked("bank_test::trig::__bank_static_SINE_TABLE"));
        // nested closure / local fn inside a banked body
        assert!(nested_in_banked("bank_test::geometry::__bank_fn_probe_nest::{closure#0}"));
        assert!(nested_in_banked("bank_test::geometry::__bank_fn_probe_nest::nested_helper"));
        // generic banked fn instance: `__bank_` sits before the `<...>`
        assert!(nested_in_banked("bank_test::arithmetic::__bank_fn_clamp::<u8>"));
        // banked trait method: name sits after the `<T as Trait>` head
        assert!(nested_in_banked(
            "<bank_test::geometry::Point as bank_test::geometry::Metric>::__bank_distance"
        ));
    }

    #[test]
    fn resident_and_param_contamination_do_not_match() {
        // resident macro symbols
        assert!(!nested_in_banked("bank_test::__fixed_resident_sum"));
        assert!(!nested_in_banked("bank_test::__dyn_resident_inc"));
        assert!(!nested_in_banked("bank_test::__far_add"));
        // group marker (capital B)
        assert!(!nested_in_banked(
            "<bank_test::arithmetic::__BankGroup as gb_bank::Group>::bank::BANK"
        ));
        // resident generic merely *parameterised by* a banked closure type:
        // the `__bank_` is inside `<...>`, so it stays resident.
        assert!(!nested_in_banked(
            "bank_test::resident_gen::<bank_test::geometry::__bank_fn_f::{closure#0}>"
        ));
        // plain resident fn with a nested closure
        assert!(!nested_in_banked("bank_test::main::{closure#0}"));
    }
}
