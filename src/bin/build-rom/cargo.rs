use std::process::{self, Command};

use colored::Colorize;

use crate::{BuildStep, BuildStepError};

pub struct Cargo {}

impl BuildStep for Cargo {
    fn run(dir: &crate::WorkingDirectory) -> Result<(), BuildStepError> {
        if let Ok(status) = Command::new("cargo")
            .args(["build", "--release", "--target-dir", dir.out.as_str()])
            .status()
        {
            if status.success() {
                return Ok(());
            } else {
                return Err(BuildStepError::ChildProcessFailed(
                    "cargo".to_string(),
                    status,
                ));
            }
        } else {
            println!("{}", "[cargo] cargo execution failed".red());
            return Err(BuildStepError::ChildExecutionFailed("Cargo".to_string()));
        }
    }
}
