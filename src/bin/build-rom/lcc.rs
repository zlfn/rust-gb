use std::process::{Command, Stdio};

use colored::Colorize;
use indicatif::ProgressBar;

use crate::{BuildStep, BuildStepError};

pub struct Lcc {}

impl BuildStep for Lcc {
    fn run(dir: &crate::WorkingDirectory, bar: &ProgressBar) -> Result<(), crate::BuildStepError> {
        bar.set_message("GB ROM Linking...");
        let mut asm_path = dir.get_asm_paths();

        let mut lcc_args: Vec<String> = vec![
            "-msm83:gb".to_string(),
            "-o".to_string(),
            format!("{}/out.gb", dir.out),
        ];

        lcc_args.append(&mut asm_path);

        if let Ok(output) = Command::new(format!("{}/bin/lcc", dir.ext))
            .args(lcc_args)
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .output()
        {
            if output.status.success() {
                bar.finish_with_message(format!("{}", "GB ROM Linking Succeeded".green()));
                return Ok(());
            } else {
                bar.println(String::from_utf8_lossy(&output.stderr));
                bar.finish_with_message(format!("{}", "GB ROM Linking Failed".red()));
                return Err(BuildStepError::ChildProcessFailed(
                    "lcc".to_string(),
                    output.status,
                ));
            }
        } else {
            bar.println("lcc executing failed");
            return Err(BuildStepError::ChildExecutionFailed("lcc".to_string()));
        }
    }
}
