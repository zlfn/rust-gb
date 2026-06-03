//! `cargo gb`: build a Game Boy ROM from a Rust crate using the rust-z80
//! toolchain. Everything is taken from the toolchain sysroot, so no linker or
//! tool paths need configuring.

mod rom;
mod toolchain;
mod ui;

use cargo_metadata::MetadataCommand;
use object::{Object, ObjectSymbol};
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use toolchain::{TARGET, Toolchain};

fn main() {
    let cmd = std::env::args().nth(1).unwrap_or_else(|| "build".to_string());
    let result = match cmd.as_str() {
        // Invoked as `cargo gb <cmd>`, so the subcommand is argv[2] under cargo;
        // accept it directly too.
        "gb" => dispatch(std::env::args().nth(2).as_deref().unwrap_or("build")),
        other => dispatch(other),
    };
    if let Err(e) = result {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

fn dispatch(cmd: &str) -> Result<(), String> {
    match cmd {
        "build" => build(),
        "run" => run_rom(),
        "clean" => clean(),
        other => Err(format!("unknown command '{other}' (expected build, run, or clean)")),
    }
}

fn run_rom() -> Result<(), String> {
    build()?;
    let proj = resolve_project()?;
    let gb = proj.out_dir.join(format!("{}.gb", proj.name));
    let emu = std::env::var("EMULATOR").unwrap_or_else(|_| "sameboy".to_string());
    ui::status("Running", &emu);
    let status = Command::new(&emu)
        .arg(&gb)
        .status()
        .map_err(|e| format!("failed to launch {emu}: {e}"))?;
    if !status.success() {
        return Err(format!("{emu} exited with an error"));
    }
    Ok(())
}

struct Project {
    name: String,
    staticlib: PathBuf,
    release_dir: PathBuf,
    out_dir: PathBuf,
    header: Option<PathBuf>,
}

fn resolve_project() -> Result<Project, String> {
    let metadata = MetadataCommand::new()
        .exec()
        .map_err(|e| format!("cargo metadata failed: {e}"))?;
    let root = metadata
        .root_package()
        .ok_or("no root package (run inside a Game Boy crate)")?;
    let lib = root
        .targets
        .iter()
        .find(|t| t.kind.iter().any(|k| k == "staticlib"))
        .ok_or("the crate has no `staticlib` target")?;

    let release_dir = metadata
        .target_directory
        .join(TARGET)
        .join("release")
        .into_std_path_buf();
    let staticlib = release_dir.join(format!("lib{}.a", lib.name));

    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let out_dir = cwd.join("target");
    let header = cwd.join("header.toml");

    Ok(Project {
        name: root.name.clone(),
        staticlib,
        release_dir,
        out_dir,
        header: header.exists().then_some(header),
    })
}

fn build() -> Result<(), String> {
    let tc = Toolchain::discover()?;
    let proj = resolve_project()?;

    // 1. Compile the Rust staticlib.
    let pb = ui::spinner("Building", &format!("{} ({TARGET})", proj.name));
    let built = Command::new("cargo")
        .args(["build", "--release", "--target", TARGET, "--quiet"])
        .status();
    pb.finish_and_clear();
    if !built.map_err(|e| format!("failed to run cargo: {e}"))?.success() {
        return Err("cargo build failed".to_string());
    }

    std::fs::create_dir_all(&proj.out_dir).map_err(|e| e.to_string())?;

    // 2. Banking: patch the banked objects and repack them, or take the staticlib
    //    as is. gb.ld always `INCLUDE`s gb_banked.ld, so write an empty one when
    //    there is no banking.
    let banked = is_banked(&proj.staticlib)?;
    let (link_input, summary) = if banked {
        let obj_dir = proj.out_dir.join("bank_obj");
        let objs = extract_archive(&proj.staticlib, &obj_dir)?;
        let summary = gb_bank_pack::link(&proj.out_dir, &objs, proj.header.as_deref())
            .map_err(|e| format!("banking: {e}"))?;
        let libbanked = proj.out_dir.join("libbanked.a");
        repack(&tc.ar, &libbanked, &objs)?;
        (libbanked, Some(summary))
    } else {
        std::fs::write(proj.out_dir.join("gb_banked.ld"), "/* No banking */\n")
            .map_err(|e| e.to_string())?;
        (proj.staticlib.clone(), None)
    };

    // 3. Link.
    let pb = ui::spinner("Linking", &format!("{}.elf", proj.name));
    let elf = link(&tc, &proj, &link_input, banked);
    pb.finish_and_clear();
    let elf = elf?;

    // 4. ELF -> raw ROM.
    let gb = proj.out_dir.join(format!("{}.gb", proj.name));
    let image = rom::elf_to_rom(&elf)?;
    std::fs::write(&gb, &image).map_err(|e| e.to_string())?;

    // 5. Fix the cartridge header.
    let info = match &proj.header {
        Some(h) => Some(gb_header_fix::fix(&gb, h).map_err(|e| format!("header: {e}"))?),
        None => None,
    };

    let size_kb = info
        .as_ref()
        .map(|i| i.total_bytes / 1024)
        .unwrap_or_else(|| std::fs::metadata(&gb).map(|m| m.len() as usize / 1024).unwrap_or(0));
    if let Some(i) = &info {
        for w in &i.warnings {
            eprintln!("warning: {w}");
        }
    }
    ui::status("Finished", &format!("{}.gb   {size_kb} KB", proj.name));
    if let Some(s) = &summary {
        println!();
        ui::bank_bars(&s.banks, s.bank_size);
    }
    Ok(())
}

fn is_banked(staticlib: &Path) -> Result<bool, String> {
    let data = std::fs::read(staticlib).map_err(|e| format!("{}: {e}", staticlib.display()))?;
    let archive =
        object::read::archive::ArchiveFile::parse(&*data).map_err(|e| e.to_string())?;
    for member in archive.members() {
        let member = member.map_err(|e| e.to_string())?;
        let mdata = member.data(&*data).map_err(|e| e.to_string())?;
        if let Ok(obj) = object::File::parse(mdata) {
            if obj
                .symbols()
                .any(|s| s.name().is_ok_and(|n| n.contains("bank4BANK")))
            {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn extract_archive(staticlib: &Path, dir: &Path) -> Result<Vec<PathBuf>, String> {
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let data = std::fs::read(staticlib).map_err(|e| e.to_string())?;
    let archive =
        object::read::archive::ArchiveFile::parse(&*data).map_err(|e| e.to_string())?;

    let mut objs = Vec::new();
    for (i, member) in archive.members().enumerate() {
        let member = member.map_err(|e| e.to_string())?;
        let name = String::from_utf8_lossy(member.name());
        if !name.ends_with(".o") {
            continue;
        }
        let safe = name.replace(['/', '\\'], "_");
        let path = dir.join(format!("{i:04}_{safe}"));
        let mdata = member.data(&*data).map_err(|e| e.to_string())?;
        std::fs::write(&path, mdata).map_err(|e| e.to_string())?;
        objs.push(path);
    }
    Ok(objs)
}

fn repack(ar: &Path, archive: &Path, objs: &[PathBuf]) -> Result<(), String> {
    let _ = std::fs::remove_file(archive);
    run(Command::new(ar).arg("rcs").arg(archive).args(objs), "llvm-ar")
}

fn link(tc: &Toolchain, proj: &Project, input: &Path, banked: bool) -> Result<PathBuf, String> {
    let (gb_ld, supplementary) = collect_linker_scripts(&proj.release_dir)?;
    let elf = proj.out_dir.join(format!("{}.elf", proj.name));

    let mut cmd = Command::new(&tc.lld);
    cmd.arg("-T").arg(&gb_ld).arg("-L").arg(&proj.out_dir);
    for s in &supplementary {
        cmd.arg("-T").arg(s);
    }
    cmd.arg("--gc-sections");
    if banked {
        cmd.arg("--no-check-sections");
    }
    cmd.arg(input).arg("-o").arg(&elf);
    run(&mut cmd, "ld.lld")?;
    Ok(elf)
}

/// Collect the linker scripts dropped into each crate's build-script `OUT_DIR`:
/// `gb.ld` (the main script) plus any supplementary scripts, newest-first and one
/// per filename so stale build directories do not double-define sections.
fn collect_linker_scripts(release_dir: &Path) -> Result<(PathBuf, Vec<PathBuf>), String> {
    let build = release_dir.join("build");
    let mut found: Vec<(std::time::SystemTime, PathBuf)> = Vec::new();
    for entry in std::fs::read_dir(&build).map_err(|e| e.to_string())? {
        let out = entry.map_err(|e| e.to_string())?.path().join("out");
        let Ok(read) = std::fs::read_dir(&out) else {
            continue;
        };
        for f in read {
            let path = f.map_err(|e| e.to_string())?.path();
            if path.extension().is_some_and(|e| e == "ld") {
                let mtime = std::fs::metadata(&path)
                    .and_then(|m| m.modified())
                    .map_err(|e| e.to_string())?;
                found.push((mtime, path));
            }
        }
    }
    found.sort_by(|a, b| b.0.cmp(&a.0));

    let mut seen = std::collections::HashSet::new();
    let mut gb_ld = None;
    let mut supplementary = Vec::new();
    for (_, path) in found {
        let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        if !seen.insert(name.clone()) {
            continue;
        }
        if name == "gb.ld" {
            gb_ld = Some(path);
        } else {
            supplementary.push(path);
        }
    }
    let gb_ld = gb_ld.ok_or("gb.ld not found (is gb-rt a dependency of the crate?)")?;
    Ok((gb_ld, supplementary))
}

fn clean() -> Result<(), String> {
    let out = std::env::current_dir().map_err(|e| e.to_string())?.join("target");
    let _ = std::fs::remove_dir_all(&out);
    Ok(())
}

fn run(cmd: &mut Command, what: &str) -> Result<(), String> {
    let status = cmd.status().map_err(|e| format!("failed to run {what}: {e}"))?;
    if !status.success() {
        return Err(format!("{what} failed"));
    }
    Ok(())
}
