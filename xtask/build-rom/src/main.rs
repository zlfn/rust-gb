use core::str;
use std::{fs::{self, File}, io::{ErrorKind, Write}, process::{self, Command}, str::FromStr};
use clap::Parser;

use colored::Colorize;
use project_root::get_project_root;
use tree_sitter::{self, Query, QueryCursor};

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

    let root = get_project_root().unwrap();
    let root = root.to_str().unwrap();

    fs::create_dir_all("./out").unwrap();

    if build_from <= BuildChain::Rust {
        let bundle_result = Command::new("rust_bundler_cp")
            .args([
                "--input", ".",
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
                format!("{}/ext/rust-deps/target/avr-unknown-gnu-atmega328/release/deps", root),
                "-name".to_string(),
                "libcompiler*.rlib".to_string()
            ])
            .output()
            .unwrap()
            .stdout;
        let libcompiler = str::from_utf8(&libcompiler).unwrap().trim();

        let libcore = Command::new("find")
            .args([
                format!("{}/ext/rust-deps/target/avr-unknown-gnu-atmega328/release/deps", root),
                "-name".to_string(),
                "libcore*.rlib".to_string()
            ])
            .output()
            .unwrap()
            .stdout;
        let libcore = str::from_utf8(&libcore).unwrap().trim();

        let liballoc = Command::new("find")
            .args([
                format!("{}/ext/rust-deps/target/avr-unknown-gnu-atmega328/release/deps", root),
                "-name".to_string(),
                "liballoc*.rlib".to_string()
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

                "-L", format!("dependency={}/ext/rust-deps/target/avr-unknown-gnu-atmega328/release/deps", root).as_str(),
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
    }

    if build_from <= BuildChain::LLVM {
        let llvm_status = Command::new(format!("{}/ext/llvm-project/llvm/build/bin/llvm-cbe", root))
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
        fs::copy("./out/out.c", "./out/pre.c").unwrap();
        let postprocess_status = treesitter_process();

        if postprocess_status.is_err() {
            println!("{}", "[treesitter] C postprocess for SDCC failed".red());
            process::exit(1);
        }
        else {
            println!("{}", "[treesitter] C postprocess for SDCC succeeded".green());
        }

        Command::new("sed")
            .args(["s/static __forceinline/inline/g' -i ./out/out.c"])
            .status()
            .unwrap();
        Command::new("sed")
            .args(["s/uint8_t\\* memset(uint8_t\\*, uint32_t, uint16_t);/inline uint8_t\\* memset(uint8_t\\* dst, uint8_t c, uint16_t sz) {uint8_t \\*p = dst; while (sz--) *p++ = c; return dst; }/g' -i ./out/out.c"])
            .status()
            .unwrap();
        Command::new("sed")
            .args(["/__noreturn void rust_begin_unwind(struct l_struct_core_KD__KD_panic_KD__KD_PanicInfo\\* llvm_cbe_info)/{:a;N;/__builtin_unreachable/{N;N;d};/  }/b;ba}' -i ./out/out.c"])
            .status()
            .unwrap();
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
                "--disable-warning", "110",
                "--disable-warning", "126",
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
        let lcc_status = Command::new(format!("{}/ext/bin/lcc", root))
            .args([
                "-msm83:gb",
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

fn parse_declarator_attributes(code: &mut String, code_bytes: &[u8]) {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_c::LANGUAGE.into()).unwrap();

    let tree = parser.parse(code_bytes, None).unwrap();
    let query = Query::new(&tree_sitter_c::LANGUAGE.into(), "(function_declarator(identifier)@a)@b").unwrap();
    let mut cursor = QueryCursor::new();

    let mut diff: i32 = 0;

    for qm in cursor.matches(&query, tree.root_node(), code_bytes) {
        let declarator_node = qm.captures[0].node;
        let identifier_node = qm.captures[1].node;
        let declarator = declarator_node.utf8_text(&code_bytes).unwrap();
        let identifier = identifier_node.utf8_text(&code_bytes).unwrap();
        let attributes: Vec<_> = identifier.split("_AC___").collect(); //` __`

        if attributes.len() < 2 {
            continue;
        }

        match &attributes[..] {
            [] | [_] => {},
            [_, attrib @ ..] => {
                let attrib = format!("__{}", attrib.join(" __"));
                let attrib = attrib
                    .replace("_AC_", " ")
                    .replace("_IC_", "(")
                    .replace("_JC_", ")")
                    .replace("_MC_", ",");

                let start = declarator.find("_AC_").unwrap();
                let end = declarator.find("(").unwrap();
                let mut declarator = String::from(declarator);
                declarator.replace_range(start..end, "");
                
                code.replace_range(
                    ((declarator_node.start_byte() as i32) + diff) as usize..
                    ((declarator_node.end_byte() as i32) + diff) as usize, 
                    &format!("{} {}", declarator, attrib));
                diff += (declarator.len() + attrib.len() + 1) as i32 - declarator_node.byte_range().len() as i32;
            }
        }
    }
}

fn parse_call_expression_attributes(code: &mut String, code_bytes: &[u8]) {
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&tree_sitter_c::LANGUAGE.into()).unwrap();

    let tree = parser.parse(code_bytes, None).unwrap();
    let query = Query::new(&tree_sitter_c::LANGUAGE.into(), "(call_expression(identifier)@a)").unwrap();
    let mut cursor = QueryCursor::new();

    let mut diff: i32 = 0;

    for qm in cursor.matches(&query, tree.root_node(), code_bytes) {
        let identifier_node = qm.captures[0].node;
        let identifier = identifier_node.utf8_text(&code_bytes).unwrap();

        let start = identifier.find("_AC_");
        let start = match start {
            None => continue,
            Some(start) => start
        };

        let mut identifier = String::from(identifier);
        identifier.replace_range(start.., "");

        code.replace_range(
            ((identifier_node.start_byte() as i32) + diff) as usize..
            ((identifier_node.end_byte() as i32) + diff) as usize, 
            &identifier);
        diff += identifier.len() as i32 - identifier_node.byte_range().len() as i32;
    }
}

fn treesitter_process() -> Result<(), std::io::Error> {
    let mut code = fs::read_to_string("./out/out.c")?;
    let code_bytes = code.clone();
    let code_bytes = code_bytes.as_bytes();
    parse_declarator_attributes(&mut code, code_bytes);

    let code_bytes = code.clone();
    let code_bytes = code_bytes.as_bytes();
    parse_call_expression_attributes(&mut code, code_bytes);

    let mut file = File::create("./out/out.c")?;
    file.write_all(&code.as_bytes())?;

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
