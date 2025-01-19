use std::process::{Command, Stdio};

use colored::Colorize;
use indicatif::ProgressBar;

use crate::{BuildStep, BuildStepError};

pub struct Cargo {}

impl BuildStep for Cargo {
    fn run(dir: &crate::WorkingDirectory, bar: &ProgressBar) -> Result<(), BuildStepError> {
        bar.set_message("Rust -> LLVM-IR Compiling...");
        if let Ok(output) = bar.suspend(|| {
            Command::new("cargo")
                .args(["build", "--release", "--target-dir", dir.out.as_str()])
                .stderr(Stdio::inherit())
                .output()
        }) {
            if output.status.success() {
                bar.finish_with_message(format!(
                    "{}",
                    "Rust -> LLVM-IR Compiling Succeeded".green()
                ));
                return Ok(());
            } else {
                bar.println(String::from_utf8_lossy(&output.stderr));
                bar.finish_with_message(format!("{}", "Rust -> LLVM-IR Compiling Failed".red()));
                return Err(BuildStepError::ChildProcessFailed(
                    "cargo".to_string(),
                    output.status,
                ));
            }
        } else {
            bar.println("cargo execution failed");
            bar.finish_with_message(format!("{}", "Rust -> LLVM-IR Compiling Failed".red()));
            return Err(BuildStepError::ChildExecutionFailed("cargo".to_string()));
        }
    }
}
