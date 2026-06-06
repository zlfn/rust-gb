use std::env;
use std::path::Path;

// Drop the HRAM allocation script into OUT_DIR. The build pipeline links every
// `*.ld` it finds there, so it is only present when gb-hram is a dependency.
fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    std::fs::copy(
        Path::new(&manifest_dir).join("gb_hram.ld"),
        Path::new(&out_dir).join("gb_hram.ld"),
    )
    .expect("failed to copy gb_hram.ld");
    println!("cargo:rerun-if-changed=gb_hram.ld");
}
