use std::fs;
use std::io;
use std::process::Command;

use colored::Colorize;
use indicatif::ProgressBar;

use crate::BuildStep;
use crate::BuildStepError;

pub struct AstGrep {}

impl AstGrep {
    fn scan(rule_path: &str, c_path: &str, bar: &ProgressBar) -> Result<(), BuildStepError> {
        if let Ok(output) = Command::new("ast-grep")
            .args(["scan", "--rule", rule_path, c_path, "--update-all"])
            .output()
        {
            if output.status.success() {
                return Ok(());
            } else {
                bar.println(String::from_utf8_lossy(&output.stderr));
                return Err(BuildStepError::ChildProcessFailed(
                    "ast-grep".to_string(),
                    output.status,
                ));
            }
        } else {
            bar.println("ast-grep execution failed");
            bar.finish_with_message(format!("{}", "Ast-Grep Run Failed".red()));
            return Err(BuildStepError::ChildExecutionFailed("ast-grep".to_string()));
        }
    }
}

impl BuildStep for AstGrep {
    fn run(
        dir: &crate::WorkingDirectory,
        bar: &indicatif::ProgressBar,
    ) -> Result<(), crate::BuildStepError> {
        bar.set_message("Ast-Grep Running...");

        let mut rules = fs::read_dir(format!("{}/ast-grep-rules", dir.ext))?
            .collect::<Result<Vec<_>, io::Error>>()?;
        rules.sort_by_key(|e| e.file_name());

        for rule in rules {
            if let Err(err) = Self::scan(
                &rule.path().to_str().unwrap(),
                format!("{}/out.c", dir.out).as_str(),
                &bar,
            ) {
                bar.finish_with_message(format!("{}", "Ast-Grep Run Failed".red()));
                return Err(err);
            }
        }

        bar.finish_with_message(format!("{}", "Ast-Grep Run Succeeded".green()));
        Ok(())
    }
}
