use std::path::Path;

// gbdk_init.s is compiled into the crate via global_asm! (see src/lib.rs), so no
// host assembler is invoked here. This build script only wires up the precompiled
// GBDK static libraries and drops the linker script into OUT_DIR.
fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let lib_dir = format!("{manifest_dir}/lib");

    // Precompiled GBDK libraries.
    println!("cargo:rustc-link-search=native={lib_dir}");
    println!("cargo:rustc-link-lib=static=gb");
    println!("cargo:rustc-link-lib=static=sm83");

    // Drop our linker script into OUT_DIR. The build pipeline links every `*.ld`
    // it finds under each crate's build-script OUT_DIR, so this is picked up
    // automatically without the Makefile knowing about gbdk.
    let ld_src = Path::new(manifest_dir).join("gbdk.ld");
    let ld_dst = Path::new(&out_dir).join("gbdk.ld");
    std::fs::copy(&ld_src, &ld_dst).expect("failed to copy gbdk.ld");

    println!("cargo:rerun-if-changed=gbdk_init.s");
    println!("cargo:rerun-if-changed=gbdk.ld");
    println!("cargo:rerun-if-changed=lib/libgb.a");
    println!("cargo:rerun-if-changed=lib/libsm83.a");
}
