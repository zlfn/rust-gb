//! Post-build verification of `#[ram_fn]` functions.
//!
//! `gb-ram-fn`'s `install` copies a function's machine code into a fixed-size RAM
//! buffer and runs it there. Two properties make that sound, and neither can be
//! checked before linking, so they are checked here over the final ELF:
//!
//! - **Size**: the actual code length (`END - run`) must fit the declared `max`
//!   (the `MAX_MARKER` value). The caller's buffer is checked `>= max` at compile
//!   time, so this guarantees the copy fits.
//! - **Position independence**: the code must not reference its own bytes by an
//!   absolute address; copied elsewhere, such a reference would point back at the
//!   original. Relative `jr` branches move with the code and are fine.
//!
//! A `#[ram_fn]` emits three symbols per function (`run`, `END`, `MAX_MARKER`);
//! they are grouped by their demangled module path.

use object::{Object, ObjectSection, ObjectSymbol};
use std::collections::BTreeMap;
use std::path::Path;

/// SM83 instruction lengths in bytes, indexed by opcode. CB-prefixed instructions
/// are all 2 bytes, covered by the `0xCB` entry; invalid opcodes are treated as 1.
#[rustfmt::skip]
const OP_LEN: [u8; 256] = [
    1,3,1,1,1,1,2,1, 3,1,1,1,1,1,2,1,
    2,3,1,1,1,1,2,1, 2,1,1,1,1,1,2,1,
    2,3,1,1,1,1,2,1, 2,1,1,1,1,1,2,1,
    2,3,1,1,1,1,2,1, 2,1,1,1,1,1,2,1,
    1,1,1,1,1,1,1,1, 1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1, 1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1, 1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1, 1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1, 1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1, 1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1, 1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1, 1,1,1,1,1,1,1,1,
    1,1,3,3,3,1,2,1, 1,1,3,2,3,3,2,1,
    1,1,3,1,3,1,2,1, 1,1,3,1,3,1,2,1,
    2,1,1,1,1,1,2,1, 2,1,3,1,1,1,2,1,
    2,1,1,1,1,1,2,1, 2,1,3,1,1,1,2,1,
];

#[derive(Default)]
struct Markers {
    run: Option<u64>,
    end: Option<u64>,
    max: Option<u64>,
}

/// Verify every `#[ram_fn]` in the linked ELF, returning the first violation.
pub fn check(elf_path: &Path) -> Result<(), String> {
    let data = std::fs::read(elf_path).map_err(|e| format!("{}: {e}", elf_path.display()))?;
    let obj = object::File::parse(&*data).map_err(|e| e.to_string())?;

    let mut fns: BTreeMap<String, Markers> = BTreeMap::new();
    for sym in obj.symbols() {
        let Ok(name) = sym.name() else { continue };
        let dem = format!("{:#}", rustc_demangle::demangle(name));
        let (key, set): (&str, fn(&mut Markers, u64)) =
            if let Some(k) = dem.strip_suffix("::run") {
                (k, |m, a| m.run = Some(a))
            } else if let Some(k) = dem.strip_suffix("::END") {
                (k, |m, a| m.end = Some(a))
            } else if let Some(k) = dem.strip_suffix("::MAX_MARKER") {
                (k, |m, a| m.max = Some(a))
            } else {
                continue;
            };
        set(fns.entry(key.to_string()).or_default(), sym.address());
    }

    for (name, m) in &fns {
        // A real ram_fn has all three markers; a stray `::run`/`::END` elsewhere
        // does not, and is skipped.
        let (Some(run), Some(end), Some(max_addr)) = (m.run, m.end, m.max) else {
            continue;
        };

        let len = end
            .checked_sub(run)
            .ok_or_else(|| format!("ram_fn `{name}`: END precedes run"))?;
        if len == 0 {
            return Err(format!("ram_fn `{name}`: empty (body eliminated?)"));
        }

        let max = read_at(&obj, max_addr, 2)
            .ok_or_else(|| format!("ram_fn `{name}`: unreadable max marker"))?;
        let max = u16::from_le_bytes([max[0], max[1]]) as u64;
        if len > max {
            return Err(format!(
                "ram_fn `{name}`: compiled to {len} bytes, over the declared max of {max}"
            ));
        }

        let code = read_at(&obj, run, len as usize)
            .ok_or_else(|| format!("ram_fn `{name}`: unreadable code"))?;
        if let Some(target) = self_ref(code, run, end) {
            return Err(format!(
                "ram_fn `{name}`: not position independent \
                 (absolute reference to 0x{target:04x} within its own code)"
            ));
        }
    }

    Ok(())
}

/// `len` bytes of loaded data at virtual address `vma`.
fn read_at<'d>(obj: &object::File<'d>, vma: u64, len: usize) -> Option<&'d [u8]> {
    for sec in obj.sections() {
        let addr = sec.address();
        if addr <= vma && vma < addr + sec.size() {
            let data = sec.data().ok()?;
            let off = (vma - addr) as usize;
            return data.get(off..off.checked_add(len)?);
        }
    }
    None
}

/// The first absolute operand pointing inside `[run, end)`, if any. Walks the
/// code instruction by instruction so operand bytes are never read as opcodes.
fn self_ref(code: &[u8], run: u64, end: u64) -> Option<u64> {
    let mut i = 0;
    while i < code.len() {
        let op = code[i];
        // Opcodes carrying a 2-byte absolute address: jp/call (cc) nn, and the
        // absolute loads `ld (nn),sp` / `ld (nn),a` / `ld a,(nn)`.
        let absolute = matches!(
            op,
            0xC2 | 0xC3 | 0xC4 | 0xCA | 0xCC | 0xCD | 0xD2 | 0xD4 | 0xDA | 0xDC
                | 0x08 | 0xEA | 0xFA
        );
        if absolute {
            if let Some(b) = code.get(i + 1..i + 3) {
                let target = u16::from_le_bytes([b[0], b[1]]) as u64;
                if (run..end).contains(&target) {
                    return Some(target);
                }
            }
        }
        i += OP_LEN[op as usize].max(1) as usize;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::self_ref;

    #[test]
    fn flags_absolute_self_jump() {
        // jp 0x1003 (C3 03 10), then padding.
        let code = [0xC3, 0x03, 0x10, 0x00, 0x00, 0x00];
        assert_eq!(self_ref(&code, 0x1000, 0x1006), Some(0x1003));
    }

    #[test]
    fn ignores_relative_and_external() {
        // jr -2 (18 FE); call 0x0048 (CD 48 00); ret (C9). Nothing inside the body.
        let code = [0x18, 0xFE, 0xCD, 0x48, 0x00, 0xC9];
        assert_eq!(self_ref(&code, 0x1000, 0x1006), None);
    }

    #[test]
    fn decodes_lengths_so_operands_are_not_opcodes() {
        // ld b, 0xC3 (06 C3); dec b (05); stop (10 00). A naive byte scan would
        // read the 0xC3 immediate as `jp 0x1005` and wrongly flag it.
        let code = [0x06, 0xC3, 0x05, 0x10, 0x00];
        assert_eq!(self_ref(&code, 0x1000, 0x1010), None);
    }
}
