use std::path::Path;

// Drop the linker script (gb.ld) into OUT_DIR, where the ROM build pipeline picks
// it up by globbing each crate's OUT_DIR (the same way gbdk-sys delivers
// gbdk.ld). The startup (rrt0.s) is compiled into the crate via global_asm!, so
// no assembler is invoked here.
fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let out_dir = std::env::var("OUT_DIR").unwrap();

    std::fs::copy(
        Path::new(manifest_dir).join("gb.ld"),
        Path::new(&out_dir).join("gb.ld"),
    )
    .expect("failed to copy gb.ld");

    println!("cargo:rerun-if-changed=gb.ld");
}
