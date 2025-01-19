use core::str;
use std::process::Command;

use colored::Colorize;
use indicatif::ProgressBar;

use crate::{BuildStep, BuildStepError};

pub struct LlvmLink {}
pub struct LlvmCbe {}

impl BuildStep for LlvmLink {
    fn run(dir: &crate::WorkingDirectory, bar: &ProgressBar) -> Result<(), BuildStepError> {
        bar.set_message("LLVM-IR Linking...");
        let coreir = Command::new("find")
            .args([
                format!("{}/avr-unknown-gnu-atmega328/release/deps", dir.out),
                "-name".to_string(),
                "core*.ll".to_string(),
            ])
            .output()
            .unwrap()
            .stdout;
        let coreir = str::from_utf8(&coreir).unwrap().trim();

        let gbir = Command::new("find")
            .args([
                format!("{}/avr-unknown-gnu-atmega328/release/deps", dir.out),
                "-name".to_string(),
                "gb*.ll".to_string(),
            ])
            .output()
            .unwrap()
            .stdout;
        let gbir = str::from_utf8(&gbir).unwrap().trim();

        let romir = Command::new("find")
            .args([
                format!("{}/avr-unknown-gnu-atmega328/release/deps", dir.out),
                "-name".to_string(),
                "rom*.ll".to_string(),
            ])
            .output()
            .unwrap()
            .stdout;
        let romir = str::from_utf8(&romir).unwrap().trim();

        if let Ok(output) = Command::new(format!("{}/llvm-link", dir.ext))
            .args([
                "--only-needed",
                "-S",
                romir,
                gbir,
                coreir,
                "-o",
                format!("{}/out.ll", dir.out).as_str(),
            ])
            .output()
        {
            if output.status.success() {
                bar.finish_with_message(format!("{}", "LLVM-IR Linking Succeeded".green()));
                return Ok(());
            } else {
                bar.println(String::from_utf8_lossy(&output.stderr));
                bar.finish_with_message(format!("{}", "LLVM-IR Linking Failed".red()));
                return Err(BuildStepError::ChildProcessFailed(
                    "llvm-link".to_string(),
                    output.status,
                ));
            }
        } else {
            bar.println("llvm-link execution failed");
            bar.finish_with_message(format!("{}", "LLVM Linking Failed".red()));
            return Err(BuildStepError::ChildExecutionFailed(
                "llvm-link".to_string(),
            ));
        }
    }
}

impl BuildStep for LlvmCbe {
    fn run(dir: &crate::WorkingDirectory, bar: &ProgressBar) -> Result<(), BuildStepError> {
        bar.set_message("LLVM-IR -> C Compiling...");
        if let Ok(output) = Command::new(format!("{}/llvm-cbe", dir.ext))
            .args([
                "--cbe-declare-locals-late",
                format!("{}/out.ll", dir.out).as_str(),
                "-o",
                format!("{}/out.c", dir.out).as_str(),
            ])
            .output()
        {
            if output.status.success() {
                bar.finish_with_message(format!("{}", "LLVM-IR -> C Compiling Succeeded".green()));
                return Ok(());
            } else {
                bar.println(String::from_utf8_lossy(&output.stderr));
                bar.finish_with_message(format!("{}", "LLVM-IR -> C Compiling Failed".red()));
                return Err(BuildStepError::ChildProcessFailed(
                    "llvm-cbe".to_string(),
                    output.status,
                ));
            }
        } else {
            bar.println("llvm-cbe execution failed");
            bar.finish_with_message(format!("{}", "LLVM-IR -> C Compiling Failed".red()));
            return Err(BuildStepError::ChildExecutionFailed("llvm-cbe".to_string()));
        }
    }
}
