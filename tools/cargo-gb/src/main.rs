//! `cargo gb`: build a Game Boy ROM from a Rust crate using the rust-z80
//! toolchain.

mod ramfn;
mod rom;
mod toolchain;
mod ui;

use cargo_metadata::{Message, MetadataCommand};
use object::{Object, ObjectSection};
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use toolchain::{Toolchain, TARGET};

const CHECK_CFG: &str = concat!(
    " --check-cfg=cfg(gb_wide_bank,values(\"mbc1\",\"mbc5\"))",
    " --check-cfg=cfg(gb_custom_mapper)",
    " --check-cfg=cfg(gb_flat_rom)",
    " --check-cfg=cfg(gb_pak_mbc,values(\"mbc1\",\"mbc2\",\"mbc3\",\"mbc5\",\"mbc7\"))",
    " --check-cfg=cfg(gb_pak_sram)",
    " --check-cfg=cfg(gb_pak_sram_banks,values(\"4\",\"8\",\"16\"))",
    " --check-cfg=cfg(gb_pak_rtc)",
    " --check-cfg=cfg(gb_pak_rumble)",
    " --check-cfg=cfg(gb_pak_tilt)",
);

fn main() {
    let cmd = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "build".to_string());
    let result = match cmd.as_str() {
        // Invoked as `cargo gb <cmd>`, so the subcommand is argv[2] under cargo.
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
        "build" => build().map(|_| ()),
        "run" => run_rom(),
        "clean" => clean(),
        other => Err(format!(
            "unknown command '{other}' (expected build, run, or clean)"
        )),
    }
}

fn run_rom() -> Result<(), String> {
    let rom = build()?;
    let emu = std::env::var("EMULATOR").unwrap_or_else(|_| "sameboy".to_string());
    ui::status("Running", &emu);
    let status = Command::new(&emu)
        .arg(&rom)
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
        out_dir,
        header: header.exists().then_some(header),
    })
}

fn build() -> Result<PathBuf, String> {
    let tc = Toolchain::discover()?;
    let proj = resolve_project()?;

    // 1. Compile the Rust staticlib.
    let mut cargo = Command::new("cargo");
    cargo.args(["build", "--release", "--target", TARGET]);
    let cart = match proj.header.as_deref() {
        Some(h) => gb_header_fix::read_cartridge(h).map_err(|e| e.to_string())?,
        None => None,
    };
    if let Some(cart) = &cart {
        // Setting RUSTFLAGS supersedes `build.rustflags` from a config file, so a
        // project that sets both loses the latter here.
        let mut flags = std::env::var("RUSTFLAGS").unwrap_or_default();
        flags.push_str(CHECK_CFG);
        if cart.max_bank == 0 {
            flags.push_str(" --cfg gb_flat_rom");
        }
        if let Some(kind) = cart.wide {
            flags.push_str(&format!(" --cfg gb_wide_bank=\"{kind}\""));
        }
        if let Some(mbc) = cart.mbc {
            flags.push_str(&format!(" --cfg gb_pak_mbc=\"{mbc}\""));
        }
        if cart.sram_banks > 0 {
            flags.push_str(" --cfg gb_pak_sram");
            if cart.sram_banks > 1 {
                flags.push_str(&format!(" --cfg gb_pak_sram_banks=\"{}\"", cart.sram_banks));
            }
        }
        if cart.rtc {
            flags.push_str(" --cfg gb_pak_rtc");
        }
        if cart.rumble {
            flags.push_str(" --cfg gb_pak_rumble");
        }
        if cart.tilt {
            flags.push_str(" --cfg gb_pak_tilt");
        }
        cargo.env("RUSTFLAGS", flags);
    }

    cargo.arg("--message-format=json-render-diagnostics");
    cargo.stdout(Stdio::piped());
    let mut child = cargo
        .spawn()
        .map_err(|e| format!("failed to run cargo: {e}"))?;
    let stdout = child.stdout.take().ok_or("cargo stdout is not piped")?;
    let mut out_dirs = Vec::new();
    for msg in Message::parse_stream(std::io::BufReader::new(stdout)) {
        if let Message::BuildScriptExecuted(script) = msg.map_err(|e| e.to_string())? {
            out_dirs.push(script.out_dir.into_std_path_buf());
        }
    }
    let built = child
        .wait()
        .map_err(|e| format!("failed to run cargo: {e}"))?;
    if !built.success() {
        return Err("cargo build failed".to_string());
    }

    std::fs::create_dir_all(&proj.out_dir).map_err(|e| e.to_string())?;

    // 2. Banking: patch the banked objects and repack them.
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
    let elf = link(&tc, &proj, &link_input, &out_dirs);
    pb.finish_and_clear();
    let elf = elf?;

    // 4. Verify #[ram_fn] functions (size <= max, position independent).
    ramfn::check(&elf)?;

    // 5. ELF -> raw ROM.
    let gb = proj.out_dir.join(format!("{}.gb", proj.name));
    let image = rom::elf_to_rom(&elf)?;
    std::fs::write(&gb, &image).map_err(|e| e.to_string())?;

    // 6. Fix the cartridge header.
    let info = match &proj.header {
        Some(h) => Some(gb_header_fix::fix(&gb, h).map_err(|e| format!("header: {e}"))?),
        None => None,
    };

    // 7. Set the extension of the output file (.gb or .gbc).
    let rom_path = if info
        .as_ref()
        .is_some_and(|i| i.cgb != gb_header_fix::CgbFlag::None)
    {
        let gbc = proj.out_dir.join(format!("{}.gbc", proj.name));
        std::fs::rename(&gb, &gbc).map_err(|e| e.to_string())?;
        gbc
    } else {
        gb
    };
    let rom_name = rom_path.file_name().unwrap_or_default().to_string_lossy();

    // 8. Print the packing summary.
    let size_kb = info
        .as_ref()
        .map(|i| i.total_bytes / 1024)
        .unwrap_or_else(|| {
            std::fs::metadata(&rom_path)
                .map(|m| m.len() as usize / 1024)
                .unwrap_or(0)
        });
    if let Some(i) = &info {
        for w in &i.warnings {
            eprintln!("warning: {w}");
        }
    }
    match &summary {
        Some(s) if !s.banks.is_empty() => {
            ui::status("Packed", &format!("{} bank(s)", s.banks.len()));
            ui::bank_bars(info.as_ref().map(|i| i.used), &s.banks, s.bank_size);
        }
        _ => {
            ui::status("Packed", "1 bank");
            if let Some(i) = &info {
                ui::rom_bar(i.used, i.limit);
            }
        }
    }
    ui::status("Ready", &format!("{rom_name}   {size_kb} KB"));
    Ok(rom_path)
}

fn is_banked(staticlib: &Path) -> Result<bool, String> {
    let data = std::fs::read(staticlib).map_err(|e| format!("{}: {e}", staticlib.display()))?;
    let archive = object::read::archive::ArchiveFile::parse(&*data).map_err(|e| e.to_string())?;
    for member in archive.members() {
        let member = member.map_err(|e| e.to_string())?;
        let mdata = member.data(&*data).map_err(|e| e.to_string())?;
        if let Ok(obj) = object::File::parse(mdata) {
            if obj.sections().any(|s| {
                s.name()
                    .is_ok_and(|n| n.starts_with(gb_bank_pack::MARKER_SECTION))
            }) {
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
    let archive = object::read::archive::ArchiveFile::parse(&*data).map_err(|e| e.to_string())?;

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
    run(
        Command::new(ar).arg("rcs").arg(archive).args(objs),
        "llvm-ar",
    )
}

fn link(
    tc: &Toolchain,
    proj: &Project,
    input: &Path,
    out_dirs: &[PathBuf],
) -> Result<PathBuf, String> {
    let (gb_ld, supplementary) = collect_linker_scripts(out_dirs)?;
    let elf = proj.out_dir.join(format!("{}.elf", proj.name));

    let mut cmd = Command::new(&tc.lld);
    cmd.arg("-T").arg(&gb_ld).arg("-L").arg(&proj.out_dir);
    for s in &supplementary {
        cmd.arg("-T").arg(s);
    }
    cmd.arg("--gc-sections");
    cmd.arg(input);
    // The target spec's `late-link-args`, which reach the linker after the
    // inputs so the archive answers what they left undefined. Its `crt-objects`
    // stay out: `rrt0.s` is this project's entry point.
    if let Some(sc) = &tc.self_contained {
        cmd.arg("-L").arg(sc).arg("-lsm83_rt");
    }
    cmd.arg("-o").arg(&elf);
    run(&mut cmd, "ld.lld")?;
    Ok(elf)
}

/// Collect the linker scripts from the build-script `OUT_DIR`s of the crates in
/// this build.
fn collect_linker_scripts(out_dirs: &[PathBuf]) -> Result<(PathBuf, Vec<PathBuf>), String> {
    let mut gb_ld = None;
    let mut supplementary = Vec::new();
    for out in out_dirs {
        let Ok(read) = std::fs::read_dir(out) else {
            continue;
        };
        for f in read {
            let path = f.map_err(|e| e.to_string())?.path();
            if path.extension().is_none_or(|e| e != "ld") {
                continue;
            }
            if path.file_name().is_some_and(|n| n == "gb.ld") {
                if gb_ld.replace(path).is_some() {
                    return Err("gb.ld found in more than one crate".to_string());
                }
            } else {
                supplementary.push(path);
            }
        }
    }
    let gb_ld = gb_ld.ok_or("gb.ld not found (is gb-rt a dependency of the crate?)")?;
    // gb_defaults.ld should be last because it `PROVIDE`s the default interrupt handlers.
    supplementary.sort_by_key(|p| {
        let name = p
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        (name == "gb_defaults.ld", name)
    });
    Ok((gb_ld, supplementary))
}

fn clean() -> Result<(), String> {
    let out = std::env::current_dir()
        .map_err(|e| e.to_string())?
        .join("target");
    let _ = std::fs::remove_dir_all(&out);
    Ok(())
}

fn run(cmd: &mut Command, what: &str) -> Result<(), String> {
    let status = cmd
        .status()
        .map_err(|e| format!("failed to run {what}: {e}"))?;
    if !status.success() {
        return Err(format!("{what} failed"));
    }
    Ok(())
}
