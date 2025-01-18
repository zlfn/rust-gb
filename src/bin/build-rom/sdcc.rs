use std::process::Command;

use crate::{BuildStep, BuildStepError};

pub struct Sdcc {}

impl Sdcc {
    pub fn compile(c_path: &str, asm_path: &str) -> Result<(), BuildStepError> {
        if let Ok(status) = Command::new("sdcc")
            .args([
                "-S",
                "-linc",
                "-lsrc",
                "-ltests",
                "-D",
                "__HIDDEN__=",
                "-D",
                "__attribute__(a)=",
                "-D",
                "__builtin_unreachable()=while(1);",
                "--out-fmt-ihx",
                "--max-allocs-per-node",
                "2000",
                "--disable-warning",
                "110",
                "--disable-warning",
                "126",
                "--allow-unsafe-read",
                "--opt-code-speed",
                "--no-std-crt0",
                "--nostdlib",
                "--no-xinit-opt",
                "--std-sdcc11",
                "-msm83",
                "-D",
                "__PORT_sm83",
                "-D",
                "__TARGET_gb",
                c_path,
                "-o",
                asm_path,
            ])
            .status()
        {
            if status.success() {
                return Ok(());
            } else {
                return Err(BuildStepError::ChildProcessFailed(
                    "sdcc".to_string(),
                    status,
                ));
            }
        } else {
            return Err(BuildStepError::ChildExecutionFailed("sdcc".to_string()));
        }
    }
}

impl BuildStep for Sdcc {
    fn run(dir: &crate::WorkingDirectory) -> Result<(), BuildStepError> {
        let c_file = format!("{}/out.c", dir.out);
        let asm_file = format!("{}/out.asm", dir.out);

        Self::compile(&c_file, &asm_file)?;

        let c_path = dir.get_c_paths();
        for c_file in c_path {
            let (_, out_name) = c_file.rsplit_once('/').unwrap();
            let out_name = format!("{}/asm/{}.asm", dir.out, out_name);
            Self::compile(&c_file, out_name.as_str())?;
        }
        Ok(())
    }
}
