use core::str;
use std::process::Command;

use colored::Colorize;

use crate::{BuildStep, BuildStepError};

pub struct LlvmLink {}
pub struct LlvmCbe {}

impl BuildStep for LlvmLink {
    fn run(dir: &crate::WorkingDirectory) -> Result<(), BuildStepError> {
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

        if let Ok(status) = Command::new(format!("{}/llvm-link", dir.ext))
            .args([
                "--only-needed",
                "-S",
                romir,
                gbir,
                coreir,
                "-o",
                format!("{}/out.ll", dir.out).as_str(),
            ])
            .status()
        {
            if status.success() {
                return Ok(());
            } else {
                return Err(BuildStepError::ChildProcessFailed(
                    "llvm-link".to_string(),
                    status,
                ));
            }
        } else {
            println!("{}", "[llvm-link] llvm-link execution failed".red());
            return Err(BuildStepError::ChildExecutionFailed(
                "llvm-link".to_string(),
            ));
        }
    }
}

impl BuildStep for LlvmCbe {
    fn run(dir: &crate::WorkingDirectory) -> Result<(), BuildStepError> {
        if let Ok(status) = Command::new(format!("{}/llvm-cbe", dir.ext))
            .args([
                "--cbe-declare-locals-late",
                format!("{}/out.ll", dir.out).as_str(),
                "-o",
                format!("{}/out.c", dir.out).as_str(),
            ])
            .status()
        {
            if status.success() {
                return Ok(());
            } else {
                return Err(BuildStepError::ChildProcessFailed(
                    "llvm-cbe".to_string(),
                    status,
                ));
            }
        } else {
            println!("{}", "[llvm-cbe] llvm-cbe execution failed".red());
            return Err(BuildStepError::ChildExecutionFailed("llvm-cbe".to_string()));
        }
    }
}
