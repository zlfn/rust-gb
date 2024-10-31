use core::str;
use std::{fs::{self, File}, io::{ErrorKind, Write}, os::unix::fs::PermissionsExt, process::{self, Command, ExitStatus}, str::FromStr};
use clap::Parser;

use colored::Colorize;
use include_dir::{include_dir, Dir};
use project_root::get_project_root;
use tree_sitter::{self, Query, QueryCursor};

// Build GB ROM from Rust
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Start build chain from
    #[arg(short, long, default_value = "rust")]
    from: String,
}

// External files
static EXT_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/ext");

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
    fs::set_permissions(format!("{}/llvm-link", &ext_dir), fs::Permissions::from_mode(0o770)).unwrap();
    fs::set_permissions(format!("{}/llvm-cbe", &ext_dir), fs::Permissions::from_mode(0o770)).unwrap();

    for entry in fs::read_dir(format!("{}/bin", &ext_dir)).unwrap() {
        fs::set_permissions(entry.unwrap().path(), fs::Permissions::from_mode(0o770)).unwrap();
    }

    format!("{}/ext", root)
}

fn cargo_build(out: &str) -> ExitStatus {
    if let Ok(status) = Command::new("cargo")
        .args([
            "build",
            "--release",
            "--target-dir", out,
        ])
        .status() {
            status
    } else {
        println!("{}", "[cargo] cargo execution failed".red());
        process::exit(1);
    }
}

fn llvm_link(out: &str, ext: &str) -> ExitStatus {
    // TODO: Generalize llvm-link process
    let coreir = Command::new("find")
        .args([
            format!("{}/avr-unknown-gnu-atmega328/release/deps", out),
            "-name".to_string(),
            "core*.ll".to_string()
        ])
        .output()
        .unwrap()
        .stdout;
    let coreir = str::from_utf8(&coreir).unwrap().trim();

    let gbir = Command::new("find")
        .args([
            format!("{}/avr-unknown-gnu-atmega328/release/deps", out),
            "-name".to_string(),
            "gb*.ll".to_string()
        ])
        .output()
        .unwrap()
        .stdout;
    let gbir = str::from_utf8(&gbir).unwrap().trim();
    
    let romir = Command::new("find")
        .args([
            format!("{}/avr-unknown-gnu-atmega328/release/deps", out),
            "-name".to_string(),
            "rom*.ll".to_string()
        ])
        .output()
        .unwrap()
        .stdout;
    let romir = str::from_utf8(&romir).unwrap().trim();

    if let Ok(status) = Command::new(format!("{}/llvm-link", ext))
        .args([
            "--only-needed", "-S",
            romir,
            gbir,
            coreir,

            "-o", format!("{}/out.ll", out).as_str()
        ])
        .status() {
            status
    } else {
        println!("{}", "[llvm-link] llvm-link execution failed".red());
        process::exit(1);
    }
}

fn llvm_cbe(out: &str, ext: &str) -> ExitStatus {
    if let Ok(status) = Command::new(format!("{}/llvm-cbe", ext))
        .args([
            "--cbe-declare-locals-late",
            format!("{}/out.ll", out).as_str(),
            "-o", format!("{}/out.c", out).as_str(),
        ])
        .status() {
            status
    } else {
        println!("{}", "[llvm-cbe] llvm-cbe execution failed".red());
        process::exit(1);
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

fn treesitter_process(c_path: &str) -> Result<(), std::io::Error> {
    let mut code = fs::read_to_string(c_path)?;
    let code_bytes = code.clone();
    let code_bytes = code_bytes.as_bytes();
    parse_declarator_attributes(&mut code, code_bytes);

    let code_bytes = code.clone();
    let code_bytes = code_bytes.as_bytes();
    parse_call_expression_attributes(&mut code, code_bytes);

    let mut file = File::create(c_path)?;
    file.write_all(&code.as_bytes())?;

    //Remove All Global Variable Declarations (Because it is mostly duplicated with Global Variable Definitions)
    //TODO: It can cause problems, so it needs to be replaced with a more sophisticated logic 
    let s = Command::new("sed")
        .args([
            "/.*Global Variable Declarations.*/,/.*Function Declarations.*/{/^\\//!d;}",
            "-i", c_path
        ])
        .status()?;

    if !s.success() {
        return Err(std::io::Error::new(ErrorKind::NotFound, "sed failed"))
    }

    Ok(())
}

// find c files from `./c` and `./ext/c`
fn get_c_paths(root: &str, ext: &str) -> Vec<String> {
    let c_path = Command::new("find")
        .args([
            format!("{}/c", root).as_str(),
            format!("{}/c", ext).as_str(),

            "-name", "*.c"
        ])
        .output()
        .unwrap()
        .stdout;

    let c_path = String::from_utf8(c_path).unwrap();
    let c_path = c_path.trim().to_string();

    if c_path.len() == 0 {
        Vec::new()
    }
    else {
        let c_path = c_path.split("\n");
        c_path.map(|s| s.to_string()).collect()
    }
}

fn sdcc_compile(c_path: &str, asm_path: &str) -> ExitStatus {
    if let Ok(status) = Command::new("sdcc")
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

            c_path,
            "-o", asm_path,
        ])
        .status() {
            status
    } else {
        println!("{}", "[sdcc] sdcc execution failed".red());
        process::exit(1);
    }
}

fn get_asm_paths(root: &str, out: &str, ext: &str) -> Vec<String> {
    let asm_path = Command::new("find")
        .args([
            format!("{}/asm", root).as_str(),
            format!("{}/asm", out).as_str(),
            format!("{}/asm", ext).as_str(),
            "-name", "*.asm"
        ])
        .output()
        .unwrap()
        .stdout;
    let asm_path = String::from_utf8(asm_path).unwrap();
    let asm_path = asm_path.trim().to_string();

    if asm_path.len() == 0 {
        Vec::new()
    }
    else {
        let asm_path = asm_path.split("\n");
        asm_path.map(|s| s.to_string()).collect()
    }
}

fn main() {
    let args = Args::parse();
    let build_from = BuildChain::from_str(&args.from).unwrap();

    let root = get_project_root().unwrap();
    let root = root.to_str().unwrap();

    let out_dir = create_out_dir(root);
    let ext_dir = create_ext_dir(root);

    if build_from <= BuildChain::Rust {
        if !cargo_build(&out_dir).success() {
            println!("{}", "[cargo] Rust -> LLVM-IR compile failed".red());
            process::exit(1);
        }
        else {
            println!("{}", "[cargo] Rust -> LLVM-IR compile succeeded".green());
        }
    }

    if build_from <= BuildChain::LLVM {

        if !llvm_link(&out_dir, &ext_dir).success() {
            println!("{}", "[llvm-link] LLVM-IR linking failed".red());
            process::exit(1);
        }
        else {
            println!("{}", "[llvm-link] LLVM-IR linking succeeded".green());
        }

        if !llvm_cbe(&out_dir, &ext_dir).success() {
            println!("{}", "[llvm-cbe] LLVM-IR -> C compile failed".red());
            process::exit(1);
        }
        else {
            println!("{}", "[llvm-cbe] LLVM-IR -> C compile succeeded".green());
        }

        //C postprocessing for SDCC
        fs::copy("./out/out.c", "./out/pre.c").unwrap();
        let c_path = format!("{}/out.c", out_dir);

        if treesitter_process(&c_path).is_err() {
            println!("{}", "[treesitter] C postprocess for SDCC failed".red());
            process::exit(1);
        }
        else {
            println!("{}", "[treesitter] C postprocess for SDCC succeeded".green());
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
        let c_file = format!("{}/out.c", out_dir);
        let asm_file = format!("{}/out.asm", out_dir);

        if !sdcc_compile(&c_file, &asm_file).success() {
            println!("{}", "[sdcc] C -> ASM code compile failed".red());
            process::exit(1);
        }

        let c_path = get_c_paths(&root, &ext_dir);
        println!("{:?}", c_path);
        for c_file in c_path {
            let (_, out_name) = c_file.rsplit_once('/').unwrap();
            let out_name = format!("{}/asm/{}.asm", out_dir, out_name);
            if !sdcc_compile(&c_file, out_name.as_str()).success() {
                println!("{}", "[sdcc] C -> ASM library compile failed".red());
                process::exit(1);
            }
        }

        println!("{}", "[sdcc] C -> ASM compile succeeded".green());
    }

    if build_from <= BuildChain::ASM {
        let asm_path = get_asm_paths(&root, &out_dir, &ext_dir);

        let mut lcc_args: Vec<&str> = vec!["-msm83:gb", "-o", "./out/out.gb", "./out/out.asm"];

        lcc_args.append(&mut asm_path.iter().map(|s| s.as_str()).collect());

        let lcc_status = Command::new(format!("{}/bin/lcc", ext_dir))
            .args(lcc_args)
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

