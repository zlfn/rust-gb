//! Flatten a linked SM83 ELF into a raw ROM image (the `objcopy -O binary` step).

use object::Endianness;
use object::elf::{FileHeader32, PT_LOAD, SHT_NOBITS};
use object::read::elf::{FileHeader, ProgramHeader, SectionHeader};
use std::path::Path;

/// Place each loadable section's bytes at its physical address (LMA), filling the
/// gaps between sections with `0xFF`. A section's LMA is taken from the `PT_LOAD`
/// segment that holds its file range, so banked sections that share a virtual
/// address still land in their own bank. NOLOAD / `.bss` sections carry no file
/// bytes and drop out.
pub fn elf_to_rom(elf_path: &Path) -> Result<Vec<u8>, String> {
    let data = std::fs::read(elf_path).map_err(|e| format!("{}: {e}", elf_path.display()))?;
    let header = FileHeader32::<Endianness>::parse(&*data)
        .map_err(|e| format!("parsing {}: {e}", elf_path.display()))?;
    let endian = header.endian().map_err(|e| e.to_string())?;
    let segments = header
        .program_headers(endian, &*data)
        .map_err(|e| e.to_string())?;
    let sections = header.sections(endian, &*data).map_err(|e| e.to_string())?;

    let loads: Vec<_> = segments
        .iter()
        .filter(|p| p.p_type(endian) == PT_LOAD)
        .collect();

    let mut placed: Vec<(u32, &[u8])> = Vec::new();
    for sec in sections.iter() {
        if sec.sh_type(endian) == SHT_NOBITS {
            continue;
        }
        let size = sec.sh_size(endian);
        if size == 0 {
            continue;
        }
        let offset = sec.sh_offset(endian);

        let Some(seg) = loads.iter().find(|p| {
            let start = p.p_offset(endian);
            offset >= start && offset + size <= start + p.p_filesz(endian)
        }) else {
            continue;
        };

        let lma = seg.p_paddr(endian) + (offset - seg.p_offset(endian));
        let bytes = data
            .get(offset as usize..(offset + size) as usize)
            .ok_or("section data out of bounds")?;
        placed.push((lma, bytes));
    }

    if placed.is_empty() {
        return Err("no loadable sections in ELF".to_string());
    }

    let base = placed.iter().map(|(lma, _)| *lma).min().unwrap();
    let end = placed
        .iter()
        .map(|(lma, b)| lma + b.len() as u32)
        .max()
        .unwrap();

    let mut rom = vec![0xFFu8; (end - base) as usize];
    for (lma, bytes) in placed {
        let at = (lma - base) as usize;
        rom[at..at + bytes.len()].copy_from_slice(bytes);
    }
    Ok(rom)
}
