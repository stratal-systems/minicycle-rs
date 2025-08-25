use std::process::Command;
use std::io;

pub fn check() -> Result<bool, io::Error> {
    let output = Command::new("git")
        .arg("--version")
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim_start().starts_with("git version"))
    } else {
        Ok(false)
    }
}

