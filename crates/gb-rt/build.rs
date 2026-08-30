use std::path::Path;

// Drop the linker scripts into OUT_DIR, where the ROM build pipeline picks them
// up by globbing each crate's OUT_DIR for `*.ld`. The startup (rrt0.s) is
// compiled into the crate via global_asm!, so no assembler is invoked here.
fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let out_dir = std::env::var("OUT_DIR").unwrap();

    for script in ["gb.ld", "gb_defaults.ld"] {
        std::fs::copy(
            Path::new(manifest_dir).join(script),
            Path::new(&out_dir).join(script),
        )
        .unwrap_or_else(|e| panic!("failed to copy {script}: {e}"));
        println!("cargo:rerun-if-changed={script}");
    }
}
