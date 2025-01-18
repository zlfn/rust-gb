use std::process::Command;

use crate::{BuildStep, BuildStepError};

pub struct Lcc {}

impl BuildStep for Lcc {
    fn run(dir: &crate::WorkingDirectory) -> Result<(), crate::BuildStepError> {
        let mut asm_path = dir.get_asm_paths();

        let mut lcc_args: Vec<String> = vec![
            "-msm83:gb".to_string(),
            "-o".to_string(),
            format!("{}/out.gb", dir.out),
            format!("{}/out.asm", dir.out),
        ];

        lcc_args.append(&mut asm_path);

        let status = Command::new(format!("{}/bin/lcc", dir.ext))
            .args(lcc_args)
            .status()
            .map_err(|_| BuildStepError::ChildExecutionFailed("lcc".to_string()))?;

        if status.success() {
            return Ok(());
        } else {
            return Err(BuildStepError::ChildProcessFailed(
                "lcc".to_string(),
                status,
            ));
        }
    }
}
