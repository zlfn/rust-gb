use std::env;
use std::path::Path;

// Drop the RAM-function placement script into OUT_DIR. The build pipeline links
// every `*.ld` it finds there, so it is only present when gb-ram-fn is a
// dependency.
fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    std::fs::copy(
        Path::new(&manifest_dir).join("gb_ram_fn.ld"),
        Path::new(&out_dir).join("gb_ram_fn.ld"),
    )
    .expect("failed to copy gb_ram_fn.ld");
    println!("cargo:rerun-if-changed=gb_ram_fn.ld");
}
