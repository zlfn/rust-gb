use core::str;
use std::{fs, io::{ErrorKind, Write}, process::{self, Command}, str::FromStr};
use clap::Parser;

use colored::Colorize;

///Build GB ROM from Rust
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Start build chain from
    #[arg(short, long, default_value = "rust")]
    from: String,
}

#[derive(PartialEq, PartialOrd)]
enum BuildChain {
    Rust = 0,
    Bundle = 1,
    LLVM = 2,
    C = 3,
    ASM = 4,
}

impl FromStr for BuildChain {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "rust" => Ok(BuildChain::Rust),
            "bundle" => Ok(BuildChain::Bundle),
            "llvm" => Ok(BuildChain::LLVM),
            "c" => Ok(BuildChain::C),
            "asm" => Ok(BuildChain::ASM),
            _ => Err(()),
        }
    }
}

fn main() {
    let args = Args::parse();
    let build_from = BuildChain::from_str(&args.from).unwrap();

    fs::create_dir_all("./out").unwrap();

    if build_from <= BuildChain::Rust {
        let bundle_result = Command::new("rust_bundler_cp")
            .args([
                "--input", "./source",
            ])
            .output()
            .unwrap();

        let mut file = fs::File::create("./out/out.rs").unwrap();
        let bundle_result = file.write_all(&bundle_result.stdout);

        if let Err(_) = bundle_result {
            println!("{}", "[rust_bundler_cp] Rust bundling failed".red());
            process::exit(1);
        }
        else {
            println!("{}", "[rust_bundler_cp] Rust bundling succeeded".green());
        }
    }


    if build_from <= BuildChain::Bundle {

        let libcompiler = Command::new("find")
            .args([
                "./ext/rust-deps/target/avr-unknown-gnu-atmega328/release/deps",
                "-name",
                "libcompiler*.rlib"
            ])
            .output()
            .unwrap()
            .stdout;
        let libcompiler = str::from_utf8(&libcompiler).unwrap().trim();

        let libcore = Command::new("find")
            .args([
                "./ext/rust-deps/target/avr-unknown-gnu-atmega328/release/deps",
                "-name",
                "libcore*.rlib"
            ])
            .output()
            .unwrap()
            .stdout;
        let libcore = str::from_utf8(&libcore).unwrap().trim();

        let liballoc = Command::new("find")
            .args([
                "./ext/rust-deps/target/avr-unknown-gnu-atmega328/release/deps",
                "-name",
                "liballoc*.rlib"
            ])
            .output()
            .unwrap()
            .stdout;
        let liballoc = str::from_utf8(&liballoc).unwrap().trim();

        let rustc_status = Command::new("rustc")
            .args([
                "--emit=llvm-ir",
                "--target", "avr-unknown-gnu-atmega328",
                "-C", "opt-level=3",
                "-C", "embed-bitcode=no",
                "-C", "panic=abort",
                "-Z", "unstable-options",

                "-L", "dependency=./ext/rust-deps/target/avr-unknown-gnu-atmega328/release/deps",
                "--extern", format!("noprelude:compiler_builtins={}", libcompiler).as_str(),
                "--extern", format!("noprelude:core={}", libcore).as_str(),
                "--extern", format!("noprelude:alloc={}", liballoc).as_str(),

                "./out/out.rs",
                "-o", "./out/out.ll"
            ])
            .status()
            .unwrap();

        if !rustc_status.success() {
            println!("{}", "[rustc] Rust -> LLVM-IR compile failed".red());
            process::exit(1);
        }
        else {
            println!("{}", "[rustc] Rust -> LLVM-IR compile succeeded".green());
        }

        fs::copy("./out/out.ll", "./out/pre.ll").unwrap();
        let postprocess_status = llvm_post_process();


        if postprocess_status.is_err() {
            println!("{}", "[sed] LLVM-IR postprocess for LLVM-CBE failed".red());
            process::exit(1);
        }
        else {
            println!("{}", "[sed] LLVM-IR postprocess for LLVM-CBE succeeded".green());
        }
    }

    if build_from <= BuildChain::LLVM {
        let llvm_status = Command::new("./ext/llvm-project/llvm/build/bin/llvm-cbe")
            .args([
                "--cbe-declare-locals-late",
                "./out/out.ll",
                "-o", "./out/out.c",
            ])
            .status()
            .unwrap();

        if !llvm_status.success() {
            println!("{}", "[llvm-cbe] LLVM-IR -> C compile failed".red());
            process::exit(1);
        }
        else {
            println!("{}", "[llvm-cbe] LLVM-IR -> C compile succeeded".green());
        }

        //C postprocessing for SDCC
        //Command::new("sed 's/static __forceinline/inline/g' -i ./out/main.c").status().unwrap();
        //Command::new("sed 's/uint8_t\\* memset(uint8_t\\*, uint32_t, uint16_t);/inline uint8_t\\* memset(uint8_t\\* dst, uint8_t c, uint16_t sz) {uint8_t \\*p = dst; while (sz--) *p++ = c; return dst; }/g' -i ./out/main.c").status().unwrap();
        //Command::new("sed '/__noreturn void rust_begin_unwind(struct l_struct_core_KD__KD_panic_KD__KD_PanicInfo\\* llvm_cbe_info)/{:a;N;/__builtin_unreachable/{N;N;d};/  }/b;ba}' -i ./out/main.c").status().unwrap();
        fs::copy("./out/out.c", "./out/pre.c").unwrap();
        let postprocess_status = c_post_process();

        if postprocess_status.is_err() {
            println!("{}", "[sed] C postprocess for SDCC failed".red());
            process::exit(1);
        }
        else {
            println!("{}", "[sed] C postprocess for SDCC succeeded".green());
        }
    }

    if build_from <= BuildChain::C {
        let sdcc_status = Command::new("sdcc")
            .args([
                "-S", "-linc", "-lsrc", "-ltests",
                "-D", "__HIDDEN__=",
                "-D", "__attribute__(a)=",
                "-D", "__builtin_unreachable()=while(1);",
                "--out-fmt-ihx",
                "--max-allocs-per-node", "2000",
                "--allow-unsafe-read",
                "--opt-code-speed",
                "--no-std-crt0",
                "--nostdlib",
                "--no-xinit-opt",
                "--std-sdcc11",

                "-msm83",
                "-D", "__PORT_sm83",
                "-D", "__TARGET_gb",

                "./out/out.c",
                "-o", "./out/out.asm"
            ])
            .status()
            .unwrap();

        if !sdcc_status.success() {
            println!("{}", "[sdcc] C -> ASM compile failed".red());
            process::exit(1);
        }
        else {
            println!("{}", "[sdcc] C -> ASM compile succeeded".green());
        }
    }

    if build_from <= BuildChain::ASM {
        let lcc_status = Command::new("./ext/bin/lcc")
            .args([
                "-o", "./out/out.gb",
                "./out/out.asm"
            ])
            .status()
            .unwrap();

        if !lcc_status.success() {
            println!("{}", "[lcc] GBDK link failed".red());
            process::exit(1);
        }
        else {
            println!("{}", "[lcc] GBDK link succeeded".green());
        }

        println!("{}", "GB ROM build succeeded".green());
    }
}

fn llvm_post_process() -> Result<(), std::io::Error> {
    //`or disjoint` -> `or`
    let s = Command::new("sed")
        .args([
            "s/or disjoint/or/g",
            "-i", "./out/out.ll",
        ])
        .status()?;

    if !s.success() {
        return Err(std::io::Error::new(ErrorKind::NotFound, "sed failed"))
    }

    Ok(())
}

fn c_post_process() -> Result<(), std::io::Error> {
    //Add sdcc calling convention attributes to functions that have #[link_name="`function_name` __`convention`"]
    //__sdcccall(0)
    let s = Command::new("sed")
        .args([
            "/\\(void\\|uint8_t\\) .*\\(_AC___sdcccall_IC_0_JC_\\).*/ s/((nothrow));/((nothrow)) __sdcccall(0);/g;s/_AC___sdcccall_IC_0_JC_//g",
            "-i", "./out/out.c"
        ])
        .status()?;

    if !s.success() {
        return Err(std::io::Error::new(ErrorKind::NotFound, "sed failed"))
    }

    //__sdcccall(1)
    let s = Command::new("sed")
        .args([
            "/\\(void\\|uint8_t\\) .*\\(_AC___sdcccall_IC_1_JC_\\).*/ s/((nothrow));/((nothrow)) __sdcccall(1);/g;s/_AC___sdcccall_IC_1_JC_//g",
            "-i", "./out/out.c"
        ])
        .status()?;

    if !s.success() {
        return Err(std::io::Error::new(ErrorKind::NotFound, "sed failed"))
    }

    //__nonbanked
    let s = Command::new("sed")
        .args([
            "/void.*\\(_AC___nonbanked\\).*/ s/;/ __nonbanked;/g;s/_AC___nonbanked//g",
            "-i", "./out/out.c"
        ])
        .status()?;

    if !s.success() {
        return Err(std::io::Error::new(ErrorKind::NotFound, "sed failed"))
    }

    //Remove All Global Variable Declarations (Because it is mostly duplicated with Global Variable Definitions)
    //TODO: It can cause problems, so it needs to be replaced with a more sophisticated logic 
    let s = Command::new("sed")
        .args([
            "/.*Global Variable Declarations.*/,/.*Function Declarations.*/{/^\\//!d;}",
            "-i", "./out/out.c"
        ])
        .status()?;

    if !s.success() {
        return Err(std::io::Error::new(ErrorKind::NotFound, "sed failed"))
    }

    Ok(())
}
