use std::process::{self, Command};

use colored::Colorize;

fn main() {
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
