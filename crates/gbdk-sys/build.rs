use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let lib_dir = format!("{manifest_dir}/lib");

    // Find LLVM_Z80_BIN from environment (set via .env)
    let llvm_bin = std::env::var("LLVM_Z80_BIN")
        .expect("LLVM_Z80_BIN not set. Define it in .env at the workspace root.");

    // Assemble gbdk_init.s → gbdk_init.o
    let init_s = Path::new(manifest_dir).join("gbdk_init.s");
    let init_o = Path::new(&out_dir).join("gbdk_init.o");
    let status = Command::new(format!("{llvm_bin}/clang"))
        .args(["--target=sm83", "-c"])
        .arg(&init_s)
        .arg("-o")
        .arg(&init_o)
        .status()
        .expect("failed to run clang for gbdk_init.s");
    assert!(status.success(), "clang failed to assemble gbdk_init.s");

    // Create static library from gbdk_init.o
    let init_a = Path::new(&out_dir).join("libgbdk_init.a");
    let status = Command::new("ar")
        .args(["rcs"])
        .arg(&init_a)
        .arg(&init_o)
        .status()
        .expect("failed to run ar");
    assert!(status.success(), "ar failed to create libgbdk_init.a");

    // Link GBDK libraries
    println!("cargo:rustc-link-search=native={lib_dir}");
    println!("cargo:rustc-link-search=native={out_dir}");
    println!("cargo:rustc-link-lib=static=gbdk_init");
    println!("cargo:rustc-link-lib=static=gb");
    println!("cargo:rustc-link-lib=static=sm83");

    // Drop our linker script into OUT_DIR. The build pipeline links every
    // `*.ld` it finds under each crate's build-script OUT_DIR, so this is picked
    // up automatically without the Makefile knowing about gbdk.
    let ld_src = Path::new(manifest_dir).join("gbdk.ld");
    let ld_dst = Path::new(&out_dir).join("gbdk.ld");
    std::fs::copy(&ld_src, &ld_dst).expect("failed to copy gbdk.ld");

    println!("cargo:rerun-if-changed=gbdk_init.s");
    println!("cargo:rerun-if-changed=gbdk.ld");
    println!("cargo:rerun-if-changed=lib/libgb.a");
    println!("cargo:rerun-if-changed=lib/libsm83.a");
}
