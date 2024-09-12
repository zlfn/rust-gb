use std::{fs, process::{self, Command}};

use colored::Colorize;

fn main() {
    fs::create_dir_all("./out").unwrap();
    let rustc_status = Command::new("rustc")
        .args([
            "--emit=llvm-ir",
            "--target", "avr-unknown-gnu-atmega328",
            "-C", "opt-level=3",
            "-C", "embed-bitcode=no",
            "-C", "panic=abort",
            "-Z", "unstable-options",

            //TODO: self compile these dependencies
            "-L", "dependency=/home/zlfn/rust-z80/target/avr-unknown-gnu-atmega328/release/deps",
            "--extern", "noprelude:compiler_builtins=/home/zlfn/rust-z80/target/avr-unknown-gnu-atmega328/release/deps/libcompiler_builtins-7c8cb6a88df6298c.rlib",
            "--extern", "noprelude:core=/home/zlfn/rust-z80/target/avr-unknown-gnu-atmega328/release/deps/libcore-19ea3eec74d36e50.rlib",

            "./source/src/main.rs",
            "-o", "./out/main.ll"
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

    let llvm_status = Command::new("./ext/llvm-project/llvm/build/bin/llvm-cbe")
        .args([
            "--cbe-declare-locals-late",
            "./out/main.ll",
            "-o", "./out/main.c",
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

            "./out/main.c",
            "-o", "./out/main.asm"
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

    let lcc_status = Command::new("./ext/bin/lcc")
        .args([
            "-o", "./out/main.gb",
            "./out/main.asm"
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
