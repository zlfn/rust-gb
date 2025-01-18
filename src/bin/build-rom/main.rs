use cargo::Cargo;
use clap::{arg, command, Parser};
use core::str;
use lcc::Lcc;
use llvm::{LlvmCbe, LlvmLink};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    process::{self, Command, ExitStatus},
    str::FromStr,
};

use colored::Colorize;
use include_dir::{include_dir, Dir};
use sdcc::Sdcc;
use treesitter::Treesitter;

mod cargo;
mod lcc;
mod llvm;
mod sdcc;
mod treesitter;

// External files
static EXT_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/ext");

#[derive(Parser)]
#[command(bin_name = "cargo", version, author, disable_help_subcommand = true)]
pub enum Subcommand {
    /// Build a Cargo package to GameBoy ROM
    #[command(name = "build-rom", version, author, disable_version_flag = true)]
    BuildRom(BuildRom),
}

#[derive(Parser, Debug)]
pub struct BuildRom {
    /// Select at what stage Rust-GB starts building (rust, llvm, c, asm)
    #[arg(short, long, default_value = "rust")]
    from: String,
}

#[derive(PartialEq, PartialOrd)]
enum BuildChain {
    Rust = 0,
    LLVM = 1,
    C = 2,
    ASM = 3,
}

impl FromStr for BuildChain {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rust" => Ok(BuildChain::Rust),
            "llvm" => Ok(BuildChain::LLVM),
            "c" => Ok(BuildChain::C),
            "asm" => Ok(BuildChain::ASM),
            _ => Err(()),
        }
    }
}

pub struct WorkingDirectory {
    root: String,
    ext: String,
    out: String,
}

impl WorkingDirectory {
    pub fn init(root: &str) -> WorkingDirectory {
        Self {
            root: root.to_string(),
            ext: Self::create_ext_dir(root),
            out: Self::create_out_dir(root),
        }
    }

    fn create_out_dir(root: &str) -> String {
        // Create output directory
        fs::create_dir_all(format!("{}/out", root)).unwrap();
        fs::create_dir_all(format!("{}/out/asm", root)).unwrap();

        format!("{}/out", root)
    }

    fn create_ext_dir(root: &str) -> String {
        let ext_dir = format!("{}/ext", root);

        fs::create_dir_all(&ext_dir).unwrap();
        if let Err(_) = EXT_DIR.extract(&ext_dir) {
            println!("{}", "EXT extraction failed".red());
            process::exit(1);
        };

        // Set execution permission to ext binaries.
        fs::set_permissions(
            format!("{}/llvm-link", &ext_dir),
            fs::Permissions::from_mode(0o770),
        )
        .unwrap();
        fs::set_permissions(
            format!("{}/llvm-cbe", &ext_dir),
            fs::Permissions::from_mode(0o770),
        )
        .unwrap();

        for entry in fs::read_dir(format!("{}/bin", &ext_dir)).unwrap() {
            fs::set_permissions(entry.unwrap().path(), fs::Permissions::from_mode(0o770)).unwrap();
        }

        format!("{}/ext", root)
    }

    // find c files from `./c` and `./ext/c`
    pub fn get_c_paths(&self) -> Vec<String> {
        let c_path = Command::new("find")
            .args([
                format!("{}/c", self.root).as_str(),
                format!("{}/c", self.ext).as_str(),
                "-name",
                "*.c",
            ])
            .output()
            .unwrap()
            .stdout;

        let c_path = String::from_utf8(c_path).unwrap();
        let c_path = c_path.trim().to_string();

        if c_path.len() == 0 {
            Vec::new()
        } else {
            let c_path = c_path.split("\n");
            c_path.map(|s| s.to_string()).collect()
        }
    }

    pub fn get_asm_paths(&self) -> Vec<String> {
        let asm_path = Command::new("find")
            .args([
                format!("{}/asm", self.root).as_str(),
                format!("{}/asm", self.out).as_str(),
                format!("{}/asm", self.ext).as_str(),
                "-name",
                "*.asm",
            ])
            .output()
            .unwrap()
            .stdout;
        let asm_path = String::from_utf8(asm_path).unwrap();
        let asm_path = asm_path.trim().to_string();

        if asm_path.len() == 0 {
            Vec::new()
        } else {
            let asm_path = asm_path.split("\n");
            asm_path.map(|s| s.to_string()).collect()
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BuildStepError {
    #[error("Command execution failed. Is {0} installed?")]
    ChildExecutionFailed(String),
    #[error("{0} exited with status {1}")]
    ChildProcessFailed(String, ExitStatus),
    #[error("File error: {0}")]
    FileError(#[from] std::io::Error),
}

pub trait BuildStep {
    fn run(dir: &WorkingDirectory) -> Result<(), BuildStepError>;
}

fn main() {
    let Subcommand::BuildRom(build_rom) = Subcommand::parse();
    let build_from = match BuildChain::from_str(&build_rom.from) {
        Ok(from) => from,
        Err(_) => {
            println!("{}", "[rust-gb] build from flag parse failed".red());
            process::exit(1)
        }
    };

    let root = Command::new("cargo")
        .args(["locate-project", "--message-format", "plain"])
        .output()
        .unwrap()
        .stdout;
    let root = String::from_utf8(root).unwrap();
    let (root, _) = root.rsplit_once('/').unwrap();

    let work_dir = WorkingDirectory::init(root);

    if build_from <= BuildChain::Rust {
        if !Cargo::run(&work_dir).is_ok() {
            println!("{}", "[cargo] Rust -> LLVM-IR compile failed".red());
            process::exit(1);
        } else {
            println!("{}", "[cargo] Rust -> LLVM-IR compile succeeded".green());
        }
    }

    if build_from <= BuildChain::LLVM {
        if !LlvmLink::run(&work_dir).is_ok() {
            println!("{}", "[llvm-link] LLVM-IR linking failed".red());
            process::exit(1);
        } else {
            println!("{}", "[llvm-link] LLVM-IR linking succeeded".green());
        }

        if !LlvmCbe::run(&work_dir).is_ok() {
            println!("{}", "[llvm-cbe] LLVM-IR -> C compile failed".red());
            process::exit(1);
        } else {
            println!("{}", "[llvm-cbe] LLVM-IR -> C compile succeeded".green());
        }

        //C postprocessing for SDCC
        fs::copy(
            format!("{}/out.c", work_dir.out),
            format!("{}/pre.c", work_dir.out),
        )
        .unwrap();

        if !Treesitter::run(&work_dir).is_ok() {
            println!("{}", "[treesitter] C postprocess for SDCC failed".red());
            process::exit(1);
        } else {
            println!(
                "{}",
                "[treesitter] C postprocess for SDCC succeeded".green()
            );
        }

        /*Command::new("sed")
            .args(["'s/static __forceinline/inline/g' -i ./out/out.c"])
            .status()
            .unwrap();
        Command::new("sed")
            .args(["'s/uint8_t\\* memset(uint8_t\\*, uint32_t, uint16_t);/inline uint8_t\\* memset(uint8_t\\* dst, uint8_t c, uint16_t sz) {uint8_t \\*p = dst; while (sz--) *p++ = c; return dst; }/g' -i ./out/out.c"])
            .status()
            .unwrap();
        Command::new("sed")
            .args(["/__noreturn void rust_begin_unwind(struct l_struct_core_KD__KD_panic_KD__KD_PanicInfo\\* llvm_cbe_info)/{:a;N;/__builtin_unreachable/{N;N;d};/  }/b;ba}' -i ./out/out.c"])
            .status()
            .unwrap();*/
    }

    if build_from <= BuildChain::C {
        if !Sdcc::run(&work_dir).is_ok() {
            println!("{}", "[sdcc] C -> ASM compile failed".red());
            process::exit(1);
        } else {
            println!("{}", "[sdcc] C -> ASM compile succeeded".green());
        }
    }

    if build_from <= BuildChain::ASM {
        if !Lcc::run(&work_dir).is_ok() {
            println!("{}", "[lcc] GBDK link failed".red());
            process::exit(1);
        } else {
            println!("{}", "[lcc] GBDK link succeeded".green());
        }
    }

    println!("{}", "GB ROM build succeeded".green());
}
