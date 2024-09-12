use std::{io::ErrorKind, process::{self, Command}};

use colored::Colorize;

fn main() {
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
    let postprocess_status = postprocess();

    if postprocess_status.is_err() {
        println!("{}", "[sed] C postprocess for SDCC failed".red());
        process::exit(1);
    }
    else {
        println!("{}", "[sed] C postprocess for SDCC succeeded".green());
    }
    
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

fn postprocess() -> Result<(), std::io::Error> {
    //Add sdcc calling convention attributes to functions that have #[link_name="`function_name` __sdcccall"]
    //"__sdcccall(0)"
    let s = Command::new("sed")
        .args([
            "/void.*\\(_AC___sdcccall_IC_0_JC_\\).*/ s/;/ __sdcccall(0);/g;s/_AC___sdcccall_IC_0_JC_//g",
            "-i", "./out/main.c"
        ])
        .status()?;

    if !s.success() {
        return Err(std::io::Error::new(ErrorKind::NotFound, "sed failed"))
    }

    //"__sdcccall(1)"
    let s = Command::new("sed")
        .args([
            "/void.*\\(_AC___sdcccall_IC_1_JC_\\).*/ s/;/ __sdcccall(1);/g;s/_AC___sdcccall_IC_1_JC_//g",
            "-i", "./out/main.c"
        ])
        .status()?;

    if !s.success() {
        return Err(std::io::Error::new(ErrorKind::NotFound, "sed failed"))
    }

    Ok(())
}
