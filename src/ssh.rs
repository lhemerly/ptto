use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

#[derive(Debug, Clone)]
pub struct SshClient {
    target: String,
    dry_run: bool,
}

impl SshClient {
    pub fn new(target: impl Into<String>, dry_run: bool) -> Self {
        Self {
            target: target.into(),
            dry_run,
        }
    }

    pub fn run(&self, remote_command: &str) -> Result<()> {
        let args = build_ssh_args(&self.target, remote_command);
        if self.dry_run {
            println!("[ptto] dry-run: ssh {}", args.join(" "));
            return Ok(());
        }

        let status = Command::new("ssh")
            .args(&args)
            .status()
            .context("failed to start ssh process")?;

        if !status.success() {
            bail!("ssh command failed with status {status}");
        }

        Ok(())
    }

    pub fn copy_file(&self, local_file: &Path, remote_path: &str) -> Result<()> {
        let args = build_scp_args(&self.target, local_file, remote_path)?;
        if self.dry_run {
            println!("[ptto] dry-run: scp {}", args.join(" "));
            return Ok(());
        }

        let status = Command::new("scp")
            .args(&args)
            .status()
            .context("failed to start scp process")?;

        if !status.success() {
            bail!("scp command failed with status {status}");
        }

        Ok(())
    }
}

fn build_ssh_args(target: &str, remote_command: &str) -> Vec<String> {
    vec![
        "-o".to_string(),
        "BatchMode=yes".to_string(),
        "-o".to_string(),
        "StrictHostKeyChecking=accept-new".to_string(),
        target.to_string(),
        remote_command.to_string(),
    ]
}

fn build_scp_args(target: &str, local_file: &Path, remote_path: &str) -> Result<Vec<String>> {
    let local_file = local_file
        .to_str()
        .context("local file path contains unsupported UTF-8")?;

    Ok(vec![
        "-o".to_string(),
        "BatchMode=yes".to_string(),
        "-o".to_string(),
        "StrictHostKeyChecking=accept-new".to_string(),
        local_file.to_string(),
        format!("{target}:{remote_path}"),
    ])
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{build_scp_args, build_ssh_args};

    #[test]
    fn ssh_args_include_safety_flags() {
        let args = build_ssh_args("root@example.com", "echo ok");
        assert_eq!(
            args,
            vec![
                "-o",
                "BatchMode=yes",
                "-o",
                "StrictHostKeyChecking=accept-new",
                "root@example.com",
                "echo ok"
            ]
        );
    }

    #[test]
    fn scp_args_include_source_and_target() {
        let args = build_scp_args("deployer@example.com", Path::new("./app"), "/tmp/app")
            .expect("valid args");
        assert_eq!(
            args,
            vec![
                "-o",
                "BatchMode=yes",
                "-o",
                "StrictHostKeyChecking=accept-new",
                "./app",
                "deployer@example.com:/tmp/app"
            ]
        );
    }
}
